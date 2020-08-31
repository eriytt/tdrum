use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicU64};
use std::sync::atomic::Ordering::Relaxed;
use crossbeam_channel;

use pyo3::prelude::*;
use pyo3::exceptions;

mod fader;
use fader::{Fader, FaderRef};

mod processor;
use processor::{Processor, SharedState, ProcessorMessage, FaderSourceType};

mod samples;
use samples::{Sample, SampleHandle};

mod instrument;
use instrument::{Instrument, InstrumentRef};

trait Source {
    fn fill(&self, iterator: &mut std::slice::IterMut<f32>);
}

#[derive(Debug)]
enum SharedIdx {
    Shared1,
    Shared2
}

#[pyclass(unsendable)]
struct Core {
    client: Option<jack::AsyncClient<(), Processor>>,
    drop: Option<Arc<[AtomicPtr<Sample>; 2]>>,
    play_queue: Option<crossbeam_channel::Sender<ProcessorMessage>>,
    gen: Arc<AtomicU64>,
    shidx: SharedIdx,
    shptr: Arc<AtomicPtr<SharedState>>,
    shared1: *mut SharedState,
    shared2: *mut SharedState,
    jack_running: bool,
}

impl Core {

    fn get_shared_state(&mut self) -> &mut SharedState {
        unsafe {
            match &self.shidx {
                SharedIdx::Shared1 => &mut *self.shared1,
                SharedIdx::Shared2 => &mut *self.shared2
            }
        }
    }

    fn get_shadow_state(&self) -> &mut SharedState {
        unsafe {
            match &self.shidx {
                SharedIdx::Shared1 => &mut *self.shared2,
                SharedIdx::Shared2 => &mut *self.shared1,
            }
        }
    }

