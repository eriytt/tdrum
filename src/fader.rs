use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pyo3::prelude::*;

fn unit_int_to_float(int: u32) -> f32 {
    (int as f64 / std::u32::MAX as f64) as f32
}

fn unit_float_to_int(float: f64) -> u32 {
    ((std::u32::MAX as f64) * float) as u32
}

#[pyclass]
#[derive(Clone)]
pub struct Fader {
    pub name: String,
    gain: Arc<AtomicU32>,
}

impl Fader {
    pub fn initf64(name: &str, v: f64) -> Fader {
        let ival = unit_float_to_int(v);//(std::u32::MAX as f64 * v) as u32;
        Fader {
            name: name.to_string(),
            gain: Arc::new(AtomicU32::new(ival)),
        }
    }

    pub fn initu32(name: &str, v: u32) -> Fader {
        Fader {
            name: name.to_string(),
            gain: Arc::new(AtomicU32::new(v))
        }
    }

    pub fn get_gain(&self) -> f32 {
        //let ival = (std::u32::MAX as f64 * gain as f64) as u32;
        //(self.gain.load(Relaxed) as f64 / std::u32::MAX as f64) as f32
        unit_int_to_float(self.gain.load(Relaxed))
    }

}

#[pymethods]
impl Fader {
    #[new]
    fn new(obj: &PyRawObject, name: &str) {
        obj.init({
            Fader::initu32(name, 0)
        });
    }

    fn clone(&self) -> Fader {
        Fader::initu32(&self.name, self.gain.load(Relaxed))
    }

    #[allow(non_snake_case)]
    fn addSource(&self, src: i64) {
    }

    #[allow(non_snake_case)]
    fn setDownstream(&self, fader: i64) {
    }


    #[allow(non_snake_case)]
    fn getGain(&self) -> PyResult<f32> {
        Ok(0.0)
    }

    fn set_gain(&self, gain: f32) {
        let ival = (std::u32::MAX as f64 * gain as f64) as u32;
        self.gain.store(ival, Relaxed);
    }

    #[allow(non_snake_case)]
    fn registerJackPorts(&self, client: i64) {
    }
}

#[pyclass]
pub struct FaderRef {
    pub tcid: usize
}

impl Clone for FaderRef {
    fn clone(&self) -> FaderRef {
        FaderRef{tcid: self.tcid}
    }
}
