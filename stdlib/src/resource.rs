// spell-checker:disable

pub(crate) use resource::make_module;

#[pymodule]
mod resource {
    use crate::vm::{
        PyObject, PyObjectRef, PyResult, TryFromBorrowedObject, VirtualMachine,
        convert::{ToPyException, ToPyObject},
        stdlib::os,
        types::PyStructSequence,
    };
    use std::{io, mem};

    cfg_if::cfg_if! {
        if #[cfg(target_os = "android")] {
            #[expect(deprecated)]
            const RLIM_NLIMITS: i32 = libc::RLIM_NLIMITS;
        } else {
            // This constant isn't abi-stable across os versions, so we just
            // pick a high number so we don't get false positive ValueErrors and just bubble up the
            // EINVAL that get/setrlimit return on an invalid resource
            const RLIM_NLIMITS: i32 = 256;
        }
    }

    // TODO: RLIMIT_OFILE,
    #[pyattr]
    use libc::{
        RLIM_INFINITY, RLIMIT_AS, RLIMIT_CORE, RLIMIT_CPU, RLIMIT_DATA, RLIMIT_FSIZE,
        RLIMIT_MEMLOCK, RLIMIT_NOFILE, RLIMIT_NPROC, RLIMIT_RSS, RLIMIT_STACK,
    };

    #[cfg(any(target_os = "linux", target_os = "android", target_os = "emscripten"))]
    #[pyattr]
    use libc::{RLIMIT_MSGQUEUE, RLIMIT_NICE, RLIMIT_RTPRIO, RLIMIT_SIGPENDING};
    // TODO: I think this is supposed to be defined for all linux_like?
    #[cfg(target_os = "linux")]
    #[pyattr]
    use libc::RLIMIT_RTTIME;

    #[cfg(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "solaris",
        target_os = "illumos"
    ))]
    #[pyattr]
    use libc::RLIMIT_SBSIZE;

    #[cfg(any(target_os = "freebsd", target_os = "solaris", target_os = "illumos"))]
    #[pyattr]
    use libc::{RLIMIT_NPTS, RLIMIT_SWAP};

    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    #[pyattr]
    use libc::RLIMIT_VMEM;

    #[cfg(any(target_os = "linux", target_os = "emscripten", target_os = "freebsd"))]
    #[pyattr]
    use libc::RUSAGE_THREAD;
    #[cfg(not(any(target_os = "windows", target_os = "redox")))]
    #[pyattr]
    use libc::{RUSAGE_CHILDREN, RUSAGE_SELF};

    #[pyattr]
    #[pyclass(name = "struct_rusage")]
    #[derive(PyStructSequence)]
    struct Rusage {
        ru_utime: f64,
        ru_stime: f64,
        ru_maxrss: libc::c_long,
        ru_ixrss: libc::c_long,
        ru_idrss: libc::c_long,
        ru_isrss: libc::c_long,
        ru_minflt: libc::c_long,
        ru_majflt: libc::c_long,
        ru_nswap: libc::c_long,
        ru_inblock: libc::c_long,
        ru_oublock: libc::c_long,
        ru_msgsnd: libc::c_long,
        ru_msgrcv: libc::c_long,
        ru_nsignals: libc::c_long,
        ru_nvcsw: libc::c_long,
        ru_nivcsw: libc::c_long,
    }

    #[pyclass(with(PyStructSequence))]
    impl Rusage {}

    impl From<libc::rusage> for Rusage {
        fn from(rusage: libc::rusage) -> Self {
            let tv = |tv: libc::timeval| tv.tv_sec as f64 + (tv.tv_usec as f64 / 1_000_000.0);
            Self {
                ru_utime: tv(rusage.ru_utime),
                ru_stime: tv(rusage.ru_stime),
                ru_maxrss: rusage.ru_maxrss,
                ru_ixrss: rusage.ru_ixrss,
                ru_idrss: rusage.ru_idrss,
                ru_isrss: rusage.ru_isrss,
                ru_minflt: rusage.ru_minflt,
                ru_majflt: rusage.ru_majflt,
                ru_nswap: rusage.ru_nswap,
                ru_inblock: rusage.ru_inblock,
                ru_oublock: rusage.ru_oublock,
                ru_msgsnd: rusage.ru_msgsnd,
                ru_msgrcv: rusage.ru_msgrcv,
                ru_nsignals: rusage.ru_nsignals,
                ru_nvcsw: rusage.ru_nvcsw,
                ru_nivcsw: rusage.ru_nivcsw,
            }
        }
    }

    #[pyfunction]
    fn getrusage(who: i32, vm: &VirtualMachine) -> PyResult<Rusage> {
        let res = unsafe {
            let mut rusage = mem::MaybeUninit::<libc::rusage>::uninit();
            if libc::getrusage(who, rusage.as_mut_ptr()) == -1 {
                Err(io::Error::last_os_error())
            } else {
                Ok(rusage.assume_init())
            }
        };
        res.map(Rusage::from).map_err(|e| {
            if e.kind() == io::ErrorKind::InvalidInput {
                vm.new_value_error("invalid who parameter")
            } else {
                e.to_pyexception(vm)
            }
        })
    }

    struct Limits(libc::rlimit);
    impl<'a> TryFromBorrowedObject<'a> for Limits {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &'a PyObject) -> PyResult<Self> {
            let seq: Vec<libc::rlim_t> = obj.try_to_value(vm)?;
            match *seq {
                [cur, max] => Ok(Self(libc::rlimit {
                    rlim_cur: cur & RLIM_INFINITY,
                    rlim_max: max & RLIM_INFINITY,
                })),
                _ => Err(vm.new_value_error("expected a tuple of 2 integers")),
            }
        }
    }
    impl ToPyObject for Limits {
        fn to_pyobject(self, vm: &VirtualMachine) -> PyObjectRef {
            (self.0.rlim_cur, self.0.rlim_max).to_pyobject(vm)
        }
    }

    #[pyfunction]
    fn getrlimit(resource: i32, vm: &VirtualMachine) -> PyResult<Limits> {
        #[allow(clippy::unnecessary_cast)]
        if resource < 0 || resource >= RLIM_NLIMITS as i32 {
            return Err(vm.new_value_error("invalid resource specified"));
        }
        let rlimit = unsafe {
            let mut rlimit = mem::MaybeUninit::<libc::rlimit>::uninit();
            if libc::getrlimit(resource as _, rlimit.as_mut_ptr()) == -1 {
                return Err(os::errno_err(vm));
            }
            rlimit.assume_init()
        };
        Ok(Limits(rlimit))
    }

    #[pyfunction]
    fn setrlimit(resource: i32, limits: Limits, vm: &VirtualMachine) -> PyResult<()> {
        #[allow(clippy::unnecessary_cast)]
        if resource < 0 || resource >= RLIM_NLIMITS as i32 {
            return Err(vm.new_value_error("invalid resource specified"));
        }
        let res = unsafe {
            if libc::setrlimit(resource as _, &limits.0) == -1 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        };
        res.map_err(|e| match e.kind() {
            io::ErrorKind::InvalidInput => {
                vm.new_value_error("current limit exceeds maximum limit")
            }
            io::ErrorKind::PermissionDenied => {
                vm.new_value_error("not allowed to raise maximum limit")
            }
            _ => e.to_pyexception(vm),
        })
    }
}