    fn swap_states(&mut self) {
        if !self.jack_running {return;}

        let s: *mut _ = self.get_shadow_state();
        self.shptr.store(s, Relaxed);

        self.shidx = match &self.shidx {
            SharedIdx::Shared1 => SharedIdx::Shared2,
            SharedIdx::Shared2 => SharedIdx::Shared1
        };

        let mut generation = self.gen.load(Relaxed);
        loop {
            if generation & 1 == 0 {return;}
            std::thread::sleep(std::time::Duration::from_millis(1));
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
}

#[pymethods]
impl Core {

    #[new]
    fn new() -> Self {
        let state = Box::new(SharedState::new());
        let shadow = Box::new(SharedState::new());
        let mut core = Self {
            client: None,
            drop: None,
            play_queue: None,
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
        core
    }

    fn register_jack(&mut self) -> PyResult<()> {
        let (client, _status) =
            jack::Client::new("Tdrum", jack::ClientOptions::NO_START_SERVER).unwrap();

        let midi_input_port = client
            .register_port("in", jack::MidiIn::default())
            .unwrap();

        let audio_output_port_left = client
            .register_port("out_l", jack::AudioOut::default())
            .unwrap();

        let audio_output_port_right = client
            .register_port("out_r", jack::AudioOut::default())
            .unwrap();


        let (process, tx) = Processor::new(midi_input_port,
                                           audio_output_port_left, audio_output_port_right,
                                           self.gen.clone(), self.shptr.clone());
        self.drop = Some(process.get_drop_list());
        self.play_queue = Some(tx);

        let aclient = client.activate_async((), process).unwrap();
        self.client = Some(aclient);
        self.jack_running = true;
        Ok(())
    }

    fn instrument_new(&mut self, name: &str) -> InstrumentRef {
        let instr = Box::new(Instrument::create(name));
        let iptr = Box::into_raw(instr);
        self.with_swap_states(|s| {s.instr_map.insert(iptr as usize, iptr); ()});

        let instrument = InstrumentRef{
            tcid: iptr as usize
        };

        let fader = self.fader_new(name);
        self.fader_add_instrument_src(&fader, &instrument);
        instrument
    }

    fn instrument_delete(&mut self, instrument: &InstrumentRef) {
        if let Some(fader_id) = self.get_shadow_state().find_instrument_fader_idx(instrument) {
            // The instrument fader might have been removed, but its source records might not
            let fader_exists = self.get_shadow_state().fader_map.contains_key(&fader_id);
            self.with_swap_states(|s| {
                s.fsrc_map.remove(&fader_id);
                s.fader_map.remove(&fader_id);
            });
            if fader_exists {
                drop(unsafe {Box::from_raw(fader_id as *mut Fader)});
            }
        }

        self.with_swap_states(|s| {
            s.note_map.retain(|_note, id| id != &instrument.tcid);
            s.instr_map.remove(&instrument.tcid);
        });

        drop(unsafe {Box::from_raw(instrument.tcid as *mut Instrument)});
    }

    fn instrument_set_note(&mut self, instrument: &InstrumentRef, note: u16) {
        self.with_swap_states(|s| {s.note_map.insert((note & 0xf) as u8, instrument.tcid); ()});
    }

    fn instrument_add_sample(&mut self, instrument: &InstrumentRef, sample: &mut SampleHandle) -> PyResult<()> {
        let iptr = match self.get_shadow_state().instr_map.get(&instrument.tcid) {
            Some(iptr) => *iptr,
            None => return Err(PyErr::new::<exceptions::LookupError, _>("Instrument not found"))
        };
        self.with_swap_states(|s| {s.instr_map.remove(&instrument.tcid); ()});

        let i: &mut Instrument = unsafe{&mut *iptr};
        i.add_sample(sample);

        self.with_swap_states(|s| {s.instr_map.insert(instrument.tcid, iptr); ()});
        Ok(())
    }

    fn instrument_get_fader(&self, instrument: &InstrumentRef)  -> PyResult<FaderRef> {
        let src_entry = self.get_shadow_state().find_instrument_fader_idx(instrument);
        match src_entry {
            Some(k) => Ok(FaderRef{tcid: k}),
            None => Err(PyErr::new::<exceptions::LookupError, _>("Fader not found"))
        }
    }

    fn instrument_play(&mut self, instr: &InstrumentRef, velocity: u8) -> PyResult<bool> {
        let idx = instr.tcid;
        println!("Playing instrument {} with velocity {}", idx, velocity);

        match &self.play_queue {
            None => Ok(false),
            Some(sender) =>
                match sender.send(ProcessorMessage::PlayInstrument {
                    iptr: idx,
                    velocity: velocity
                }) {
                    Ok(_) => Ok(true),
                    Err(e)    => Err(PyErr::new::<exceptions::RuntimeError, _>(format!("Failed to play instrument '{}'", e))),
                }
        }
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

    fn fader_delete(&mut self, fader_ref: &FaderRef) {
        self.with_swap_states(|s| {
            s.delete_fader_source(&fader_ref);
            s.fader_map.remove(&fader_ref.tcid);
        });

        drop(unsafe {Box::from_raw(fader_ref.tcid as *mut Fader)});
    }

    fn fader_get_internal_id(&mut self, fader_ref: &FaderRef) -> PyResult<usize> {
        Ok(fader_ref.tcid)
    }

    fn fader_add_fader_src(&mut self, dst: &FaderRef, src: &FaderRef) {
        self.with_swap_states(|s| {
            let srcs = s.fsrc_map.entry(dst.tcid).or_default();

            if !srcs.iter().any(|s| s == &FaderSourceType::FaderSrc(src.tcid)) {
                srcs.push(FaderSourceType::FaderSrc(src.tcid));
            }
        });
    }

    fn fader_del_fader_src(&mut self, dst: &FaderRef, src: &FaderRef) {
        self.with_swap_states(|s| {
            let srcs = s.fsrc_map.entry(dst.tcid).or_default();
            srcs.retain(|fs| fs != &FaderSourceType::FaderSrc(src.tcid))
        });
    }

    fn fader_add_instrument_src(&mut self, dst: &FaderRef, src: &InstrumentRef) {
        self.with_swap_states(|s| {
            let srcs = s.fsrc_map.entry(dst.tcid).or_default();

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
        if let Some(ptr) = state.fader_map.get(&fader.tcid) {
            let f: &Fader = unsafe{&**ptr};
            f.set_gain(gain);
        }
    }

    fn fader_get_gain(&mut self, fader: &FaderRef) -> PyResult<f32> {
        let state = self.get_shared_state();
        match state.fader_map.get(&fader.tcid) {
            Some(ptr) => {
                let f: &Fader = unsafe{&**ptr};
                Ok(f.get_gain())
            }
            None => Err(PyErr::new::<exceptions::IndexError, _>(format!("No fader '{}'", fader.tcid)))
        }
    }

    fn fader_set_panning(&mut self, fader: &FaderRef, panning: f32) {
        let state = self.get_shared_state();
        if let Some(ptr) = state.fader_map.get(&fader.tcid) {
            let f: &Fader = unsafe{&**ptr};
            f.set_panning(panning);
        }
    }

}

#[pymodule]
fn tdrum(_py: Python, m: &PyModule) -> PyResult<()> {

    #[pyfn(m, "load_sample")]
    fn load_sample(_py: Python, path: String) -> PyResult<SampleHandle> {
        match samples::load_sample(path, 1.0f32) {
            Ok(handle) => Ok(handle),
            Err(s) => Err(PyErr::new::<exceptions::IOError, _>(s))
        }
    }

    m.add_class::<Core>()?;
    Ok(())
}
