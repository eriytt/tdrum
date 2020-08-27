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

#[derive(Clone)]
pub struct Instrument {
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
            sample_store: HashMap::new(),
            sample_levels: Vec::new()
        }
    }
}


impl Instrument{
    pub fn add_sample(&mut self, sample: &mut SampleHandle) {
        println!("Adding sample");
        //self.sample_store.insert(path, sample.sample.clone());
        self.sample_levels.push(LevelSample::new(sample.sample.clone(), sample.trig));
    }

    fn set_velocity(&self, sample: i64, velocity: u8) {
    }
}
