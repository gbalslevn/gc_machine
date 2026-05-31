// use std::fs::File;
// use std::io;
// use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

// #[repr(C)]
// struct PerfEventAttr {
//     kind:          u32,
//     size:          u32,
//     config:        u64,
//     sample_period: u64,
//     sample_type:   u64,
//     read_format:   u64,
//     flags:         u64,
//     _rest:         [u64; 16],
// }

// impl PerfEventAttr {
//     fn instructions() -> Self {
//         let mut a: Self = unsafe { std::mem::zeroed() };
//         a.kind   = 0; // PERF_TYPE_HARDWARE
//         a.size   = std::mem::size_of::<Self>() as u32;
//         a.config = 1; // PERF_COUNT_HW_INSTRUCTIONS
//         a
//     }
// }

// fn perf_event_open(attr: &PerfEventAttr) -> io::Result<RawFd> {
//     let fd = unsafe {
//         libc::syscall(
//             libc::SYS_perf_event_open,
//             attr as *const _,
//             0i32,   // pid = current process
//             -1i32,  // cpu = any
//             -1i32,  // group_fd = none
//             0u64,   // flags
//         )
//     };
//     if fd < 0 { Err(io::Error::last_os_error()) } else { Ok(fd as RawFd) }
// }

// const IOC_RESET:   u64 = 0x2403;
// const IOC_ENABLE:  u64 = 0x2400;
// const IOC_DISABLE: u64 = 0x2401;

// pub struct Counter(File);

// impl Counter {
//     pub fn new() -> io::Result<Self> {
//         let attr = PerfEventAttr::instructions();
//         let fd = perf_event_open(&attr)?;
//         Ok(Self(unsafe { File::from_raw_fd(fd) }))
//     }

//     pub fn measure<F, R>(&self, f: F) -> io::Result<(R, u64)>
//     where F: FnOnce() -> R {
//         let fd = self.0.as_raw_fd();
//         unsafe {
//             libc::ioctl(fd, IOC_RESET,   0);
//             libc::ioctl(fd, IOC_ENABLE,  0);
//         }
//         let result = f();
//         unsafe { libc::ioctl(fd, IOC_DISABLE, 0); }

//         let mut count = 0u64;
//         let n = unsafe {
//             libc::read(fd, &mut count as *mut u64 as *mut libc::c_void, 8)
//         };
//         if n != 8 { return Err(io::Error::last_os_error()); }
//         Ok((result, count))
//     }
// }