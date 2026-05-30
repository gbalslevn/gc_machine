// src/bench_perf.rs

#[cfg(target_os = "linux")]
mod inner {
    use std::fs::File;
    use std::io;
    use std::os::unix::io::{FromRawFd, RawFd};

    // Mirrors the kernel's perf_event_attr layout (simplified)
    #[repr(C)]
    struct PerfEventAttr {
        kind:         u32,  // PERF_TYPE_HARDWARE = 0
        size:         u32,
        config:       u64,  // PERF_COUNT_HW_INSTRUCTIONS = 1
        sample_period: u64,
        sample_type:  u64,
        read_format:  u64,
        flags:        u64,
        _rest:        [u64; 16],
    }

    impl PerfEventAttr {
        fn instructions() -> Self {
            let mut attr: Self = unsafe { std::mem::zeroed() };
            attr.kind   = 0; // PERF_TYPE_HARDWARE
            attr.size   = std::mem::size_of::<Self>() as u32;
            attr.config = 1; // PERF_COUNT_HW_INSTRUCTIONS
            attr
        }
    }

    fn perf_event_open(attr: &PerfEventAttr) -> io::Result<RawFd> {
        let fd = unsafe {
            libc::syscall(
                libc::SYS_perf_event_open,
                attr as *const _,
                0i32,   // pid = current process
                -1i32,  // cpu = any
                -1i32,  // group_fd = none
                0u64,   // flags
            )
        };
        if fd < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(fd as RawFd)
        }
    }

    const PERF_EVENT_IOC_RESET:  u64 = 0x2403;
    const PERF_EVENT_IOC_ENABLE: u64 = 0x2400;
    const PERF_EVENT_IOC_DISABLE:u64 = 0x2401;

    pub struct InstructionCounter {
        file: File,
    }

    impl InstructionCounter {
        pub fn new() -> io::Result<Self> {
            let attr = PerfEventAttr::instructions();
            let fd = perf_event_open(&attr)?;
            Ok(Self { file: unsafe { File::from_raw_fd(fd) } })
        }

        pub fn measure<F, R>(&self, f: F) -> io::Result<(R, u64)>
        where
            F: FnOnce() -> R,
        {
            use std::os::unix::io::AsRawFd;
            let fd = self.file.as_raw_fd();

            // Reset → enable → run → disable → read
            unsafe {
                libc::ioctl(fd, PERF_EVENT_IOC_RESET,   0);
                libc::ioctl(fd, PERF_EVENT_IOC_ENABLE,  0);
            }

            let result = f();

            unsafe {
                libc::ioctl(fd, PERF_EVENT_IOC_DISABLE, 0);
            }

            let mut count: u64 = 0;
            let n = unsafe {
                libc::read(
                    fd,
                    &mut count as *mut u64 as *mut libc::c_void,
                    std::mem::size_of::<u64>(),
                )
            };
            if n != std::mem::size_of::<u64>() as isize {
                return Err(io::Error::last_os_error());
            }

            Ok((result, count))
        }
    }
}

// Public re-export gated on Linux
#[cfg(target_os = "linux")]
pub use inner::InstructionCounter;