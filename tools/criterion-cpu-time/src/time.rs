use std::time::Duration;
use criterion::measurement::{Measurement, ValueFormatter};

#[cfg(unix)]
use libc::{timespec, timeval, c_long, time_t};

#[cfg(unix)]
struct Timeval {
    tv_sec: time_t,
    tv_usec: c_long,
}

#[cfg(unix)]
struct Rusage {
    ru_utime: Timeval,
    ru_stime: Timeval,
    // Other fields omitted as they are not used
    // This avoids having to match the exact layout
}

pub struct ProcessTime;

impl ProcessTime {
    pub fn new() -> Self {
        ProcessTime
    }

    #[cfg(unix)]
    pub fn get_process_time(&self) -> Duration {
        use std::mem::zeroed;

        #[cfg(target_os = "macos")]
        use libc::{RUSAGE_SELF, getrusage};

        #[cfg(not(target_os = "macos"))]
        use libc::{RUSAGE_SELF, getrusage, CLOCK_PROCESS_CPUTIME_ID, clock_gettime};

        unsafe {
            // Try using getrusage first
            let mut r_usage: Rusage = zeroed();
            let result = getrusage(RUSAGE_SELF, &mut r_usage as *mut Rusage as *mut _);
            
            if result == 0 {
                let user_sec = r_usage.ru_utime.tv_sec as u64;
                let user_usec = r_usage.ru_utime.tv_usec as u32;
                let sys_sec = r_usage.ru_stime.tv_sec as u64;
                let sys_usec = r_usage.ru_stime.tv_usec as u32;
                
                let user = Duration::new(user_sec, user_usec * 1000);
                let sys = Duration::new(sys_sec, sys_usec * 1000);
                
                user + sys
            } else {
                // Fallback to clock_gettime if available (not on macOS)
                #[cfg(not(target_os = "macos"))]
                {
                    let mut time_spec: timespec = zeroed();
                    let result = clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &mut time_spec);
                    
                    if result == 0 {
                        Duration::new(time_spec.tv_sec as u64, time_spec.tv_nsec as u32)
                    } else {
                        // If all else fails, return a zero duration
                        Duration::new(0, 0)
                    }
                }
                #[cfg(target_os = "macos")]
                {
                    // On macOS, just return a zero duration if getrusage fails
                    Duration::new(0, 0)
                }
            }
        }
    }

    #[cfg(windows)]
    pub fn get_process_time(&self) -> Duration {
        use std::mem::zeroed;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        use windows_sys::Win32::System::Threading::GetProcessTimes;
        use windows_sys::Win32::Foundation::FILETIME;
        
        unsafe {
            let h_process = GetCurrentProcess();
            let mut creation_time: FILETIME = zeroed();
            let mut exit_time: FILETIME = zeroed();
            let mut kernel_time: FILETIME = zeroed();
            let mut user_time: FILETIME = zeroed();
            
            let result = GetProcessTimes(
                h_process,
                &mut creation_time,
                &mut exit_time,
                &mut kernel_time,
                &mut user_time,
            );
            
            if result != 0 {
                // Convert FILETIME to Duration (100-nanosecond intervals)
                let user = filetime_to_duration(&user_time);
                let kernel = filetime_to_duration(&kernel_time);
                
                user + kernel
            } else {
                Duration::new(0, 0)
            }
        }
    }
}

#[cfg(windows)]
fn filetime_to_duration(ft: &windows_sys::Win32::Foundation::FILETIME) -> Duration {
    // FILETIME is in 100-nanosecond intervals
    // Combine the high and low parts to get the total 100-nanosecond intervals
    let total_intervals = ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64);
    
    // Convert to seconds and nanoseconds
    let seconds = total_intervals / 10_000_000;
    let nanoseconds = ((total_intervals % 10_000_000) * 100) as u32;
    
    Duration::new(seconds, nanoseconds)
}

impl Measurement for ProcessTime {
    type Intermediate = Duration;
    type Value = Duration;

    fn start(&self) -> Self::Intermediate {
        self.get_process_time()
    }

    fn end(&self, start: Self::Intermediate) -> Self::Value {
        self.get_process_time() - start
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        *v1 + *v2
    }

    fn zero(&self) -> Self::Value {
        Duration::from_nanos(0)
    }

    fn to_f64(&self, val: &Self::Value) -> f64 {
        val.as_secs_f64() * 1e9 // convert to nanoseconds
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        &crate::formatter::DurationFormatter
    }
}