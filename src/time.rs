use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct Timer {
    start_times: HashMap<String, u128>,
    diff_times: HashMap<String, u128>,
}

impl Timer {
    pub fn new() -> Timer {
        Default::default()
    }

    pub fn start(&mut self, name: &str) {
        self.start_times
            .insert(name.to_string(), Self::curr_time_millis());
    }

    pub fn finish(&mut self, name: &str) -> u128 {
        let start_time = *self.start_times.get(name).unwrap();
        let diff = Self::curr_time_millis() - start_time;
        self.diff_times.insert(name.to_string(), diff);
        return diff;
    }

    pub fn get_res(&self, name: &str) -> u128 {
        *self.diff_times.get(name).unwrap()
    }

    pub fn curr_time_millis() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}
