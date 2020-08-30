use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicU64};
use std::sync::atomic::Ordering::Relaxed;
use std::collections::HashMap;
use std::ops::Deref;

use super::instrument::{Instrument, InstrumentRef};
use super::samples::{PlayingSample, Sample};
use super::fader::{Fader, FaderRef};

pub const MAX_INPUTS: usize = 10;

pub type ConnectionMatrix = HashMap<String, [AtomicPtr<Fader>; MAX_INPUTS]>;

#[derive(PartialEq, Debug)]
pub enum FaderSourceType {
    FaderSrc(usize),
    InstrumentSrc(usize),
}

#[derive(PartialEq)]
pub enum Channel {
    Left,
    Right
}

struct VecZip {
    vec: Vec<Box<dyn Iterator<Item = f32>>>
}

impl VecZip {
    fn from_vec(ivec: Vec<Box<dyn Iterator<Item = f32>>>) -> Self{
        Self {
            vec: ivec
        }
    }

    fn from_iter(iter: impl Iterator<Item = Box<dyn Iterator<Item = f32> > >) -> Self {
        use std::iter::FromIterator;
        Self::from_vec(Vec::from_iter(iter))
    }

}

impl Iterator for VecZip {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let v: Vec<Option<f32>> = self.vec.iter_mut().map(|i| i.next()).collect();
        match v.iter().any(|o| match o {None => true, _ => false}) {
            false => Some(v.iter().map(|o| match o {
                Some(f) => *f,
                _ => 0.0f32 }).sum()),
            true =>  None
        }
    }
}

#[derive(Debug)]
pub struct SharedState {
    pub instr_map: HashMap<usize, *mut Instrument>,
    pub fader_map: HashMap<usize, *mut Fader>,
    pub note_map: HashMap<u8, usize>,
    pub fsrc_map: HashMap<usize, Vec<FaderSourceType>>,
    buses: Vec<Fader>,
    cm: ConnectionMatrix,
    pub master: FaderRef,
}


impl SharedState {
    pub fn new() -> SharedState {
        SharedState {
            instr_map: HashMap::new(),
            fader_map: HashMap::new(),
            note_map: HashMap::new(),
            fsrc_map: HashMap::new(),
            buses: Vec::new(),
            cm: HashMap::new(),
            master: FaderRef{tcid: 0},
        }
    }

    pub fn set_master(&mut self, fref: FaderRef) {
        self.master = fref;
    }

    pub fn find_instrument_fader_idx(&self, instrument: &InstrumentRef)
                                 -> Option<usize> {
        self.fsrc_map.iter().find(
            |(_k, src_vec)| src_vec.iter().any(
                |s| match s {
                    FaderSourceType::FaderSrc(_idx) => false,
                    FaderSourceType::InstrumentSrc(idx) => idx == &instrument.tcid
                }
            )).map(|(k, _)| *k)
    }

    pub fn delete_fader_source(&mut self, fader: &FaderRef) {
        for (_, sources) in self.fsrc_map.iter_mut() {
            sources.retain(|fs| fs != &FaderSourceType::FaderSrc(fader.tcid))
        }
    }
}

struct StateGuard<'a> {
    generation: &'a AtomicU64,
    state: &'a SharedState,
}

impl<'a> StateGuard<'a> {
    fn new(gen: &'a AtomicU64, state: &'a AtomicPtr<SharedState>) -> StateGuard<'a> {
        gen.fetch_add(1, Relaxed);
        let state_ptr = state.load(Relaxed);
        StateGuard {
            generation: gen,
            state: unsafe {&*state_ptr}
        }
    }
}

impl<'a> Drop for StateGuard<'a> {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Relaxed);
    }
}

impl<'a> Deref for StateGuard<'a> {
    type Target = SharedState;

    fn deref(&self) -> &SharedState {
        self.state
    }
}

pub enum ProcessorMessage {
    PlayInstrument {iptr: usize, velocity: u8},
}

pub struct Processor {
    midi_port: jack::Port<jack::MidiIn>,
    audio_port_left:  jack::Port<jack::AudioOut>,
    audio_port_right: jack::Port<jack::AudioOut>,
    samples: HashMap<usize, Vec<PlayingSample>>,
    drop: Arc<[AtomicPtr<Sample>; 2]>,
    generation: Arc<AtomicU64>,
    shared: Arc<AtomicPtr<SharedState>>,
    messages: crossbeam_channel::Receiver<ProcessorMessage>,
}

impl Processor {
    pub fn new(midi_port: jack::Port<jack::MidiIn>,
               audio_port_left: jack::Port<jack::AudioOut>,
               audio_port_right: jack::Port<jack::AudioOut>,
               generation: Arc<AtomicU64>,
               shared: Arc<AtomicPtr<SharedState>>,
    ) -> (Processor, crossbeam_channel::Sender<ProcessorMessage>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (Processor {
            midi_port: midi_port,
            audio_port_left: audio_port_left,
            audio_port_right: audio_port_right,
            samples: HashMap::new(),
            drop: Arc::new([AtomicPtr::default(), AtomicPtr::default()]),
            generation: generation,
            shared: shared,
            messages: rx,
        }, tx)
    }

