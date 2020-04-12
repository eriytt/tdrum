use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicU64};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc;
use std::collections::HashMap;
use std::ops::Deref;

use super::instrument::{Instrument, InstrumentMap, InstrumentRef};
use super::samples::{PlayingSample, LevelSample, Sample};
use super::fader::{Fader, FaderRef};

pub const MAX_INPUTS: usize = 10;

pub type ConnectionMatrix = HashMap<String, [AtomicPtr<Fader>; MAX_INPUTS]>;

type PlayQueue = Arc<[AtomicPtr<(u8, u8)>; 2]>;

#[derive(PartialEq)]
pub enum FaderSourceType {
    FaderSrc(usize),
    InstrumentSrc(usize),
}

struct VecZip {
    vec: Vec<Box<Iterator<Item = f32>>>
}
use std::iter::FromIterator;

fn ps_2_boxediter(ps: &PlayingSample) -> Box<Iterator<Item = f32>>
{
    Box::new(ps.clone().into_iter())
}

impl VecZip {
    fn from_vec(ivec: Vec<Box<Iterator<Item = f32>>>) -> Self{
        Self {
            vec: ivec
        }
    }

    fn from_iter(iter: impl Iterator<Item = Box<Iterator<Item = f32> > >) -> Self {
        Self::from_vec(Vec::from_iter(iter))
    }

}

impl Iterator for VecZip {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let v: Vec<Option<f32>> = self.vec.iter_mut().map(|mut i| i.next()).collect();
        match v.iter().any(|o| match o {None => true, _ => false}) {
            false => Some(v.iter().map(|o| match o {
                Some(f) => *f,
                _ => 0.0f32 }).sum()),
            true =>  None
        }
    }
}

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
            |(k, src_vec)| src_vec.iter().any(
                |s| match s {
                    FaderSourceType::FaderSrc(idx) => false,
                    FaderSourceType::InstrumentSrc(idx) => idx == &instrument.tcid
                }
            )).map(|(k, v)| *k)
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
    audio_port: jack::Port<jack::AudioOut>,
    // instrument_map: InstrumentMap,
    // faders: Vec<Fader>,
    // master: Fader,
    samples: HashMap<usize, Vec<PlayingSample>>,
    drop: Arc<[AtomicPtr<Sample>; 2]>,
    time: f64,
    generation: Arc<AtomicU64>,
    shared: Arc<AtomicPtr<SharedState>>,
    messages: crossbeam_channel::Receiver<ProcessorMessage>,
    // cm: Arc<AtomicPtr<ConnectionMatrix>>,
}

impl Processor {
    pub fn new(midi_port: jack::Port<jack::MidiIn>,
               audio_port: jack::Port<jack::AudioOut>,
               generation: Arc<AtomicU64>,
               shared: Arc<AtomicPtr<SharedState>>,
    ) -> (Processor, crossbeam_channel::Sender<ProcessorMessage>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (Processor {
            midi_port: midi_port,
            audio_port: audio_port,
            // instrument_map: instr_map,
            // faders: faders,
            // master: master,
            samples: HashMap::new(),
            drop: Arc::new([AtomicPtr::default(), AtomicPtr::default()]),
            time: 0.0,
            // cm: cm,
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

    fn get_fader_src_iter(&self, fidx: usize, state: &SharedState) -> Box<Iterator<Item = f32>> {
        match state.fsrc_map.get(&fidx) {
            None => Box::new(std::iter::repeat(0.0f32)),
            Some(v) => {
                let iters = v.iter().map(|i| {
                    match i {
                        FaderSourceType::FaderSrc(fidx) => self.get_fader_src_iter(*fidx, state),
                        FaderSourceType::InstrumentSrc(iidx) => self.get_instr_src_iter(*iidx)
                    }
                });
                Box::new(VecZip::from_iter(iters))
            }
        }
    }


    fn get_instr_src_iter(&self, iidx: usize) -> Box<Iterator<Item = f32>> {
        match self.samples.get(&iidx) {
            None => Box::new(std::iter::repeat(0.0f32)),
            Some(v) => {
                let iters = v.iter().map(|ps| ps_2_boxediter(ps));
                Box::new(VecZip::from_iter(iters))
            }
        }
    }

    // fn find_drop_slot(&self) -> i8 {
    //     for (i, dptr) in self.drop.iter().enumerate() {
    //         match unsafe {dptr.load(Relaxed).as_ref()} {
    //             None => return i as i8,
    //             Some(x) => continue
    //         }
    //     }
    //     -1
    // }

    pub fn get_drop_list(&self) -> Arc<[AtomicPtr<Sample>; 2]> {
        self.drop.clone()
    }

    // fn get_master(&self) -> Fader {
    //     self.master.clone()
    // }
}

impl jack::ProcessHandler for Processor {
    fn process(&mut self, _client: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let state = StateGuard::new(&self.generation, &self.shared);
        let midi_events = self.midi_port.iter(ps).collect::<std::vec::Vec<jack::RawMidi>>();

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

                        println!("{} live samples", self.samples.len());
                        self.samples.entry(iptr).or_insert(Vec::new()).push(PlayingSample::from_sample(sample, 1.0));
                    }
                },
            }
        }


        let fiter = self.get_fader_src_iter(state.master.tcid, &state);
        let out = self.audio_port.as_mut_slice(ps);
        for (ou, v) in out.iter_mut().zip(fiter) {
            *ou = v;
        }

        jack::Control::Continue
    }
}
