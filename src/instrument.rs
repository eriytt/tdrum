use std::collections::HashMap;

use pyo3::prelude::*;

use super::samples::{LevelSample, Sample, SampleHandle};

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
    pub fn create(_name: &str) -> Instrument {
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
}
