use std::result::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicPtr, AtomicU64};
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use crossbeam_channel;
use std::collections::HashMap;
use std::ops::Deref;

use pyo3::prelude::*;

mod fader;
use fader::{Fader, FaderRef};

mod processor;
use processor::{Processor, ConnectionMatrix, SharedState, ProcessorMessage, FaderSourceType};

mod samples;
use samples::{Sample, LevelSample, SampleHandle};

mod instrument;
use instrument::{Instrument, InstrumentMap, InstrumentRef};

trait Source {
    fn fill(&self, iterator: &mut std::slice::IterMut<f32>);
}


enum SharedIdx {
    Shared1,
    Shared2
}

#[pyclass(dict)]
struct Core {
    instruments: Arc<HashMap<usize, Box<Instrument>>>,
    master: Fader,
    buses: Vec<Fader>,
    //client: Option<jack::AsyncClient<(), Processor>>,
    drop: Option<Arc<[AtomicPtr<Sample>; 2]>>,
    play_queue: Option<crossbeam_channel::Sender<ProcessorMessage>>,
    play_events: Vec<(u8, u8)>,
    cm: Arc<AtomicPtr<ConnectionMatrix>>,
    gen: Arc<AtomicU64>,
    shidx: SharedIdx,
    shptr: Arc<AtomicPtr<SharedState>>,
    shared1: *mut SharedState,
    shared2: *mut SharedState,
    jack_running: bool,
}

impl Core {
    fn register_jack(&mut self) -> Result<(), & str> {
        let (client, _status) =
            jack::Client::new("Tdrum", jack::ClientOptions::NO_START_SERVER).unwrap();

        let midi_input_port = client
            .register_port("in", jack::MidiIn::default())
            .unwrap();

        let mut audio_output_port = client
            .register_port("out_l", jack::AudioOut::default())
            .unwrap();

        let (process, tx) = Processor::new(midi_input_port, audio_output_port,
                                       self.gen.clone(), self.shptr.clone());
        self.drop = Some(process.get_drop_list());
        self.play_queue = Some(tx);

        //let aclient = client.activate_async((), process).unwrap();
        client.activate_async((), process).unwrap();
        self.jack_running = true;
        //self.client = Some(aclient);
        Ok(())
    }


    fn get_shared_state(&mut self) -> &mut SharedState {
        unsafe {
            match &self.shidx {
                Shared1 => &mut *self.shared1,
                Shared2 => &mut *self.shared2
            }
        }
    }

    fn get_shadow_state(&mut self) -> &mut SharedState {
        unsafe {
            match &self.shidx {
                Shared1 => &mut *self.shared2,
                Shared2 => &mut *self.shared1
            }
        }
    }

    fn swap_states(&mut self) {
        if !self.jack_running {return;}

        let s: *mut _ = self.get_shadow_state();
        self.shptr.store(s, Relaxed);

        self.shidx = match &self.shidx {
            Shared1 => SharedIdx::Shared2,
            Shared2 => SharedIdx::Shared1
        };

        let mut generation = self.gen.load(Relaxed);
        loop {
            if generation & 1 == 0 {return;}
            std::thread::sleep_ms(1);
            let new_generation = self.gen.load(Relaxed);

            if new_generation != generation {return;}
            generation = new_generation;
        }

    }

    fn with_swap_states<F>(&mut self, f: F)
    where F: Fn(&mut SharedState) -> () {

        let s1 =
            if self.jack_running {
                self.get_shadow_state()
            } else {
                self.get_shared_state()
            };
        f(s1);
        self.swap_states();

        let s2 = self.get_shadow_state();
        f(s2);
    }

    fn update_instrument(&mut self, note: u8, iref: &InstrumentRef) {
        println!("Adding instrument for note {}", note);
        let idx = iref.tcid;
        let s1 = self.get_shadow_state();
        self.with_swap_states(|s| {s.note_map.insert((note & 0xf), idx); ()});
    }

    fn update_bus(&mut self, name: &str, bus: &mut Fader) {
        self.buses.push(bus.clone());
    }

    fn get_master(&self) -> Fader {
        self.master.clone()
    }

    fn instrument_remove(&mut self, note: u8) {
        unimplemented!();
        //self.cur_instr[(note & 0xf) as usize].store(std::ptr::null_mut(), Relaxed);
    }


