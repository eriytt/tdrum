use std::ffi::CString;
use std::mem::MaybeUninit;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicPtr;

use pyo3::prelude::*;

use super::fader::Fader;
use super::samples::{LevelSample, Sample, SampleHandle};

pub type InstrumentMap = [AtomicPtr<Instrument>; 128];

#[pyclass]
pub struct InstrumentRef {
    pub tcid: usize
}

#[pyclass]
#[derive(Clone)]
pub struct Instrument {
    fader: Fader,
    sample_store: HashMap<String, Sample>,
    sample_levels: Vec<LevelSample>
}

impl Instrument {
    pub fn sample_for_level(&self, level: u8) -> Sample {
        let mut lsample = &self.sample_levels[0];
        for ls in &self.sample_levels[1..] {
            if level > ls.level {
                lsample = ls;
            } else {
                break;
            }
        }
        return lsample.sample.clone();
    }
}

impl Instrument {
    pub fn create(name: &str) -> Instrument {
        Instrument {
            fader: Fader::initu32(name, 0),
            sample_store: HashMap::new(),
            sample_levels: Vec::new()
        }
    }
}


#[pymethods]
impl Instrument{

     #[new]
    fn new(obj: &PyRawObject, py: Python, name: &str) {
        obj.init({
            Instrument::create(name)
         });
    }

    fn add_sample(&mut self, path: String, sample: &mut SampleHandle) -> PyResult<()> {
        println!("Adding sample");
        self.sample_store.insert(path, sample.sample.clone());
        self.sample_levels.push(LevelSample::new(sample.sample.clone(), sample.trig));
        Ok(())
    }

    #[allow(non_snake_case)]
    fn loadSample(&mut self, path: String, gain: f32) -> PyResult<SampleHandle> {
        let c_path = CString::new(path.clone()).unwrap();

        let mut info = MaybeUninit::<sndfile_sys::SF_INFO>::uninit();

        let fh = unsafe {sndfile_sys::sf_open(c_path.as_ptr(), sndfile_sys::SFM_READ, info.as_mut_ptr())};
        let info = unsafe { info.assume_init() };

        let items = info.frames * info.channels as i64;
        let mut data = Vec::<f32>::with_capacity(items as usize);
        unsafe {data.set_len(items as usize)};
        let read_items = unsafe {sndfile_sys::sf_read_float(fh, data.as_mut_ptr(), items)};
        unsafe {sndfile_sys::sf_close(fh)};

        if read_items != items { panic!("Read error")} // TODO: throw IOError

        println!("Loading sample of size {} ({})", items, data.len());
        let sample = Sample::new(data, items as usize);
        // self.sample_store.insert(path, sample.clone());
        // self.sample_levels.push(LevelSample::new(sample.clone(), 127));

        // TODO: this is a wierd way to have a handle
        Ok(SampleHandle::from_sample(&sample))
    }

    #[allow(non_snake_case)]
    fn setVelocity(&self, sample: i64, velocity: u8) {
    }

    // #[allow(non_snake_case)]
    // fn setFader(&mut self, fader: &Fader) {
    //     //self.fader = Some(*fader);
    // }

    fn get_fader(&self, py: Python) -> PyResult<Fader> {
        Ok(self.fader.clone())
    }

}
