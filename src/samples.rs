use std::ffi::CString;
use std::mem::MaybeUninit;
use std::sync::Arc;

use pyo3::prelude::*;

#[derive(Clone)]
pub struct Sample {
    data: Arc<Vec<f32>>,
    length: usize
}

impl Sample {
    pub fn new(data: Vec<f32>, length: usize) -> Sample {
        Sample {
            data: Arc::new(data),
            length: length
        }
    }
}

#[derive(Clone)]
pub struct LevelSample {
    pub sample: Sample,
    pub level: u8
}

impl LevelSample {
    pub fn new(sample: Sample, level: u8) -> LevelSample {
        LevelSample {
            sample: sample,
            level: level
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SampleHandle {
    pub sample:  Sample,
    pub trig: u8,
    gain: f32,
}

#[pymethods]
impl SampleHandle {

    #[getter]
    fn get_trig(&self) -> PyResult<u8> {
        Ok(self.trig)
    }

    #[setter]
    fn set_trig(&mut self, val: u8) -> PyResult<()> {
        self.trig = val;
        Ok(())
     }
}

impl SampleHandle {
    pub fn from_sample(sample: &Sample) -> SampleHandle {
        SampleHandle {
            sample: sample.clone(),
            trig: 0,
            gain: 0.,
        }
    }
}


#[derive(Clone)]
pub struct PlayingSample {
    pub sample: Sample,
    gain: f32,
    position: usize
}

impl PlayingSample {
    pub fn from_sample(sample: Sample, gain: f32) -> PlayingSample{
        PlayingSample {
            sample: sample,
            gain: gain,
            position: 0
        }
    }


    pub fn finished(&self) -> bool {
        self.position >= self.sample.length
    }

    pub fn advance(&mut self, steps: usize) {
        self.position += steps;
    }

    pub fn iter(&self) -> PlayingSampleIterator {
        PlayingSampleIterator {
            data: self.sample.data.clone(),
            position: self.position
        }
    }
}

pub struct PlayingSampleIterator {
    data: Arc<Vec<f32>>,
    position: usize,
}

impl std::iter::Iterator for PlayingSampleIterator {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.position >= self.data.len() {
            return None
        }

        let v = self.data[self.position];
        self.position += 1;

        Some(v)
    }
}


pub fn load_sample(path: String, gain: f32) -> Result<SampleHandle, String> {
    let c_path = CString::new(path.clone()).unwrap();

    let mut info = MaybeUninit::<sndfile_sys::SF_INFO>::uninit();

    let fh = unsafe {sndfile_sys::sf_open(c_path.as_ptr(), sndfile_sys::SFM_READ, info.as_mut_ptr())};
    if fh == std::ptr::null_mut() {
        return Err(format!("sf_open({},...) failed", path))
    }

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
