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
}


impl Iterator for PlayingSample {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.finished() {
            return Some(0.0f32);
        }

        let v = self.sample.data[self.position];
        self.position += 1;

        Some(v)
    }
}