    fn get_instrument_for_index(&self, idx: usize, state: &SharedState) -> Option<&Instrument> {
        match state.instr_map.get(&idx) {
            None => None,
            Some(iptr) => Some(Box::leak(unsafe {Box::from_raw(*iptr)}))
        }
    }

    #[allow(dead_code)]
    fn get_fader_for_index(&self, idx: usize, state: &SharedState) -> Option<&Fader> {
        match state.fader_map.get(&idx) {
            None => None,
            Some(fptr) => Some(Box::leak(unsafe {Box::from_raw(*fptr)}))
        }
    }

    fn get_instrument_for_note(&self, note: u8, state: &SharedState) -> Option<(&Instrument, usize)> {
        println!("Looking up instrument for note {}", note);
        match state.note_map.get(&note) {
            None => None,
            Some(iidx) => match self.get_instrument_for_index(*iidx, state) {
                Some(instr) => Some((instr, *iidx)),
                None => None
            }
        }
    }

    fn get_fader_src_iter(&self, fidx: usize, channel: &Channel, state: &SharedState)
                          -> Box<dyn Iterator<Item = f32>> {
        let (gain, pan) = match state.fader_map.get(&fidx) {
            None => (0.0f32, 0.5),
            Some(f) => {
                let fader = unsafe{&**f};
                (fader.get_gain(), fader.get_panning())
            }
        };

        let pan_gain = match channel {
            Channel::Left  => pan,
            Channel::Right => 1.0 - pan,
        };

        match state.fsrc_map.get(&fidx) {
            None => Box::new(std::iter::repeat(0.0f32)),
            Some(v) => {
                let iters = v.iter().map(|i| {
                    match i {
                        FaderSourceType::FaderSrc(fidx) => self.get_fader_src_iter(*fidx, &channel, state),
                        FaderSourceType::InstrumentSrc(iidx) => self.get_instr_src_iter(*iidx)
                    }
                });
                Box::new(VecZip::from_iter(iters).map(move |amplitude| amplitude * gain * pan_gain))
            }
        }
    }


    fn get_instr_src_iter(&self, iidx: usize) -> Box<dyn Iterator<Item = f32>> {
        match self.samples.get(&iidx) {
            None => Box::new(std::iter::repeat(0.0f32)),
            Some(v) => {
                let iters = v.iter().map(
                    |ps: &PlayingSample| Box::new(ps.iter().chain(std::iter::repeat(0.0f32)))
                        as Box<dyn Iterator<Item = f32>>
                );
                Box::new(VecZip::from_iter(iters))
            }
        }
    }

    pub fn get_drop_list(&self) -> Arc<[AtomicPtr<Sample>; 2]> {
        self.drop.clone()
    }
}

impl jack::ProcessHandler for Processor {
    fn process(&mut self, _client: &jack::Client, process_scope: &jack::ProcessScope) -> jack::Control {
        {
            let state = StateGuard::new(&self.generation, &self.shared);
            let midi_events = self.midi_port.iter(process_scope).collect::<std::vec::Vec<jack::RawMidi>>();

            for e in midi_events {
                if (e.bytes[0] & 0xf0) == 0x90 && e.bytes[1] > 0 {
                    let note = e.bytes[1];
                    let velocity = e.bytes[2];

                    let (instr, idx) = match self.get_instrument_for_note(note, &state) {
                        Some((instr, idx)) => (instr, idx),
                        None => continue
                    };

                    let sample = instr.sample_for_level(velocity);

                    self.samples.entry(idx).or_insert(Vec::new()).push(PlayingSample::from_sample(sample, 1.0))
                }
            }

            for m in self.messages.try_iter() {
                match m {
                    ProcessorMessage::PlayInstrument{iptr, velocity} => {
                        if let Some(instr) = self.get_instrument_for_index(iptr, &state) {
                            let sample = instr.sample_for_level(velocity);
                            self.samples.entry(iptr).or_insert(Vec::new()).push(PlayingSample::from_sample(sample, 1.0));
                        }
                    },
                }
            }

            for channel in vec![Channel::Left,  Channel::Right] {
                let fiter = self.get_fader_src_iter(state.master.tcid, &channel, &state);
                let audio_port = match channel {
                    Channel::Left  => &mut self.audio_port_left,
                    Channel::Right => &mut self.audio_port_right,
                };
                let out = audio_port.as_mut_slice(process_scope);
                for (ou, v) in out.iter_mut().zip(fiter) {
                    *ou = v;
                }
            }
        }

        self.samples.values_mut().for_each(
            |psv| {
                psv.iter_mut().for_each(|ps| ps.advance(process_scope.n_frames() as usize));
                psv.retain(|ps| !ps.finished());
            }
        );

        jack::Control::Continue
    }
}
