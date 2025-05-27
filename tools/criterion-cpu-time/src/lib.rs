use criterion::measurement::{Measurement, ValueFormatter};
use std::time::Duration;

mod formatter;
mod time;

use formatter::DurationFormatter;
pub use time::ProcessTime;

pub enum PosixTime {
    UserTime,
    UserAndSystemTime,
}

impl PosixTime {
    fn get_time(&self) -> Duration {
        // For simplicity, we'll use ProcessTime for both types
        // In a real implementation, you might want to separate user and system time
        let process_time = ProcessTime::new();
        process_time.get_process_time()
    }
}

impl Measurement for PosixTime {
    type Intermediate = Duration;
    type Value = Duration;

    fn start(&self) -> Self::Intermediate {
        self.get_time()
    }

    fn end(&self, i: Self::Intermediate) -> Self::Value {
        self.get_time() - i
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        *v1 + *v2
    }

    fn zero(&self) -> Self::Value {
        Duration::from_secs(0)
    }

    fn to_f64(&self, value: &Self::Value) -> f64 {
        value.as_nanos() as f64
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        &DurationFormatter
    }
}