    fn instrument_play(&mut self, instr: &InstrumentRef, velocity: u8) {
        let idx = instr.tcid;
        println!("Playing instrument {} with velocity {}", idx, velocity);
        if let Some(sender) = &self.play_queue {
            sender.send(ProcessorMessage::PlayInstrument {
                iptr: idx,
                velocity: velocity
            });
        }
    }

    fn rebuild_signal_chain(&self, master: &mut Fader) {
        unimplemented!();
        // let mut new_connection_matrix = ConnectionMatrix::new();
        // let master_inputs = init_array!(AtomicPtr<Fader>, processor::MAX_INPUTS, AtomicPtr::default());
        // master_inputs[0].store(master, Relaxed);
        // new_connection_matrix.insert(master.name.clone(), master_inputs);

        // self.cm.store(&mut new_connection_matrix , Relaxed);
    }


    fn set_instrument_note(&self, note: u16, instrument: &Instrument) {
    }


}

#[pymethods]
impl Core {

    #[new]
    fn new(obj: &PyRawObject) {
        let state = Box::new(SharedState::new());
        let shadow = Box::new(SharedState::new());
        let mut core = Self {
            instruments: Arc::new(HashMap::new()),
            master: Fader::initu32("Master", 0),
            buses: Vec::new(),
            //client: None,
            drop: None,
            play_queue: None,
            play_events: Vec::new(),
            cm: Arc::new(AtomicPtr::default()),
            gen: Arc::new(AtomicU64::new(0)),
            shidx: SharedIdx::Shared1,
            shptr: Arc::new(AtomicPtr::default()),
            shared1: Box::into_raw(state),
            shared2: Box::into_raw(shadow),
            jack_running: false
        };

        let mref = core.fader_new("Master");
        core.get_shared_state().set_master(mref.clone());
        core.get_shadow_state().set_master(mref);

        core.shptr.store(core.shared1, Relaxed);

        obj.init({
            core
        });
    }

    fn get_master_fader(&self) -> FaderRef {
        let state_ptr = self.shptr.load(Relaxed);
        let state: &SharedState =  unsafe {&*state_ptr};
        state.master.clone()
    }

    fn fader_new(&mut self, name: &str) -> FaderRef {
        let fad = Box::new(Fader::initf64(name, 1.0f64));
        let fptr = Box::into_raw(fad);
        self.with_swap_states(|s| {s.fader_map.insert(fptr as usize, fptr); ()});

        FaderRef{
            tcid: fptr as usize
        }
    }

    fn instrument_new(&mut self, name: &str) -> InstrumentRef {
        let instr = Box::new(Instrument::create(name));
        let iptr = Box::into_raw(instr);
        self.with_swap_states(|s| {s.instr_map.insert(iptr as usize, iptr);; ()});

        InstrumentRef{
            tcid: iptr as usize
        }
    }

    fn fader_add_fader_src(&mut self, dst: &FaderRef, src: &FaderRef) {
        self.with_swap_states(|s| {
            let mut srcs = s.fsrc_map.entry(dst.tcid).or_default();

            if !srcs.iter().any(|s| s == &FaderSourceType::FaderSrc(src.tcid)) {
                srcs.push(FaderSourceType::FaderSrc(src.tcid));
            }
        });
    }

    fn fader_add_instrument_src(&mut self, dst: &FaderRef, src: &InstrumentRef) {
        self.with_swap_states(|s| {
            let mut srcs = s.fsrc_map.entry(dst.tcid).or_default();

            if !srcs.iter().any(|s| s == &FaderSourceType::InstrumentSrc(src.tcid)) {
                srcs.push(FaderSourceType::InstrumentSrc(src.tcid));
            }
        });
    }

    fn fader_test_method(&mut self, fader: &FaderRef, other_fader: &FaderRef, some_int: i32) {
        println!("Test method called with fader {}, {}, {}", fader.tcid, other_fader.tcid, some_int);
    }

    fn fader_set_gain(&mut self, fader: &FaderRef, gain: f32) {
        let state = self.get_shared_state();
        if let Some(mut ptr) = state.fader_map.get(&fader.tcid) {
            let f: &Fader = unsafe{&**ptr};
            f.set_gain(gain);
        }
    }


}

#[pymodule]
fn tdrum(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Core>()?;
    m.add_class::<Instrument>()?;
    m.add_class::<Fader>()?;
    Ok(())
}
