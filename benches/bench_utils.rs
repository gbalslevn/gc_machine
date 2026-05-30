use stats_alloc::{Region, StatsAlloc};
use std::alloc::{System};
use std::collections::HashMap;
use stats_alloc::Stats;
use serde::{Serialize, Deserialize};

// maybe use this to run benching, cargo bench -- --test-threads=1

// Measures amount of heap memory used for the provided action
pub fn get_memory<F, R>(function: F, global: &StatsAlloc<System>) -> (R, Stats)
where
    F: FnOnce() -> R,
{
    let reg = Region::new(&global);
    let result = function();
    let stats = reg.change();
    (result, stats)
}

// ------------------------------------------------------------------
// Instruction counter — real impl on Linux, no-op stub elsewhere
// ------------------------------------------------------------------

#[cfg(target_os = "linux")]
pub mod perf {
    use std::fs::File;
    use std::io;
    use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

    #[repr(C)]
    struct PerfEventAttr {
        kind: u32, config: u64, size: u32,
        sample_period: u64, sample_type: u64,
        read_format: u64, flags: u64,
        _rest: [u64; 16],
    }
    impl PerfEventAttr {
        fn instructions() -> Self {
            let mut a: Self = unsafe { std::mem::zeroed() };
            a.kind   = 0; // PERF_TYPE_HARDWARE
            a.size   = std::mem::size_of::<Self>() as u32;
            a.config = 1; // PERF_COUNT_HW_INSTRUCTIONS
            a
        }
    }

    const IOC_RESET:   u64 = 0x2403;
    const IOC_ENABLE:  u64 = 0x2400;
    const IOC_DISABLE: u64 = 0x2401;

    pub struct Counter(File);

    impl Counter {
        pub fn new() -> io::Result<Self> {
            let attr = PerfEventAttr::instructions();
            let fd = unsafe {
                libc::syscall(libc::SYS_perf_event_open,
                    &attr as *const _, 0i32, -1i32, -1i32, 0u64)
            };
            if fd < 0 { return Err(io::Error::last_os_error()); }
            Ok(Self(unsafe { File::from_raw_fd(fd as RawFd) }))
        }

        pub fn measure<F, R>(&self, f: F) -> io::Result<(R, u64)>
        where F: FnOnce() -> R {
            let fd = self.0.as_raw_fd();
            unsafe {
                libc::ioctl(fd, IOC_RESET,   0);
                libc::ioctl(fd, IOC_ENABLE,  0);
            }
            let result = f();
            unsafe { libc::ioctl(fd, IOC_DISABLE, 0); }

            let mut count = 0u64;
            let n = unsafe {
                libc::read(fd, &mut count as *mut u64 as *mut _, 8)
            };
            if n != 8 { return Err(io::Error::last_os_error()); }
            Ok((result, count))
        }
    }
}

// Instruction counter which enables us to only benchmark instruction count on Linux. Only created successfully when running Linux. 
pub struct InsnCounter {
    #[cfg(target_os = "linux")]
    inner: Option<perf::Counter>,
}

impl InsnCounter {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "linux")]
            inner: {
                let result = perf::Counter::new();
                if let Err(ref e) = result {
                    println!("perf_event_open failed: {e}");
                }
                result.ok()
            },
        }
    }

    /// Returns instruction count on Linux, None elsewhere.Thin wrapper so call sites don't need cfg blocks everywhere
    pub fn measure<F, R>(&self, f: F) -> (R, Option<u64>)
    where F: FnOnce() -> R {
        #[cfg(target_os = "linux")]
        if let Some(ref c) = self.inner {
            let (r, n) = c.measure(f).expect("perf read failed");
            return (r, Some(n));
        }
        (f(), None)
    }
}


#[derive(Serialize, Deserialize, Default)]
struct BenchMetrics {
    protocol_bytes: usize,
    garble_bytes_allocated: usize,
    eval_bytes_allocated: usize,
    garble_instructions: Option<u64>,
    eval_instructions: Option<u64>,
}

// Write metrics to a file
pub fn write_bench_metrics(name: &str, protocol_bytes: usize, garble: &Stats, eval: &Stats, garble_instructions: Option<u64>, eval_instructions: Option<u64>) {
    let path = std::path::Path::new("target/criterion/bench_metrics.json");
    let mut map: HashMap<String, BenchMetrics> = path.exists()
        .then(|| serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap_or_default())
        .unwrap_or_default();

    map.insert(name.to_string(), BenchMetrics {
        protocol_bytes,
        garble_bytes_allocated: garble.bytes_allocated,
        eval_bytes_allocated: eval.bytes_allocated,
        garble_instructions,
        eval_instructions
    });

    std::fs::create_dir_all("target/criterion").unwrap();
    std::fs::write(path, serde_json::to_string_pretty(&map).unwrap()).unwrap();
}