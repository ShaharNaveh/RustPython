/*
 * I/O core tools.
 */
cfg_if::cfg_if! {
    if #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))] {
        use crate::common::crt_fd::Offset;
    } else {
        type Offset = i64;
    }
}

use crate::{
    PyObjectRef, PyRef, PyResult, TryFromObject, VirtualMachine,
    builtins::{PyBaseExceptionRef, PyModule},
    common::os::ErrorExt,
    convert::{IntoPyException, ToPyException},
};
pub use _io::{OpenArgs, io_open as open};

impl ToPyException for std::io::Error {
    fn to_pyexception(&self, vm: &VirtualMachine) -> PyBaseExceptionRef {
        let errno = self.posix_errno();
        let msg = self.to_string();
        #[allow(clippy::let_and_return)]
        let exc = vm.new_errno_error(errno, msg);

        #[cfg(windows)]
        {
            use crate::object::AsObject;
            let winerror = if let Some(winerror) = self.raw_os_error() {
                vm.new_pyobj(winerror)
            } else {
                vm.ctx.none()
            };

            // FIXME: manual setup winerror due to lack of OSError.__init__ support
            exc.as_object()
                .set_attr("winerror", vm.new_pyobj(winerror), vm)
                .unwrap();
        }
        exc
    }
}

impl IntoPyException for std::io::Error {
    fn into_pyexception(self, vm: &VirtualMachine) -> PyBaseExceptionRef {
        self.to_pyexception(vm)
    }
}

pub(crate) fn make_module(vm: &VirtualMachine) -> PyRef<PyModule> {
    let ctx = &vm.ctx;

    let module = _io::make_module(vm);

    #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
    fileio::extend_module(vm, &module).unwrap();

    let unsupported_operation = _io::UNSUPPORTED_OPERATION
        .get_or_init(|| _io::make_unsupportedop(ctx))
        .clone();
    extend_module!(vm, &module, {
        "UnsupportedOperation" => unsupported_operation,
        "BlockingIOError" => ctx.exceptions.blocking_io_error.to_owned(),
    });

    module
}

// not used on all platforms
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Fildes(pub i32);

impl TryFromObject for Fildes {
    fn try_from_object(vm: &VirtualMachine, obj: PyObjectRef) -> PyResult<Self> {
        use crate::builtins::int;
        let int = match obj.downcast::<int::PyInt>() {
            Ok(i) => i,
            Err(obj) => {
                let fileno_meth = vm.get_attribute_opt(obj, "fileno")?.ok_or_else(|| {
                    vm.new_type_error("argument must be an int, or have a fileno() method.")
                })?;
                fileno_meth
                    .call((), vm)?
                    .downcast()
                    .map_err(|_| vm.new_type_error("fileno() returned a non-integer"))?
            }
        };
        let fd = int.try_to_primitive(vm)?;
        if fd < 0 {
            return Err(vm.new_value_error(format!(
                "file descriptor cannot be a negative integer ({fd})"
            )));
        }
        Ok(Self(fd))
    }
}

#[cfg(unix)]
impl std::os::fd::AsFd for Fildes {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        // SAFETY: none, really. but, python's os api of passing around file descriptors
        //         everywhere isn't really io-safe anyway, so, this is passed to the user.
        unsafe { std::os::fd::BorrowedFd::borrow_raw(self.0) }
    }
}
#[cfg(unix)]
impl std::os::fd::AsRawFd for Fildes {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.0
    }
}

#[pymodule]
mod _io {
    use super::*;
    use crate::{
        AsObject, Context, Py, PyObject, PyObjectRef, PyPayload, PyRef, PyResult,
        TryFromBorrowedObject, TryFromObject,
        builtins::{
            PyBaseExceptionRef, PyByteArray, PyBytes, PyBytesRef, PyIntRef, PyMemoryView, PyStr,
            PyStrRef, PyTuple, PyTupleRef, PyType, PyTypeRef, PyUtf8StrRef,
        },
        class::StaticType,
        common::lock::{
            PyMappedThreadMutexGuard, PyRwLock, PyRwLockReadGuard, PyRwLockWriteGuard,
            PyThreadMutex, PyThreadMutexGuard,
        },
        common::wtf8::{Wtf8, Wtf8Buf},
        convert::ToPyObject,
        function::{
            ArgBytesLike, ArgIterable, ArgMemoryBuffer, ArgSize, Either, FuncArgs, IntoFuncArgs,
            OptionalArg, OptionalOption, PySetterValue,
        },
        protocol::{
            BufferDescriptor, BufferMethods, BufferResizeGuard, PyBuffer, PyIterReturn, VecBuffer,
        },
        recursion::ReprGuard,
        types::{
            Callable, Constructor, DefaultConstructor, Destructor, Initializer, IterNext, Iterable,
        },
        vm::VirtualMachine,
    };
    use bstr::ByteSlice;
    use crossbeam_utils::atomic::AtomicCell;
    use malachite_bigint::{BigInt, BigUint};
    use num_traits::ToPrimitive;
    use std::{
        borrow::Cow,
        io::{self, Cursor, SeekFrom, prelude::*},
        ops::Range,
    };

    #[allow(clippy::let_and_return)]
    fn validate_whence(whence: i32) -> bool {
        let x = (0..=2).contains(&whence);
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "linux"))] {
                x || matches!(whence, libc::SEEK_DATA | libc::SEEK_HOLE)
            } else {
                x
            }
        }
    }

    fn ensure_unclosed(file: &PyObject, msg: &str, vm: &VirtualMachine) -> PyResult<()> {
        if file.get_attr("closed", vm)?.try_to_bool(vm)? {
            Err(vm.new_value_error(msg))
        } else {
            Ok(())
        }
    }

    pub fn new_unsupported_operation(vm: &VirtualMachine, msg: String) -> PyBaseExceptionRef {
        vm.new_exception_msg(UNSUPPORTED_OPERATION.get().unwrap().clone(), msg)
    }

    fn _unsupported<T>(vm: &VirtualMachine, zelf: &PyObject, operation: &str) -> PyResult<T> {
        Err(new_unsupported_operation(
            vm,
            format!("{}.{}() not supported", zelf.class().name(), operation),
        ))
    }

    #[derive(FromArgs)]
    pub(super) struct OptionalSize {
        // In a few functions, the default value is -1 rather than None.
        // Make sure the default value doesn't affect compatibility.
        #[pyarg(positional, default)]
        size: Option<ArgSize>,
    }

    impl OptionalSize {
        #[allow(clippy::wrong_self_convention)]
        pub fn to_usize(self) -> Option<usize> {
            self.size?.to_usize()
        }

        pub fn try_usize(self, vm: &VirtualMachine) -> PyResult<Option<usize>> {
            self.size
                .map(|v| {
                    let v = *v;
                    if v >= 0 {
                        Ok(v as usize)
                    } else {
                        Err(vm.new_value_error(format!("Negative size value {v}")))
                    }
                })
                .transpose()
        }
    }

    fn os_err(vm: &VirtualMachine, err: io::Error) -> PyBaseExceptionRef {
        #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
        {
            use crate::convert::ToPyException;
            err.to_pyexception(vm)
        }
        #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
        {
            vm.new_os_error(err.to_string())
        }
    }

    pub(super) fn io_closed_error(vm: &VirtualMachine) -> PyBaseExceptionRef {
        vm.new_value_error("I/O operation on closed file")
    }

    #[pyattr]
    const DEFAULT_BUFFER_SIZE: usize = 8 * 1024;

    pub(super) fn seekfrom(
        vm: &VirtualMachine,
        offset: PyObjectRef,
        how: OptionalArg<i32>,
    ) -> PyResult<SeekFrom> {
        let seek = match how {
            OptionalArg::Present(0) | OptionalArg::Missing => {
                SeekFrom::Start(offset.try_into_value(vm)?)
            }
            OptionalArg::Present(1) => SeekFrom::Current(offset.try_into_value(vm)?),
            OptionalArg::Present(2) => SeekFrom::End(offset.try_into_value(vm)?),
            _ => return Err(vm.new_value_error("invalid value for how")),
        };
        Ok(seek)
    }

    #[derive(Debug)]
    struct BufferedIO {
        cursor: Cursor<Vec<u8>>,
    }

    impl BufferedIO {
        const fn new(cursor: Cursor<Vec<u8>>) -> Self {
            Self { cursor }
        }

        fn write(&mut self, data: &[u8]) -> Option<u64> {
            if data.is_empty() {
                return Some(0);
            }
            let length = data.len();
            self.cursor.write_all(data).ok()?;
            Some(length as u64)
        }

        //return the entire contents of the underlying
        fn getvalue(&self) -> Vec<u8> {
            self.cursor.clone().into_inner()
        }

        //skip to the jth position
        fn seek(&mut self, seek: SeekFrom) -> io::Result<u64> {
            self.cursor.seek(seek)
        }

        //Read k bytes from the object and return.
        fn read(&mut self, bytes: Option<usize>) -> Option<Vec<u8>> {
            let pos = self.cursor.position().to_usize()?;
            let avail_slice = self.cursor.get_ref().get(pos..)?;
            // if we don't specify the number of bytes, or it's too big, give the whole rest of the slice
            let n = bytes.map_or_else(
                || avail_slice.len(),
                |n| std::cmp::min(n, avail_slice.len()),
            );
            let b = avail_slice[..n].to_vec();
            self.cursor.set_position((pos + n) as u64);
            Some(b)
        }

        const fn tell(&self) -> u64 {
            self.cursor.position()
        }

        fn readline(&mut self, size: Option<usize>, vm: &VirtualMachine) -> PyResult<Vec<u8>> {
            self.read_until(size, b'\n', vm)
        }

        fn read_until(
            &mut self,
            size: Option<usize>,
            byte: u8,
            vm: &VirtualMachine,
        ) -> PyResult<Vec<u8>> {
            let size = match size {
                None => {
                    let mut buf: Vec<u8> = Vec::new();
                    self.cursor
                        .read_until(byte, &mut buf)
                        .map_err(|err| os_err(vm, err))?;
                    return Ok(buf);
                }
                Some(0) => {
                    return Ok(Vec::new());
                }
                Some(size) => size,
            };

            let available = {
                // For Cursor, fill_buf returns all of the remaining data unlike other BufReads which have outer reading source.
                // Unless we add other data by write, there will be no more data.
                let buf = self.cursor.fill_buf().map_err(|err| os_err(vm, err))?;
                if size < buf.len() { &buf[..size] } else { buf }
            };
            let buf = match available.find_byte(byte) {
                Some(i) => available[..=i].to_vec(),
                _ => available.to_vec(),
            };
            self.cursor.consume(buf.len());
            Ok(buf)
        }

        fn truncate(&mut self, pos: Option<usize>) -> usize {
            let pos = pos.unwrap_or_else(|| self.tell() as usize);
            self.cursor.get_mut().truncate(pos);
            pos
        }
    }

    fn file_closed(file: &PyObject, vm: &VirtualMachine) -> PyResult<bool> {
        file.get_attr("closed", vm)?.try_to_bool(vm)
    }

    fn check_closed(file: &PyObject, vm: &VirtualMachine) -> PyResult<()> {
        if file_closed(file, vm)? {
            Err(io_closed_error(vm))
        } else {
            Ok(())
        }
    }

    fn check_readable(file: &PyObject, vm: &VirtualMachine) -> PyResult<()> {
        if vm.call_method(file, "readable", ())?.try_to_bool(vm)? {
            Ok(())
        } else {
            Err(new_unsupported_operation(
                vm,
                "File or stream is not readable".to_owned(),
            ))
        }
    }

    fn check_writable(file: &PyObject, vm: &VirtualMachine) -> PyResult<()> {
        if vm.call_method(file, "writable", ())?.try_to_bool(vm)? {
            Ok(())
        } else {
            Err(new_unsupported_operation(
                vm,
                "File or stream is not writable.".to_owned(),
            ))
        }
    }

    fn check_seekable(file: &PyObject, vm: &VirtualMachine) -> PyResult<()> {
        if vm.call_method(file, "seekable", ())?.try_to_bool(vm)? {
            Ok(())
        } else {
            Err(new_unsupported_operation(
                vm,
                "File or stream is not seekable".to_owned(),
            ))
        }
    }

    fn check_decoded(decoded: PyObjectRef, vm: &VirtualMachine) -> PyResult<PyStrRef> {
        decoded.downcast().map_err(|obj| {
            vm.new_type_error(format!(
                "decoder should return a string result, not '{}'",
                obj.class().name()
            ))
        })
    }

    #[pyattr]
    #[pyclass(name = "_IOBase")]
    #[derive(Debug, PyPayload)]
    pub struct _IOBase;

    #[pyclass(with(IterNext, Iterable, Destructor), flags(BASETYPE, HAS_DICT))]
    impl _IOBase {
        #[pymethod]
        fn seek(
            zelf: PyObjectRef,
            _pos: PyObjectRef,
            _whence: OptionalArg,
            vm: &VirtualMachine,
        ) -> PyResult {
            _unsupported(vm, &zelf, "seek")
        }

        #[pymethod]
        fn tell(zelf: PyObjectRef, vm: &VirtualMachine) -> PyResult {
            vm.call_method(&zelf, "seek", (0, 1))
        }

        #[pymethod]
        fn truncate(zelf: PyObjectRef, _pos: OptionalArg, vm: &VirtualMachine) -> PyResult {
            _unsupported(vm, &zelf, "truncate")
        }

        #[pymethod]
        fn fileno(zelf: PyObjectRef, vm: &VirtualMachine) -> PyResult {
            _unsupported(vm, &zelf, "truncate")
        }

        #[pyattr]
        fn __closed(ctx: &Context) -> PyIntRef {
            ctx.new_bool(false)
        }

        #[pymethod]
        fn __enter__(instance: PyObjectRef, vm: &VirtualMachine) -> PyResult {
            check_closed(&instance, vm)?;
            Ok(instance)
        }

        #[pymethod]
        fn __exit__(instance: PyObjectRef, _args: FuncArgs, vm: &VirtualMachine) -> PyResult<()> {
            vm.call_method(&instance, "close", ())?;
            Ok(())
        }

        #[pymethod]
        fn flush(instance: PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
            // just check if this is closed; if it isn't, do nothing
            check_closed(&instance, vm)
        }

        #[pymethod]
        fn seekable(_self: PyObjectRef) -> bool {
            false
        }

        #[pymethod]
        fn readable(_self: PyObjectRef) -> bool {
            false
        }

        #[pymethod]
        fn writable(_self: PyObjectRef) -> bool {
            false
        }

        #[pymethod]
        fn isatty(_self: PyObjectRef) -> bool {
            false
        }

        #[pygetset]
        fn closed(instance: PyObjectRef, vm: &VirtualMachine) -> PyResult {
            instance.get_attr("__closed", vm)
        }

        #[pymethod]
        fn close(instance: PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
            iobase_close(&instance, vm)
        }

        #[pymethod]
        fn readline(
            instance: PyObjectRef,
            size: OptionalSize,
            vm: &VirtualMachine,
        ) -> PyResult<Vec<u8>> {
            let size = size.to_usize();
            let read = instance.get_attr("read", vm)?;
            let mut res = Vec::new();
            while size.is_none_or(|s| res.len() < s) {
                let read_res = ArgBytesLike::try_from_object(vm, read.call((1,), vm)?)?;
                if read_res.with_ref(|b| b.is_empty()) {
                    break;
                }
                read_res.with_ref(|b| res.extend_from_slice(b));
                if res.ends_with(b"\n") {
                    break;
                }
            }
            Ok(res)
        }

        #[pymethod]
        fn readlines(
            instance: PyObjectRef,
            hint: OptionalOption<isize>,
            vm: &VirtualMachine,
        ) -> PyResult<Vec<PyObjectRef>> {
            let hint = hint.flatten().unwrap_or(-1);
            if hint <= 0 {
                return instance.try_to_value(vm);
            }
            let hint = hint as usize;
            let mut ret = Vec::new();
            let it = ArgIterable::<PyObjectRef>::try_from_object(vm, instance)?;
            let mut full_len = 0;
            for line in it.iter(vm)? {
                let line = line?;
                let line_len = line.length(vm)?;
                ret.push(line.clone());
                full_len += line_len;
                if full_len > hint {
                    break;
                }
            }
            Ok(ret)
        }

        #[pymethod]
        fn writelines(
            instance: PyObjectRef,
            lines: ArgIterable,
            vm: &VirtualMachine,
        ) -> PyResult<()> {
            check_closed(&instance, vm)?;
            for line in lines.iter(vm)? {
                vm.call_method(&instance, "write", (line?,))?;
            }
            Ok(())
        }

        #[pymethod(name = "_checkClosed")]
        fn check_closed(instance: PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
            check_closed(&instance, vm)
        }

        #[pymethod(name = "_checkReadable")]
        fn check_readable(instance: PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
            check_readable(&instance, vm)
        }

        #[pymethod(name = "_checkWritable")]
        fn check_writable(instance: PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
            check_writable(&instance, vm)
        }

        #[pymethod(name = "_checkSeekable")]
        fn check_seekable(instance: PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
            check_seekable(&instance, vm)
        }
    }

    impl Destructor for _IOBase {
        fn slot_del(zelf: &PyObject, vm: &VirtualMachine) -> PyResult<()> {
            let _ = vm.call_method(zelf, "close", ());
            Ok(())
        }

        #[cold]
        fn del(_zelf: &Py<Self>, _vm: &VirtualMachine) -> PyResult<()> {
            unreachable!("slot_del is implemented")
        }
    }

    impl Iterable for _IOBase {
        fn slot_iter(zelf: PyObjectRef, vm: &VirtualMachine) -> PyResult {
            check_closed(&zelf, vm)?;
            Ok(zelf)
        }

        fn iter(_zelf: PyRef<Self>, _vm: &VirtualMachine) -> PyResult {
            unreachable!("slot_iter is implemented")
        }
    }

    impl IterNext for _IOBase {
        fn slot_iternext(zelf: &PyObject, vm: &VirtualMachine) -> PyResult<PyIterReturn> {
            let line = vm.call_method(zelf, "readline", ())?;
            Ok(if !line.clone().try_to_bool(vm)? {
                PyIterReturn::StopIteration(None)
            } else {
                PyIterReturn::Return(line)
            })
        }

        fn next(_zelf: &Py<Self>, _vm: &VirtualMachine) -> PyResult<PyIterReturn> {
            unreachable!("slot_iternext is implemented")
        }
    }

    pub(super) fn iobase_close(file: &PyObject, vm: &VirtualMachine) -> PyResult<()> {
        if !file_closed(file, vm)? {
            let res = vm.call_method(file, "flush", ());
            file.set_attr("__closed", vm.new_pyobj(true), vm)?;
            res?;
        }
        Ok(())
    }

    #[pyattr]
    #[pyclass(name = "_RawIOBase", base = "_IOBase")]
    pub(super) struct _RawIOBase;

    #[pyclass(flags(BASETYPE, HAS_DICT))]
    impl _RawIOBase {
        #[pymethod]
        fn read(instance: PyObjectRef, size: OptionalSize, vm: &VirtualMachine) -> PyResult {
            if let Some(size) = size.to_usize() {
                // FIXME: unnecessary zero-init
                let b = PyByteArray::from(vec![0; size]).into_ref(&vm.ctx);
                let n = <Option<usize>>::try_from_object(
                    vm,
                    vm.call_method(&instance, "readinto", (b.clone(),))?,
                )?;
                Ok(n.map(|n| {
                    let mut bytes = b.borrow_buf_mut();
                    bytes.truncate(n);
                    // FIXME: try to use Arc::unwrap on the bytearray to get at the inner buffer
                    bytes.clone()
                })
                .to_pyobject(vm))
            } else {
                vm.call_method(&instance, "readall", ())
            }
        }

        #[pymethod]
        fn readall(instance: PyObjectRef, vm: &VirtualMachine) -> PyResult<Option<Vec<u8>>> {
            let mut chunks = Vec::new();
            let mut total_len = 0;
            loop {
                let data = vm.call_method(&instance, "read", (DEFAULT_BUFFER_SIZE,))?;
                let data = <Option<PyBytesRef>>::try_from_object(vm, data)?;
                match data {
                    None => {
                        if chunks.is_empty() {
                            return Ok(None);
                        }
                        break;
                    }
                    Some(b) => {
                        if b.as_bytes().is_empty() {
                            break;
                        }
                        total_len += b.as_bytes().len();
                        chunks.push(b)
                    }
                }
            }
            let mut ret = Vec::with_capacity(total_len);
            for b in chunks {
                ret.extend_from_slice(b.as_bytes())
            }
            Ok(Some(ret))
        }
    }

    #[pyattr]
    #[pyclass(name = "_BufferedIOBase", base = "_IOBase")]
    struct _BufferedIOBase;

    #[pyclass(flags(BASETYPE))]
    impl _BufferedIOBase {
        #[pymethod]
        fn read(zelf: PyObjectRef, _size: OptionalArg, vm: &VirtualMachine) -> PyResult {
            _unsupported(vm, &zelf, "read")
        }

        #[pymethod]
        fn read1(zelf: PyObjectRef, _size: OptionalArg, vm: &VirtualMachine) -> PyResult {
            _unsupported(vm, &zelf, "read1")
        }

        fn _readinto(
            zelf: PyObjectRef,
            buf_obj: PyObjectRef,
            method: &str,
            vm: &VirtualMachine,
        ) -> PyResult<usize> {
            let b = ArgMemoryBuffer::try_from_borrowed_object(vm, &buf_obj)?;
            let l = b.len();
            let data = vm.call_method(&zelf, method, (l,))?;
            if data.is(&buf_obj) {
                return Ok(l);
            }
            let mut buf = b.borrow_buf_mut();
            let data = ArgBytesLike::try_from_object(vm, data)?;
            let data = data.borrow_buf();
            match buf.get_mut(..data.len()) {
                Some(slice) => {
                    slice.copy_from_slice(&data);
                    Ok(data.len())
                }
                None => {
                    Err(vm.new_value_error("readinto: buffer and read data have different lengths"))
                }
            }
        }
        #[pymethod]
        fn readinto(zelf: PyObjectRef, b: PyObjectRef, vm: &VirtualMachine) -> PyResult<usize> {
            Self::_readinto(zelf, b, "read", vm)
        }

        #[pymethod]
        fn readinto1(zelf: PyObjectRef, b: PyObjectRef, vm: &VirtualMachine) -> PyResult<usize> {
            Self::_readinto(zelf, b, "read1", vm)
        }

        #[pymethod]
        fn write(zelf: PyObjectRef, _b: PyObjectRef, vm: &VirtualMachine) -> PyResult {
            _unsupported(vm, &zelf, "write")
        }

        #[pymethod]
        fn detach(zelf: PyObjectRef, vm: &VirtualMachine) -> PyResult {
            _unsupported(vm, &zelf, "detach")
        }
    }

    // TextIO Base has no public constructor
    #[pyattr]
    #[pyclass(name = "_TextIOBase", base = "_IOBase")]
    #[derive(Debug, PyPayload)]
    struct _TextIOBase;

    #[pyclass(flags(BASETYPE))]
    impl _TextIOBase {
        #[pygetset]
        fn encoding(_zelf: PyObjectRef, vm: &VirtualMachine) -> PyObjectRef {
            vm.ctx.none()
        }

        #[pygetset]
        fn errors(_zelf: PyObjectRef, vm: &VirtualMachine) -> PyObjectRef {
            vm.ctx.none()
        }
    }

    #[derive(FromArgs, Clone)]
    struct BufferSize {
        #[pyarg(any, optional)]
        buffer_size: OptionalArg<isize>,
    }

    bitflags::bitflags! {
        #[derive(Copy, Clone, Debug, PartialEq, Default)]
        struct BufferedFlags: u8 {
            const DETACHED = 1 << 0;
            const WRITABLE = 1 << 1;
            const READABLE = 1 << 2;
        }
    }

    #[derive(Debug, Default)]
    struct BufferedData {
        raw: Option<PyObjectRef>,
        flags: BufferedFlags,
        abs_pos: Offset,
        buffer: Vec<u8>,
        pos: Offset,
        raw_pos: Offset,
        read_end: Offset,
        write_pos: Offset,
        write_end: Offset,
    }

    impl BufferedData {
        fn check_init(&self, vm: &VirtualMachine) -> PyResult<&PyObject> {
            if let Some(raw) = &self.raw {
                Ok(raw)
            } else {
                let msg = if self.flags.contains(BufferedFlags::DETACHED) {
                    "raw stream has been detached"
                } else {
                    "I/O operation on uninitialized object"
                };
                Err(vm.new_value_error(msg))
            }
        }

        #[inline]
        const fn writable(&self) -> bool {
            self.flags.contains(BufferedFlags::WRITABLE)
        }

        #[inline]
        const fn readable(&self) -> bool {
            self.flags.contains(BufferedFlags::READABLE)
        }

        #[inline]
        const fn valid_read(&self) -> bool {
            self.readable() && self.read_end != -1
        }

        #[inline]
        const fn valid_write(&self) -> bool {
            self.writable() && self.write_end != -1
        }

        #[inline]
        const fn raw_offset(&self) -> Offset {
            if (self.valid_read() || self.valid_write()) && self.raw_pos >= 0 {
                self.raw_pos - self.pos
            } else {
                0
            }
        }

        #[inline]
        const fn readahead(&self) -> Offset {
            if self.valid_read() {
                self.read_end - self.pos
            } else {
                0
            }
        }

        const fn reset_read(&mut self) {
            self.read_end = -1;
        }

        const fn reset_write(&mut self) {
            self.write_pos = 0;
            self.write_end = -1;
        }

        fn flush(&mut self, vm: &VirtualMachine) -> PyResult<()> {
            if !self.valid_write() || self.write_pos == self.write_end {
                self.reset_write();
                return Ok(());
            }

            let rewind = self.raw_offset() + (self.pos - self.write_pos);
            if rewind != 0 {
                self.raw_seek(-rewind, 1, vm)?;
                self.raw_pos = -rewind;
            }

            while self.write_pos < self.write_end {
                let n =
                    self.raw_write(None, self.write_pos as usize..self.write_end as usize, vm)?;
                let n = n.ok_or_else(|| {
                    vm.new_exception_msg(
                        vm.ctx.exceptions.blocking_io_error.to_owned(),
                        "write could not complete without blocking".to_owned(),
                    )
                })?;
                self.write_pos += n as Offset;
                self.raw_pos = self.write_pos;
                vm.check_signals()?;
            }

            self.reset_write();

            Ok(())
        }

        fn flush_rewind(&mut self, vm: &VirtualMachine) -> PyResult<()> {
            self.flush(vm)?;
            if self.readable() {
                let res = self.raw_seek(-self.raw_offset(), 1, vm);
                self.reset_read();
                res?;
            }
            Ok(())
        }

        fn raw_seek(&mut self, pos: Offset, whence: i32, vm: &VirtualMachine) -> PyResult<Offset> {
            let ret = vm.call_method(self.check_init(vm)?, "seek", (pos, whence))?;
            let offset = get_offset(ret, vm)?;
            if offset < 0 {
                return Err(
                    vm.new_os_error(format!("Raw stream returned invalid position {offset}"))
                );
            }
            self.abs_pos = offset;
            Ok(offset)
        }

        fn seek(&mut self, target: Offset, whence: i32, vm: &VirtualMachine) -> PyResult<Offset> {
            if matches!(whence, 0 | 1) && self.readable() {
                let current = self.raw_tell_cache(vm)?;
                let available = self.readahead();
                if available > 0 {
                    let offset = if whence == 0 {
                        target - (current - self.raw_offset())
                    } else {
                        target
                    };
                    if offset >= -self.pos && offset <= available {
                        self.pos += offset;
                        return Ok(current - available + offset);
                    }
                }
            }
            // raw.get_attr("seek", vm)?.call(args, vm)
            if self.writable() {
                self.flush(vm)?;
            }
            let target = if whence == 1 {
                target - self.raw_offset()
            } else {
                target
            };
            let res = self.raw_seek(target, whence, vm);
            self.raw_pos = -1;
            if res.is_ok() && self.readable() {
                self.reset_read();
            }
            res
        }

        fn raw_tell(&mut self, vm: &VirtualMachine) -> PyResult<Offset> {
            let raw = self.check_init(vm)?;
            let ret = vm.call_method(raw, "tell", ())?;
            let offset = get_offset(ret, vm)?;
            if offset < 0 {
                return Err(
                    vm.new_os_error(format!("Raw stream returned invalid position {offset}"))
                );
            }
            self.abs_pos = offset;
            Ok(offset)
        }

        fn raw_tell_cache(&mut self, vm: &VirtualMachine) -> PyResult<Offset> {
            if self.abs_pos == -1 {
                self.raw_tell(vm)
            } else {
                Ok(self.abs_pos)
            }
        }

        /// None means non-blocking failed
        fn raw_write(
            &mut self,
            buf: Option<PyBuffer>,
            buf_range: Range<usize>,
            vm: &VirtualMachine,
        ) -> PyResult<Option<usize>> {
            let len = buf_range.len();
            let res = if let Some(buf) = buf {
                let mem_obj = PyMemoryView::from_buffer_range(buf, buf_range, vm)?.to_pyobject(vm);

                // TODO: loop if write() raises an interrupt
                vm.call_method(self.raw.as_ref().unwrap(), "write", (mem_obj,))?
            } else {
                let v = std::mem::take(&mut self.buffer);
                let write_buf = VecBuffer::from(v).into_ref(&vm.ctx);
                let mem_obj = PyMemoryView::from_buffer_range(
                    write_buf.clone().into_pybuffer(true),
                    buf_range,
                    vm,
                )?
                .into_ref(&vm.ctx);

                // TODO: loop if write() raises an interrupt
                let res = vm.call_method(self.raw.as_ref().unwrap(), "write", (mem_obj.clone(),));

                mem_obj.release();
                self.buffer = write_buf.take();

                res?
            };

            if vm.is_none(&res) {
                return Ok(None);
            }
            let n = isize::try_from_object(vm, res)?;
            if n < 0 || n as usize > len {
                return Err(vm.new_os_error(format!(
                    "raw write() returned invalid length {n} (should have been between 0 and {len})"
                )));
            }
            if self.abs_pos != -1 {
                self.abs_pos += n as Offset
            }
            Ok(Some(n as usize))
        }

        fn write(&mut self, obj: ArgBytesLike, vm: &VirtualMachine) -> PyResult<usize> {
            if !self.valid_read() && !self.valid_write() {
                self.pos = 0;
                self.raw_pos = 0;
            }
            let avail = self.buffer.len() - self.pos as usize;
            let buf_len;
            {
                let buf = obj.borrow_buf();
                buf_len = buf.len();
                if buf.len() <= avail {
                    self.buffer[self.pos as usize..][..buf.len()].copy_from_slice(&buf);
                    if !self.valid_write() || self.write_pos > self.pos {
                        self.write_pos = self.pos
                    }
                    self.adjust_position(self.pos + buf.len() as Offset);
                    if self.pos > self.write_end {
                        self.write_end = self.pos
                    }
                    return Ok(buf.len());
                }
            }

            // TODO: something something check if error is BlockingIOError?
            let _ = self.flush(vm);

            let offset = self.raw_offset();
            if offset != 0 {
                self.raw_seek(-offset, 1, vm)?;
                self.raw_pos -= offset;
            }

            let mut remaining = buf_len;
            let mut written = 0;
            let buffer: PyBuffer = obj.into();
            while remaining > self.buffer.len() {
                let res = self.raw_write(Some(buffer.clone()), written..buf_len, vm)?;
                match res {
                    Some(n) => {
                        written += n;
                        if let Some(r) = remaining.checked_sub(n) {
                            remaining = r
                        } else {
                            break;
                        }
                        vm.check_signals()?;
                    }
                    None => {
                        // raw file is non-blocking
                        if remaining > self.buffer.len() {
                            // can't buffer everything, buffer what we can and error
                            let buf = buffer.as_contiguous().unwrap();
                            let buffer_len = self.buffer.len();
                            self.buffer.copy_from_slice(&buf[written..][..buffer_len]);
                            self.raw_pos = 0;
                            let buffer_size = self.buffer.len() as _;
                            self.adjust_position(buffer_size);
                            self.write_end = buffer_size;
                            // TODO: BlockingIOError(errno, msg, written)
                            // written += self.buffer.len();
                            return Err(vm.new_exception_msg(
                                vm.ctx.exceptions.blocking_io_error.to_owned(),
                                "write could not complete without blocking".to_owned(),
                            ));
                        } else {
                            break;
                        }
                    }
                }
            }
            if self.readable() {
                self.reset_read();
            }
            if remaining > 0 {
                let buf = buffer.as_contiguous().unwrap();
                self.buffer[..remaining].copy_from_slice(&buf[written..][..remaining]);
                written += remaining;
            }
            self.write_pos = 0;
            self.write_end = remaining as _;
            self.adjust_position(remaining as _);
            self.raw_pos = 0;

            Ok(written)
        }

        fn active_read_slice(&self) -> &[u8] {
            &self.buffer[self.pos as usize..][..self.readahead() as usize]
        }

        fn read_fast(&mut self, n: usize) -> Option<Vec<u8>> {
            let ret = self.active_read_slice().get(..n)?.to_vec();
            self.pos += n as Offset;
            Some(ret)
        }

        fn read_generic(&mut self, n: usize, vm: &VirtualMachine) -> PyResult<Option<Vec<u8>>> {
            if let Some(fast) = self.read_fast(n) {
                return Ok(Some(fast));
            }

            let current_size = self.readahead() as usize;

            let mut out = vec![0u8; n];
            let mut remaining = n;
            let mut written = 0;
            if current_size > 0 {
                let slice = self.active_read_slice();
                out[..slice.len()].copy_from_slice(slice);
                remaining -= current_size;
                written += current_size;
                self.pos += current_size as Offset;
            }
            if self.writable() {
                self.flush_rewind(vm)?;
            }
            self.reset_read();
            macro_rules! handle_opt_read {
                ($x:expr) => {
                    match ($x, written > 0) {
                        (Some(0), _) | (None, true) => {
                            out.truncate(written);
                            return Ok(Some(out));
                        }
                        (Some(r), _) => r,
                        (None, _) => return Ok(None),
                    }
                };
            }
            while remaining > 0 {
                // MINUS_LAST_BLOCK() in CPython
                let r = self.buffer.len() * (remaining / self.buffer.len());
                if r == 0 {
                    break;
                }
                let r = self.raw_read(Either::A(Some(&mut out)), written..written + r, vm)?;
                let r = handle_opt_read!(r);
                remaining -= r;
                written += r;
            }
            self.pos = 0;
            self.raw_pos = 0;
            self.read_end = 0;

            while remaining > 0 && (self.read_end as usize) < self.buffer.len() {
                let r = handle_opt_read!(self.fill_buffer(vm)?);
                if remaining > r {
                    out[written..][..r].copy_from_slice(&self.buffer[self.pos as usize..][..r]);
                    written += r;
                    self.pos += r as Offset;
                    remaining -= r;
                } else if remaining > 0 {
                    out[written..][..remaining]
                        .copy_from_slice(&self.buffer[self.pos as usize..][..remaining]);
                    written += remaining;
                    self.pos += remaining as Offset;
                    remaining = 0;
                }
                if remaining == 0 {
                    break;
                }
            }

            Ok(Some(out))
        }

        fn fill_buffer(&mut self, vm: &VirtualMachine) -> PyResult<Option<usize>> {
            let start = if self.valid_read() {
                self.read_end as usize
            } else {
                0
            };
            let buf_end = self.buffer.len();
            let res = self.raw_read(Either::A(None), start..buf_end, vm)?;
            if let Some(n) = res.filter(|n| *n > 0) {
                let new_start = (start + n) as Offset;
                self.read_end = new_start;
                self.raw_pos = new_start;
            }
            Ok(res)
        }

        fn raw_read(
            &mut self,
            v: Either<Option<&mut Vec<u8>>, PyBuffer>,
            buf_range: Range<usize>,
            vm: &VirtualMachine,
        ) -> PyResult<Option<usize>> {
            let len = buf_range.len();
            let res = match v {
                Either::A(v) => {
                    let v = v.unwrap_or(&mut self.buffer);
                    let read_buf = VecBuffer::from(std::mem::take(v)).into_ref(&vm.ctx);
                    let mem_obj = PyMemoryView::from_buffer_range(
                        read_buf.clone().into_pybuffer(false),
                        buf_range,
                        vm,
                    )?
                    .into_ref(&vm.ctx);

                    // TODO: loop if readinto() raises an interrupt
                    let res =
                        vm.call_method(self.raw.as_ref().unwrap(), "readinto", (mem_obj.clone(),));

                    mem_obj.release();
                    *v = read_buf.take();

                    res?
                }
                Either::B(buf) => {
                    let mem_obj = PyMemoryView::from_buffer_range(buf, buf_range, vm)?;
                    // TODO: loop if readinto() raises an interrupt
                    vm.call_method(self.raw.as_ref().unwrap(), "readinto", (mem_obj,))?
                }
            };

            if vm.is_none(&res) {
                return Ok(None);
            }
            let n = isize::try_from_object(vm, res)?;
            if n < 0 || n as usize > len {
                return Err(vm.new_os_error(format!(
                    "raw readinto() returned invalid length {n} (should have been between 0 and {len})"
                )));
            }
            if n > 0 && self.abs_pos != -1 {
                self.abs_pos += n as Offset
            }
            Ok(Some(n as usize))
        }

        fn read_all(&mut self, vm: &VirtualMachine) -> PyResult<Option<PyBytesRef>> {
            let buf = self.active_read_slice();
            let data = if buf.is_empty() {
                None
            } else {
                let b = buf.to_vec();
                self.pos += buf.len() as Offset;
                Some(b)
            };

            if self.writable() {
                self.flush_rewind(vm)?;
            }

            let readall = vm
                .get_str_method(self.raw.clone().unwrap(), "readall")
                .transpose()?;
            if let Some(readall) = readall {
                let res = readall.call((), vm)?;
                let res = <Option<PyBytesRef>>::try_from_object(vm, res)?;
                let ret = if let Some(mut data) = data {
                    if let Some(bytes) = res {
                        data.extend_from_slice(bytes.as_bytes());
                    }
                    Some(PyBytes::from(data).into_ref(&vm.ctx))
                } else {
                    res
                };
                return Ok(ret);
            }

            let mut chunks = Vec::new();

            let mut read_size = 0;
            loop {
                let read_data = vm.call_method(self.raw.as_ref().unwrap(), "read", ())?;
                let read_data = <Option<PyBytesRef>>::try_from_object(vm, read_data)?;

                match read_data {
                    Some(b) if !b.as_bytes().is_empty() => {
                        let l = b.as_bytes().len();
                        read_size += l;
                        if self.abs_pos != -1 {
                            self.abs_pos += l as Offset;
                        }
                        chunks.push(b);
                    }
                    read_data => {
                        let ret = if data.is_none() && read_size == 0 {
                            read_data
                        } else {
                            let mut data = data.unwrap_or_default();
                            data.reserve(read_size);
                            for bytes in &chunks {
                                data.extend_from_slice(bytes.as_bytes())
                            }
                            Some(PyBytes::from(data).into_ref(&vm.ctx))
                        };
                        break Ok(ret);
                    }
                }
            }
        }

        const fn adjust_position(&mut self, new_pos: Offset) {
            self.pos = new_pos;
            if self.valid_read() && self.read_end < self.pos {
                self.read_end = self.pos
            }
        }

        fn peek(&mut self, vm: &VirtualMachine) -> PyResult<Vec<u8>> {
            let have = self.readahead();
            let slice = if have > 0 {
                &self.buffer[self.pos as usize..][..have as usize]
            } else {
                self.reset_read();
                let r = self.fill_buffer(vm)?.unwrap_or(0);
                self.pos = 0;
                &self.buffer[..r]
            };
            Ok(slice.to_vec())
        }

        fn readinto_generic(
            &mut self,
            buf: PyBuffer,
            readinto1: bool,
            vm: &VirtualMachine,
        ) -> PyResult<Option<usize>> {
            let mut written = 0;
            let n = self.readahead();
            let buf_len;
            {
                let mut b = buf.as_contiguous_mut().unwrap();
                buf_len = b.len();
                if n > 0 {
                    if n as usize >= b.len() {
                        b.copy_from_slice(&self.buffer[self.pos as usize..][..buf_len]);
                        self.pos += buf_len as Offset;
                        return Ok(Some(buf_len));
                    }
                    b[..n as usize]
                        .copy_from_slice(&self.buffer[self.pos as usize..][..n as usize]);
                    self.pos += n;
                    written = n as usize;
                }
            }
            if self.writable() {
                self.flush_rewind(vm)?;
            }
            self.reset_read();
            self.pos = 0;

            let mut remaining = buf_len - written;
            while remaining > 0 {
                let n = if remaining > self.buffer.len() {
                    self.raw_read(Either::B(buf.clone()), written..written + remaining, vm)?
                } else if !(readinto1 && written != 0) {
                    let n = self.fill_buffer(vm)?;
                    if let Some(n) = n.filter(|&n| n > 0) {
                        let n = std::cmp::min(n, remaining);
                        buf.as_contiguous_mut().unwrap()[written..][..n]
                            .copy_from_slice(&self.buffer[self.pos as usize..][..n]);
                        self.pos += n as Offset;
                        written += n;
                        remaining -= n;
                        continue;
                    }
                    n
                } else {
                    break;
                };
                let n = match n {
                    Some(0) => break,
                    None if written > 0 => break,
                    None => return Ok(None),
                    Some(n) => n,
                };

                if readinto1 {
                    written += n;
                    break;
                }
                written += n;
                remaining -= n;
            }

            Ok(Some(written))
        }
    }

    pub fn get_offset(obj: PyObjectRef, vm: &VirtualMachine) -> PyResult<Offset> {
        let int = obj.try_index(vm)?;
        int.as_bigint().try_into().map_err(|_| {
            vm.new_value_error(format!(
                "cannot fit '{}' into an offset-sized integer",
                obj.class().name()
            ))
        })
    }

    pub fn repr_file_obj_name(obj: &PyObject, vm: &VirtualMachine) -> PyResult<Option<PyStrRef>> {
        let name = match obj.get_attr("name", vm) {
            Ok(name) => Some(name),
            Err(e)
                if e.fast_isinstance(vm.ctx.exceptions.attribute_error)
                    || e.fast_isinstance(vm.ctx.exceptions.value_error) =>
            {
                None
            }
            Err(e) => return Err(e),
        };
        match name {
            Some(name) => {
                if let Some(_guard) = ReprGuard::enter(vm, obj) {
                    name.repr(vm).map(Some)
                } else {
                    Err(vm.new_runtime_error(format!(
                        "reentrant call inside {}.__repr__",
                        obj.class().slot_name()
                    )))
                }
            }
            None => Ok(None),
        }
    }

    #[pyclass]
    trait BufferedMixin: PyPayload {
        const CLASS_NAME: &'static str;
        const READABLE: bool;
        const WRITABLE: bool;
        const SEEKABLE: bool = false;

        fn data(&self) -> &PyThreadMutex<BufferedData>;

        fn lock(&self, vm: &VirtualMachine) -> PyResult<PyThreadMutexGuard<'_, BufferedData>> {
            self.data()
                .lock()
                .ok_or_else(|| vm.new_runtime_error("reentrant call inside buffered io"))
        }

        #[pyslot]
        fn slot_init(zelf: PyObjectRef, args: FuncArgs, vm: &VirtualMachine) -> PyResult<()> {
            let zelf: PyRef<Self> = zelf.try_into_value(vm)?;
            zelf.__init__(args, vm)
        }

        #[pymethod]
        fn __init__(&self, args: FuncArgs, vm: &VirtualMachine) -> PyResult<()> {
            let (raw, BufferSize { buffer_size }): (PyObjectRef, _) =
                args.bind(vm).map_err(|e| {
                    let msg = format!("{}() {}", Self::CLASS_NAME, *e.__str__(vm));
                    vm.new_exception_msg(e.class().to_owned(), msg)
                })?;
            self.init(raw, BufferSize { buffer_size }, vm)
        }

        fn init(
            &self,
            raw: PyObjectRef,
            BufferSize { buffer_size }: BufferSize,
            vm: &VirtualMachine,
        ) -> PyResult<()> {
            let mut data = self.lock(vm)?;
            data.raw = None;
            data.flags.remove(BufferedFlags::DETACHED);

            let buffer_size = match buffer_size {
                OptionalArg::Present(i) if i <= 0 => {
                    return Err(vm.new_value_error("buffer size must be strictly positive"));
                }
                OptionalArg::Present(i) => i as usize,
                OptionalArg::Missing => DEFAULT_BUFFER_SIZE,
            };

            if Self::SEEKABLE {
                check_seekable(&raw, vm)?;
            }
            if Self::READABLE {
                data.flags.insert(BufferedFlags::READABLE);
                check_readable(&raw, vm)?;
            }
            if Self::WRITABLE {
                data.flags.insert(BufferedFlags::WRITABLE);
                check_writable(&raw, vm)?;
            }

            data.buffer = vec![0; buffer_size];

            if Self::READABLE {
                data.reset_read();
            }
            if Self::WRITABLE {
                data.reset_write();
            }
            if Self::SEEKABLE {
                data.pos = 0;
            }

            data.raw = Some(raw);

            Ok(())
        }

        #[pymethod]
        fn seek(
            &self,
            target: PyObjectRef,
            whence: OptionalArg<i32>,
            vm: &VirtualMachine,
        ) -> PyResult<Offset> {
            let whence = whence.unwrap_or(0);
            if !validate_whence(whence) {
                return Err(vm.new_value_error(format!("whence value {whence} unsupported")));
            }
            let mut data = self.lock(vm)?;
            let raw = data.check_init(vm)?;
            ensure_unclosed(raw, "seek of closed file", vm)?;
            check_seekable(raw, vm)?;
            let target = get_offset(target, vm)?;
            data.seek(target, whence, vm)
        }

        #[pymethod]
        fn tell(&self, vm: &VirtualMachine) -> PyResult<Offset> {
            let mut data = self.lock(vm)?;
            let raw_tell = data.raw_tell(vm)?;
            let raw_offset = data.raw_offset();
            let mut pos = raw_tell - raw_offset;
            // GH-95782
            if pos < 0 {
                pos = 0;
            }
            Ok(pos)
        }

        #[pymethod]
        fn truncate(
            zelf: PyRef<Self>,
            pos: OptionalOption<PyObjectRef>,
            vm: &VirtualMachine,
        ) -> PyResult {
            let pos = pos.flatten().to_pyobject(vm);
            let mut data = zelf.lock(vm)?;
            data.check_init(vm)?;
            if data.writable() {
                data.flush_rewind(vm)?;
            }
            let res = vm.call_method(data.raw.as_ref().unwrap(), "truncate", (pos,))?;
            let _ = data.raw_tell(vm);
            Ok(res)
        }
        #[pymethod]
        fn detach(zelf: PyRef<Self>, vm: &VirtualMachine) -> PyResult {
            vm.call_method(zelf.as_object(), "flush", ())?;
            let mut data = zelf.lock(vm)?;
            data.flags.insert(BufferedFlags::DETACHED);
            data.raw
                .take()
                .ok_or_else(|| vm.new_value_error("raw stream has been detached"))
        }

        #[pymethod]
        fn seekable(&self, vm: &VirtualMachine) -> PyResult {
            vm.call_method(self.lock(vm)?.check_init(vm)?, "seekable", ())
        }

        #[pygetset]
        fn raw(&self, vm: &VirtualMachine) -> PyResult<Option<PyObjectRef>> {
            Ok(self.lock(vm)?.raw.clone())
        }

        #[pygetset]
        fn closed(&self, vm: &VirtualMachine) -> PyResult {
            self.lock(vm)?.check_init(vm)?.get_attr("closed", vm)
        }

        #[pygetset]
        fn name(&self, vm: &VirtualMachine) -> PyResult {
            self.lock(vm)?.check_init(vm)?.get_attr("name", vm)
        }

        #[pygetset]
        fn mode(&self, vm: &VirtualMachine) -> PyResult {
            self.lock(vm)?.check_init(vm)?.get_attr("mode", vm)
        }

        #[pymethod]
        fn fileno(&self, vm: &VirtualMachine) -> PyResult {
            vm.call_method(self.lock(vm)?.check_init(vm)?, "fileno", ())
        }

        #[pymethod]
        fn isatty(&self, vm: &VirtualMachine) -> PyResult {
            vm.call_method(self.lock(vm)?.check_init(vm)?, "isatty", ())
        }

        #[pyslot]
        fn slot_repr(zelf: &PyObject, vm: &VirtualMachine) -> PyResult<PyRef<PyStr>> {
            let name_repr = repr_file_obj_name(zelf, vm)?;
            let cls = zelf.class();
            let slot_name = cls.slot_name();
            let repr = if let Some(name_repr) = name_repr {
                format!("<{slot_name} name={name_repr}>")
            } else {
                format!("<{slot_name}>")
            };
            Ok(vm.ctx.new_str(repr))
        }

        #[pymethod]
        fn __repr__(zelf: PyObjectRef, vm: &VirtualMachine) -> PyResult<PyRef<PyStr>> {
            Self::slot_repr(&zelf, vm)
        }

        fn close_strict(&self, vm: &VirtualMachine) -> PyResult {
            let mut data = self.lock(vm)?;
            let raw = data.check_init(vm)?;
            if file_closed(raw, vm)? {
                return Ok(vm.ctx.none());
            }
            let flush_res = data.flush(vm);
            let close_res = vm.call_method(data.raw.as_ref().unwrap(), "close", ());
            exception_chain(flush_res, close_res)
        }

        #[pymethod]
        fn close(zelf: PyRef<Self>, vm: &VirtualMachine) -> PyResult {
            {
                let data = zelf.lock(vm)?;
                let raw = data.check_init(vm)?;
                if file_closed(raw, vm)? {
                    return Ok(vm.ctx.none());
                }
            }
            let flush_res = vm.call_method(zelf.as_object(), "flush", ()).map(drop);
            let data = zelf.lock(vm)?;
            let raw = data.raw.as_ref().unwrap();
            let close_res = vm.call_method(raw, "close", ());
            exception_chain(flush_res, close_res)
        }

        #[pymethod]
        fn readable(&self) -> bool {
            Self::READABLE
        }

        #[pymethod]
        fn writable(&self) -> bool {
            Self::WRITABLE
        }

        // TODO: this should be the default for an equivalent of _PyObject_GetState
        #[pymethod]
        fn __reduce__(zelf: PyObjectRef, vm: &VirtualMachine) -> PyResult {
            Err(vm.new_type_error(format!("cannot pickle '{}' object", zelf.class().name())))
        }
    }

    #[pyclass]
    trait BufferedReadable: PyPayload {
        type Reader: BufferedMixin;

        fn reader(&self) -> &Self::Reader;

        #[pymethod]
        fn read(&self, size: OptionalSize, vm: &VirtualMachine) -> PyResult<Option<PyBytesRef>> {
            let mut data = self.reader().lock(vm)?;
            let raw = data.check_init(vm)?;
            let n = size.size.map(|s| *s).unwrap_or(-1);
            if n < -1 {
                return Err(vm.new_value_error("read length must be non-negative or -1"));
            }
            ensure_unclosed(raw, "read of closed file", vm)?;
            match n.to_usize() {
                Some(n) => data
                    .read_generic(n, vm)
                    .map(|x| x.map(|b| PyBytes::from(b).into_ref(&vm.ctx))),
                None => data.read_all(vm),
            }
        }

        #[pymethod]
        fn peek(&self, _size: OptionalSize, vm: &VirtualMachine) -> PyResult<Vec<u8>> {
            let mut data = self.reader().lock(vm)?;
            let raw = data.check_init(vm)?;
            ensure_unclosed(raw, "peek of closed file", vm)?;

            if data.writable() {
                let _ = data.flush_rewind(vm);
            }
            data.peek(vm)
        }

        #[pymethod]
        fn read1(&self, size: OptionalSize, vm: &VirtualMachine) -> PyResult<Vec<u8>> {
            let mut data = self.reader().lock(vm)?;
            let raw = data.check_init(vm)?;
            ensure_unclosed(raw, "read of closed file", vm)?;
            let n = size.to_usize().unwrap_or(data.buffer.len());
            if n == 0 {
                return Ok(Vec::new());
            }
            let have = data.readahead();
            if have > 0 {
                let n = std::cmp::min(have as usize, n);
                return Ok(data.read_fast(n).unwrap());
            }
            let mut v = vec![0; n];
            data.reset_read();
            let r = data
                .raw_read(Either::A(Some(&mut v)), 0..n, vm)?
                .unwrap_or(0);
            v.truncate(r);
            v.shrink_to_fit();
            Ok(v)
        }

        #[pymethod]
        fn readinto(&self, buf: ArgMemoryBuffer, vm: &VirtualMachine) -> PyResult<Option<usize>> {
            let mut data = self.reader().lock(vm)?;
            let raw = data.check_init(vm)?;
            ensure_unclosed(raw, "readinto of closed file", vm)?;
            data.readinto_generic(buf.into(), false, vm)
        }

        #[pymethod]
        fn readinto1(&self, buf: ArgMemoryBuffer, vm: &VirtualMachine) -> PyResult<Option<usize>> {
            let mut data = self.reader().lock(vm)?;
            let raw = data.check_init(vm)?;
            ensure_unclosed(raw, "readinto of closed file", vm)?;
            data.readinto_generic(buf.into(), true, vm)
        }
    }

    fn exception_chain<T>(e1: PyResult<()>, e2: PyResult<T>) -> PyResult<T> {
        match (e1, e2) {
            (Err(e1), Err(e)) => {
                e.set___context__(Some(e1));
                Err(e)
            }
            (Err(e), Ok(_)) | (Ok(()), Err(e)) => Err(e),
            (Ok(()), Ok(close_res)) => Ok(close_res),
        }
    }

    #[pyattr]
    #[pyclass(name = "BufferedReader", base = "_BufferedIOBase")]
    #[derive(Debug, Default, PyPayload)]
    struct BufferedReader {
        data: PyThreadMutex<BufferedData>,
    }

    impl BufferedMixin for BufferedReader {
        const CLASS_NAME: &'static str = "BufferedReader";
        const READABLE: bool = true;
        const WRITABLE: bool = false;

        fn data(&self) -> &PyThreadMutex<BufferedData> {
            &self.data
        }
    }

    impl BufferedReadable for BufferedReader {
        type Reader = Self;

        fn reader(&self) -> &Self::Reader {
            self
        }
    }

    #[pyclass(
        with(Constructor, BufferedMixin, BufferedReadable),
        flags(BASETYPE, HAS_DICT)
    )]
    impl BufferedReader {}

    impl DefaultConstructor for BufferedReader {}

    #[pyclass]
    trait BufferedWritable: PyPayload {
        type Writer: BufferedMixin;

        fn writer(&self) -> &Self::Writer;

        #[pymethod]
        fn write(&self, obj: ArgBytesLike, vm: &VirtualMachine) -> PyResult<usize> {
            let mut data = self.writer().lock(vm)?;
            let raw = data.check_init(vm)?;
            ensure_unclosed(raw, "write to closed file", vm)?;

            data.write(obj, vm)
        }

        #[pymethod]
        fn flush(&self, vm: &VirtualMachine) -> PyResult<()> {
            let mut data = self.writer().lock(vm)?;
            let raw = data.check_init(vm)?;
            ensure_unclosed(raw, "flush of closed file", vm)?;
            data.flush_rewind(vm)
        }
    }

    #[pyattr]
    #[pyclass(name = "BufferedWriter", base = "_BufferedIOBase")]
    #[derive(Debug, Default, PyPayload)]
    struct BufferedWriter {
        data: PyThreadMutex<BufferedData>,
    }

    impl BufferedMixin for BufferedWriter {
        const CLASS_NAME: &'static str = "BufferedWriter";
        const READABLE: bool = false;
        const WRITABLE: bool = true;

        fn data(&self) -> &PyThreadMutex<BufferedData> {
            &self.data
        }
    }

    impl BufferedWritable for BufferedWriter {
        type Writer = Self;

        fn writer(&self) -> &Self::Writer {
            self
        }
    }

    #[pyclass(
        with(Constructor, BufferedMixin, BufferedWritable),
        flags(BASETYPE, HAS_DICT)
    )]
    impl BufferedWriter {}

    impl DefaultConstructor for BufferedWriter {}

    #[pyattr]
    #[pyclass(name = "BufferedRandom", base = "_BufferedIOBase")]
    #[derive(Debug, Default, PyPayload)]
    struct BufferedRandom {
        data: PyThreadMutex<BufferedData>,
    }

    impl BufferedMixin for BufferedRandom {
        const CLASS_NAME: &'static str = "BufferedRandom";
        const READABLE: bool = true;
        const WRITABLE: bool = true;
        const SEEKABLE: bool = true;

        fn data(&self) -> &PyThreadMutex<BufferedData> {
            &self.data
        }
    }

    impl BufferedReadable for BufferedRandom {
        type Reader = Self;

        fn reader(&self) -> &Self::Reader {
            self
        }
    }

    impl BufferedWritable for BufferedRandom {
        type Writer = Self;

        fn writer(&self) -> &Self::Writer {
            self
        }
    }

    #[pyclass(
        with(Constructor, BufferedMixin, BufferedReadable, BufferedWritable),
        flags(BASETYPE, HAS_DICT)
    )]
    impl BufferedRandom {}

    impl DefaultConstructor for BufferedRandom {}

    #[pyattr]
    #[pyclass(name = "BufferedRWPair", base = "_BufferedIOBase")]
    #[derive(Debug, Default, PyPayload)]
    struct BufferedRWPair {
        read: BufferedReader,
        write: BufferedWriter,
    }

    impl BufferedReadable for BufferedRWPair {
        type Reader = BufferedReader;

        fn reader(&self) -> &Self::Reader {
            &self.read
        }
    }

    impl BufferedWritable for BufferedRWPair {
        type Writer = BufferedWriter;

        fn writer(&self) -> &Self::Writer {
            &self.write
        }
    }

    impl DefaultConstructor for BufferedRWPair {}

    impl Initializer for BufferedRWPair {
        type Args = (PyObjectRef, PyObjectRef, BufferSize);

        fn init(
            zelf: PyRef<Self>,
            (reader, writer, buffer_size): Self::Args,
            vm: &VirtualMachine,
        ) -> PyResult<()> {
            zelf.read.init(reader, buffer_size.clone(), vm)?;
            zelf.write.init(writer, buffer_size, vm)?;
            Ok(())
        }
    }

    #[pyclass(
        with(Constructor, Initializer, BufferedReadable, BufferedWritable),
        flags(BASETYPE, HAS_DICT)
    )]
    impl BufferedRWPair {
        #[pymethod]
        fn flush(&self, vm: &VirtualMachine) -> PyResult<()> {
            self.write.flush(vm)
        }

        #[pymethod]
        const fn readable(&self) -> bool {
            true
        }
        #[pymethod]
        const fn writable(&self) -> bool {
            true
        }

        #[pygetset]
        fn closed(&self, vm: &VirtualMachine) -> PyResult {
            self.write.closed(vm)
        }

        #[pymethod]
        fn isatty(&self, vm: &VirtualMachine) -> PyResult {
            // read.isatty() or write.isatty()
            let res = self.read.isatty(vm)?;
            if res.clone().try_to_bool(vm)? {
                Ok(res)
            } else {
                self.write.isatty(vm)
            }
        }

        #[pymethod]
        fn close(&self, vm: &VirtualMachine) -> PyResult {
            let write_res = self.write.close_strict(vm).map(drop);
            let read_res = self.read.close_strict(vm);
            exception_chain(write_res, read_res)
        }
    }

    #[derive(FromArgs)]
    struct TextIOWrapperArgs {
        #[pyarg(any, default)]
        encoding: Option<PyUtf8StrRef>,
        #[pyarg(any, default)]
        errors: Option<PyStrRef>,
        #[pyarg(any, default)]
        newline: Option<Newlines>,
        #[pyarg(any, default)]
        line_buffering: Option<bool>,
        #[pyarg(any, default)]
        write_through: Option<bool>,
    }

    #[derive(Debug, Copy, Clone, Default)]
    enum Newlines {
        #[default]
        Universal,
        Passthrough,
        Lf,
        Cr,
        Crlf,
    }

    impl Newlines {
        /// returns position where the new line starts if found, otherwise position at which to
        /// continue the search after more is read into the buffer
        fn find_newline(&self, s: &Wtf8) -> Result<usize, usize> {
            let len = s.len();
            match self {
                Self::Universal | Self::Lf => s.find("\n".as_ref()).map(|p| p + 1).ok_or(len),
                Self::Passthrough => {
                    let bytes = s.as_bytes();
                    memchr::memchr2(b'\n', b'\r', bytes)
                        .map(|p| {
                            let nl_len =
                                if bytes[p] == b'\r' && bytes.get(p + 1).copied() == Some(b'\n') {
                                    2
                                } else {
                                    1
                                };
                            p + nl_len
                        })
                        .ok_or(len)
                }
                Self::Cr => s.find("\n".as_ref()).map(|p| p + 1).ok_or(len),
                Self::Crlf => {
                    // s[searched..] == remaining
                    let mut searched = 0;
                    let mut remaining = s.as_bytes();
                    loop {
                        match memchr::memchr(b'\r', remaining) {
                            Some(p) => match remaining.get(p + 1) {
                                Some(&ch_after_cr) => {
                                    let pos_after = p + 2;
                                    if ch_after_cr == b'\n' {
                                        break Ok(searched + pos_after);
                                    } else {
                                        searched += pos_after;
                                        remaining = &remaining[pos_after..];
                                        continue;
                                    }
                                }
                                None => break Err(searched + p),
                            },
                            None => break Err(len),
                        }
                    }
                }
            }
        }
    }

    impl TryFromObject for Newlines {
        fn try_from_object(vm: &VirtualMachine, obj: PyObjectRef) -> PyResult<Self> {
            let nl = if vm.is_none(&obj) {
                Self::Universal
            } else {
                let s = obj.downcast::<PyStr>().map_err(|obj| {
                    vm.new_type_error(format!(
                        "newline argument must be str or None, not {}",
                        obj.class().name()
                    ))
                })?;
                match s.as_str() {
                    "" => Self::Passthrough,
                    "\n" => Self::Lf,
                    "\r" => Self::Cr,
                    "\r\n" => Self::Crlf,
                    _ => return Err(vm.new_value_error(format!("illegal newline value: {s}"))),
                }
            };
            Ok(nl)
        }
    }

    /// A length of or index into a UTF-8 string, measured in both chars and bytes
    #[derive(Debug, Default, Copy, Clone)]
    struct Utf8size {
        bytes: usize,
        chars: usize,
    }

    impl Utf8size {
        fn len_pystr(s: &PyStr) -> Self {
            Self {
                bytes: s.byte_len(),
                chars: s.char_len(),
            }
        }

        fn len_str(s: &Wtf8) -> Self {
            Self {
                bytes: s.len(),
                chars: s.code_points().count(),
            }
        }
    }

    impl std::ops::Add for Utf8size {
        type Output = Self;

        #[inline]
        fn add(mut self, rhs: Self) -> Self {
            self += rhs;
            self
        }
    }

    impl std::ops::AddAssign for Utf8size {
        #[inline]
        fn add_assign(&mut self, rhs: Self) {
            self.bytes += rhs.bytes;
            self.chars += rhs.chars;
        }
    }

    impl std::ops::Sub for Utf8size {
        type Output = Self;

        #[inline]
        fn sub(mut self, rhs: Self) -> Self {
            self -= rhs;
            self
        }
    }

    impl std::ops::SubAssign for Utf8size {
        #[inline]
        fn sub_assign(&mut self, rhs: Self) {
            self.bytes -= rhs.bytes;
            self.chars -= rhs.chars;
        }
    }

    // TODO: implement legit fast-paths for other encodings
    type EncodeFunc = fn(PyStrRef) -> PendingWrite;
    const fn textio_encode_utf8(s: PyStrRef) -> PendingWrite {
        PendingWrite::Utf8(s)
    }

    #[derive(Debug)]
    struct TextIOData {
        buffer: PyObjectRef,
        encoder: Option<(PyObjectRef, Option<EncodeFunc>)>,
        decoder: Option<PyObjectRef>,
        encoding: PyUtf8StrRef,
        errors: PyStrRef,
        newline: Newlines,
        line_buffering: bool,
        write_through: bool,
        chunk_size: usize,
        seekable: bool,
        has_read1: bool,
        // these are more state than configuration
        pending: PendingWrites,
        telling: bool,
        snapshot: Option<(i32, PyBytesRef)>,
        decoded_chars: Option<PyStrRef>,
        // number of characters we've consumed from decoded_chars
        decoded_chars_used: Utf8size,
        b2cratio: f64,
    }

    #[derive(Debug, Default)]
    struct PendingWrites {
        num_bytes: usize,
        data: PendingWritesData,
    }

    #[derive(Debug, Default)]
    enum PendingWritesData {
        #[default]
        None,
        One(PendingWrite),
        Many(Vec<PendingWrite>),
    }

    #[derive(Debug)]
    enum PendingWrite {
        Utf8(PyStrRef),
        Bytes(PyBytesRef),
    }

    impl PendingWrite {
        fn as_bytes(&self) -> &[u8] {
            match self {
                Self::Utf8(s) => s.as_bytes(),
                Self::Bytes(b) => b.as_bytes(),
            }
        }
    }

    impl PendingWrites {
        fn push(&mut self, write: PendingWrite) {
            self.num_bytes += write.as_bytes().len();
            self.data = match std::mem::take(&mut self.data) {
                PendingWritesData::None => PendingWritesData::One(write),
                PendingWritesData::One(write1) => PendingWritesData::Many(vec![write1, write]),
                PendingWritesData::Many(mut v) => {
                    v.push(write);
                    PendingWritesData::Many(v)
                }
            }
        }
        fn take(&mut self, vm: &VirtualMachine) -> PyBytesRef {
            let Self { num_bytes, data } = std::mem::take(self);
            if let PendingWritesData::One(PendingWrite::Bytes(b)) = data {
                return b;
            }
            let writes_iter = match data {
                PendingWritesData::None => itertools::Either::Left(vec![].into_iter()),
                PendingWritesData::One(write) => itertools::Either::Right(std::iter::once(write)),
                PendingWritesData::Many(writes) => itertools::Either::Left(writes.into_iter()),
            };
            let mut buf = Vec::with_capacity(num_bytes);
            writes_iter.for_each(|chunk| buf.extend_from_slice(chunk.as_bytes()));
            PyBytes::from(buf).into_ref(&vm.ctx)
        }
    }

    #[derive(Default, Debug)]
    struct TextIOCookie {
        start_pos: Offset,
        dec_flags: i32,
        bytes_to_feed: i32,
        chars_to_skip: i32,
        need_eof: bool,
        // chars_to_skip but utf8 bytes
        bytes_to_skip: i32,
    }

    impl TextIOCookie {
        const START_POS_OFF: usize = 0;
        const DEC_FLAGS_OFF: usize = Self::START_POS_OFF + std::mem::size_of::<Offset>();
        const BYTES_TO_FEED_OFF: usize = Self::DEC_FLAGS_OFF + 4;
        const CHARS_TO_SKIP_OFF: usize = Self::BYTES_TO_FEED_OFF + 4;
        const NEED_EOF_OFF: usize = Self::CHARS_TO_SKIP_OFF + 4;
        const BYTES_TO_SKIP_OFF: usize = Self::NEED_EOF_OFF + 1;
        const BYTE_LEN: usize = Self::BYTES_TO_SKIP_OFF + 4;

        fn parse(cookie: &BigInt) -> Option<Self> {
            let (_, mut buf) = cookie.to_bytes_le();
            if buf.len() > Self::BYTE_LEN {
                return None;
            }
            buf.resize(Self::BYTE_LEN, 0);
            let buf: &[u8; Self::BYTE_LEN] = buf.as_slice().try_into().unwrap();
            macro_rules! get_field {
                ($t:ty, $off:ident) => {{
                    <$t>::from_ne_bytes(
                        buf[Self::$off..][..std::mem::size_of::<$t>()]
                            .try_into()
                            .unwrap(),
                    )
                }};
            }
            Some(Self {
                start_pos: get_field!(Offset, START_POS_OFF),
                dec_flags: get_field!(i32, DEC_FLAGS_OFF),
                bytes_to_feed: get_field!(i32, BYTES_TO_FEED_OFF),
                chars_to_skip: get_field!(i32, CHARS_TO_SKIP_OFF),
                need_eof: get_field!(u8, NEED_EOF_OFF) != 0,
                bytes_to_skip: get_field!(i32, BYTES_TO_SKIP_OFF),
            })
        }

        fn build(&self) -> BigInt {
            let mut buf = [0; Self::BYTE_LEN];
            macro_rules! set_field {
                ($field:expr, $off:ident) => {{
                    let field = $field;
                    buf[Self::$off..][..std::mem::size_of_val(&field)]
                        .copy_from_slice(&field.to_ne_bytes())
                }};
            }
            set_field!(self.start_pos, START_POS_OFF);
            set_field!(self.dec_flags, DEC_FLAGS_OFF);
            set_field!(self.bytes_to_feed, BYTES_TO_FEED_OFF);
            set_field!(self.chars_to_skip, CHARS_TO_SKIP_OFF);
            set_field!(self.need_eof as u8, NEED_EOF_OFF);
            set_field!(self.bytes_to_skip, BYTES_TO_SKIP_OFF);
            BigUint::from_bytes_le(&buf).into()
        }

        fn set_decoder_state(&self, decoder: &PyObject, vm: &VirtualMachine) -> PyResult<()> {
            if self.start_pos == 0 && self.dec_flags == 0 {
                vm.call_method(decoder, "reset", ())?;
            } else {
                vm.call_method(
                    decoder,
                    "setstate",
                    ((vm.ctx.new_bytes(vec![]), self.dec_flags),),
                )?;
            }
            Ok(())
        }

        const fn num_to_skip(&self) -> Utf8size {
            Utf8size {
                bytes: self.bytes_to_skip as usize,
                chars: self.chars_to_skip as usize,
            }
        }

        const fn set_num_to_skip(&mut self, num: Utf8size) {
            self.bytes_to_skip = num.bytes as i32;
            self.chars_to_skip = num.chars as i32;
        }
    }

    #[pyattr]
    #[pyclass(name = "TextIOWrapper", base = "_TextIOBase")]
    #[derive(Debug, Default, PyPayload)]
    struct TextIOWrapper {
        data: PyThreadMutex<Option<TextIOData>>,
    }

    impl DefaultConstructor for TextIOWrapper {}

    impl Initializer for TextIOWrapper {
        type Args = (PyObjectRef, TextIOWrapperArgs);

        fn init(
            zelf: PyRef<Self>,
            (buffer, args): Self::Args,
            vm: &VirtualMachine,
        ) -> PyResult<()> {
            let mut data = zelf.lock_opt(vm)?;
            *data = None;

            let encoding = match args.encoding {
                None if vm.state.settings.utf8_mode > 0 => identifier_utf8!(vm, utf_8).to_owned(),
                Some(enc) if enc.as_str() != "locale" => enc,
                _ => {
                    // None without utf8_mode or "locale" encoding
                    vm.import("locale", 0)?
                        .get_attr("getencoding", vm)?
                        .call((), vm)?
                        .try_into_value(vm)?
                }
            };

            let errors = args
                .errors
                .unwrap_or_else(|| identifier!(vm, strict).to_owned());

            let has_read1 = vm.get_attribute_opt(buffer.clone(), "read1")?.is_some();
            let seekable = vm.call_method(&buffer, "seekable", ())?.try_to_bool(vm)?;

            let newline = args.newline.unwrap_or_default();
            let (encoder, decoder) =
                Self::find_coder(&buffer, encoding.as_str(), &errors, newline, vm)?;

            *data = Some(TextIOData {
                buffer,
                encoder,
                decoder,
                encoding,
                errors,
                newline,
                line_buffering: args.line_buffering.unwrap_or_default(),
                write_through: args.write_through.unwrap_or_default(),
                chunk_size: 8192,
                seekable,
                has_read1,

                pending: PendingWrites::default(),
                telling: seekable,
                snapshot: None,
                decoded_chars: None,
                decoded_chars_used: Utf8size::default(),
                b2cratio: 0.0,
            });

            Ok(())
        }
    }

    impl TextIOWrapper {
        fn lock_opt(
            &self,
            vm: &VirtualMachine,
        ) -> PyResult<PyThreadMutexGuard<'_, Option<TextIOData>>> {
            self.data
                .lock()
                .ok_or_else(|| vm.new_runtime_error("reentrant call inside textio"))
        }

        fn lock(&self, vm: &VirtualMachine) -> PyResult<PyMappedThreadMutexGuard<'_, TextIOData>> {
            let lock = self.lock_opt(vm)?;
            PyThreadMutexGuard::try_map(lock, |x| x.as_mut())
                .map_err(|_| vm.new_value_error("I/O operation on uninitialized object"))
        }

        #[allow(clippy::type_complexity)]
        fn find_coder(
            buffer: &PyObject,
            encoding: &str,
            errors: &Py<PyStr>,
            newline: Newlines,
            vm: &VirtualMachine,
        ) -> PyResult<(
            Option<(PyObjectRef, Option<EncodeFunc>)>,
            Option<PyObjectRef>,
        )> {
            let codec = vm.state.codec_registry.lookup(encoding, vm)?;

            let encoder = if vm.call_method(buffer, "writable", ())?.try_to_bool(vm)? {
                let incremental_encoder =
                    codec.get_incremental_encoder(Some(errors.to_owned()), vm)?;
                let encoding_name = vm.get_attribute_opt(incremental_encoder.clone(), "name")?;
                let encode_func = encoding_name.and_then(|name| {
                    let name = name.downcast_ref::<PyStr>()?;
                    match name.as_str() {
                        "utf-8" => Some(textio_encode_utf8 as EncodeFunc),
                        _ => None,
                    }
                });
                Some((incremental_encoder, encode_func))
            } else {
                None
            };

            let decoder = if vm.call_method(buffer, "readable", ())?.try_to_bool(vm)? {
                let decoder = codec.get_incremental_decoder(Some(errors.to_owned()), vm)?;
                if let Newlines::Universal | Newlines::Passthrough = newline {
                    let args = IncrementalNewlineDecoderArgs {
                        decoder,
                        translate: matches!(newline, Newlines::Universal),
                        errors: None,
                    };
                    Some(IncrementalNewlineDecoder::construct_and_init(args, vm)?.into())
                } else {
                    Some(decoder)
                }
            } else {
                None
            };
            Ok((encoder, decoder))
        }
    }

    #[pyclass(with(Constructor, Initializer), flags(BASETYPE))]
    impl TextIOWrapper {
        #[pymethod]
        fn reconfigure(&self, args: TextIOWrapperArgs, vm: &VirtualMachine) -> PyResult<()> {
            let mut data = self.data.lock().unwrap();
            if let Some(data) = data.as_mut() {
                if let Some(encoding) = args.encoding {
                    let (encoder, decoder) = Self::find_coder(
                        &data.buffer,
                        encoding.as_str(),
                        &data.errors,
                        data.newline,
                        vm,
                    )?;
                    data.encoding = encoding;
                    data.encoder = encoder;
                    data.decoder = decoder;
                }
                if let Some(errors) = args.errors {
                    data.errors = errors;
                }
                if let Some(newline) = args.newline {
                    data.newline = newline;
                }
                if let Some(line_buffering) = args.line_buffering {
                    data.line_buffering = line_buffering;
                }
                if let Some(write_through) = args.write_through {
                    data.write_through = write_through;
                }
            }
            Ok(())
        }

        #[pymethod]
        fn seekable(&self, vm: &VirtualMachine) -> PyResult {
            let textio = self.lock(vm)?;
            vm.call_method(&textio.buffer, "seekable", ())
        }

        #[pymethod]
        fn readable(&self, vm: &VirtualMachine) -> PyResult {
            let textio = self.lock(vm)?;
            vm.call_method(&textio.buffer, "readable", ())
        }

        #[pymethod]
        fn writable(&self, vm: &VirtualMachine) -> PyResult {
            let textio = self.lock(vm)?;
            vm.call_method(&textio.buffer, "writable", ())
        }

        #[pygetset]
        fn line_buffering(&self, vm: &VirtualMachine) -> PyResult<bool> {
            Ok(self.lock(vm)?.line_buffering)
        }

        #[pygetset]
        fn write_through(&self, vm: &VirtualMachine) -> PyResult<bool> {
            Ok(self.lock(vm)?.write_through)
        }

        #[pygetset]
        fn newlines(&self, vm: &VirtualMachine) -> PyResult<Option<PyObjectRef>> {
            let data = self.lock(vm)?;
            let Some(decoder) = &data.decoder else {
                return Ok(None);
            };
            vm.get_attribute_opt(decoder.clone(), "newlines")
        }

        #[pygetset(name = "_CHUNK_SIZE")]
        fn chunksize(&self, vm: &VirtualMachine) -> PyResult<usize> {
            Ok(self.lock(vm)?.chunk_size)
        }

        #[pygetset(setter, name = "_CHUNK_SIZE")]
        fn set_chunksize(
            &self,
            chunk_size: PySetterValue<usize>,
            vm: &VirtualMachine,
        ) -> PyResult<()> {
            let mut textio = self.lock(vm)?;
            match chunk_size {
                PySetterValue::Assign(chunk_size) => textio.chunk_size = chunk_size,
                PySetterValue::Delete => Err(vm.new_attribute_error("cannot delete attribute"))?,
            };
            // TODO: RUSTPYTHON
            // Change chunk_size type, validate it manually and throws ValueError if invalid.
            // https://github.com/python/cpython/blob/2e9da8e3522764d09f1d6054a2be567e91a30812/Modules/_io/textio.c#L3124-L3143
            Ok(())
        }

        #[pymethod]
        fn seek(
            zelf: PyRef<Self>,
            cookie: PyObjectRef,
            how: OptionalArg<i32>,
            vm: &VirtualMachine,
        ) -> PyResult {
            let how = how.unwrap_or(0);

            let reset_encoder = |encoder, start_of_stream| {
                if start_of_stream {
                    vm.call_method(encoder, "reset", ())
                } else {
                    vm.call_method(encoder, "setstate", (0,))
                }
            };

            let textio = zelf.lock(vm)?;

            if !textio.seekable {
                return Err(new_unsupported_operation(
                    vm,
                    "underlying stream is not seekable".to_owned(),
                ));
            }

            let cookie = match how {
                // SEEK_SET
                0 => cookie,
                // SEEK_CUR
                1 => {
                    if vm.bool_eq(&cookie, vm.ctx.new_int(0).as_ref())? {
                        vm.call_method(&textio.buffer, "tell", ())?
                    } else {
                        return Err(new_unsupported_operation(
                            vm,
                            "can't do nonzero cur-relative seeks".to_owned(),
                        ));
                    }
                }
                // SEEK_END
                2 => {
                    if vm.bool_eq(&cookie, vm.ctx.new_int(0).as_ref())? {
                        drop(textio);
                        vm.call_method(zelf.as_object(), "flush", ())?;
                        let mut textio = zelf.lock(vm)?;
                        textio.set_decoded_chars(None);
                        textio.snapshot = None;
                        if let Some(decoder) = &textio.decoder {
                            vm.call_method(decoder, "reset", ())?;
                        }
                        let res = vm.call_method(&textio.buffer, "seek", (0, 2))?;
                        if let Some((encoder, _)) = &textio.encoder {
                            let start_of_stream = vm.bool_eq(&res, vm.ctx.new_int(0).as_ref())?;
                            reset_encoder(encoder, start_of_stream)?;
                        }
                        return Ok(res);
                    } else {
                        return Err(new_unsupported_operation(
                            vm,
                            "can't do nonzero end-relative seeks".to_owned(),
                        ));
                    }
                }
                _ => {
                    return Err(
                        vm.new_value_error(format!("invalid whence ({how}, should be 0, 1 or 2)"))
                    );
                }
            };
            use crate::types::PyComparisonOp;
            if cookie.rich_compare_bool(vm.ctx.new_int(0).as_ref(), PyComparisonOp::Lt, vm)? {
                return Err(
                    vm.new_value_error(format!("negative seek position {}", &cookie.repr(vm)?))
                );
            }
            drop(textio);
            vm.call_method(zelf.as_object(), "flush", ())?;
            let cookie_obj = crate::builtins::PyIntRef::try_from_object(vm, cookie)?;
            let cookie = TextIOCookie::parse(cookie_obj.as_bigint())
                .ok_or_else(|| vm.new_value_error("invalid cookie"))?;
            let mut textio = zelf.lock(vm)?;
            vm.call_method(&textio.buffer, "seek", (cookie.start_pos,))?;
            textio.set_decoded_chars(None);
            textio.snapshot = None;
            if let Some(decoder) = &textio.decoder {
                cookie.set_decoder_state(decoder, vm)?;
            }
            if cookie.chars_to_skip != 0 {
                let TextIOData {
                    ref decoder,
                    ref buffer,
                    ref mut snapshot,
                    ..
                } = *textio;
                let decoder = decoder
                    .as_ref()
                    .ok_or_else(|| vm.new_value_error("invalid cookie"))?;
                let input_chunk = vm.call_method(buffer, "read", (cookie.bytes_to_feed,))?;
                let input_chunk: PyBytesRef = input_chunk.downcast().map_err(|obj| {
                    vm.new_type_error(format!(
                        "underlying read() should have returned a bytes object, not '{}'",
                        obj.class().name()
                    ))
                })?;
                *snapshot = Some((cookie.dec_flags, input_chunk.clone()));
                let decoded = vm.call_method(decoder, "decode", (input_chunk, cookie.need_eof))?;
                let decoded = check_decoded(decoded, vm)?;
                let pos_is_valid = decoded
                    .as_wtf8()
                    .is_code_point_boundary(cookie.bytes_to_skip as usize);
                textio.set_decoded_chars(Some(decoded));
                if !pos_is_valid {
                    return Err(vm.new_os_error("can't restore logical file position"));
                }
                textio.decoded_chars_used = cookie.num_to_skip();
            } else {
                textio.snapshot = Some((cookie.dec_flags, PyBytes::from(vec![]).into_ref(&vm.ctx)))
            }
            if let Some((encoder, _)) = &textio.encoder {
                let start_of_stream = cookie.start_pos == 0 && cookie.dec_flags == 0;
                reset_encoder(encoder, start_of_stream)?;
            }
            Ok(cookie_obj.into())
        }

        #[pymethod]
        fn tell(zelf: PyRef<Self>, vm: &VirtualMachine) -> PyResult {
            let mut textio = zelf.lock(vm)?;
            if !textio.seekable {
                return Err(new_unsupported_operation(
                    vm,
                    "underlying stream is not seekable".to_owned(),
                ));
            }
            if !textio.telling {
                return Err(vm.new_os_error("telling position disabled by next() call"));
            }
            textio.write_pending(vm)?;
            drop(textio);
            vm.call_method(zelf.as_object(), "flush", ())?;
            let textio = zelf.lock(vm)?;
            let pos = vm.call_method(&textio.buffer, "tell", ())?;
            let (decoder, (dec_flags, next_input)) = match (&textio.decoder, &textio.snapshot) {
                (Some(d), Some(s)) => (d, s),
                _ => return Ok(pos),
            };
            let pos = Offset::try_from_object(vm, pos)?;
            let mut cookie = TextIOCookie {
                start_pos: pos - next_input.len() as Offset,
                dec_flags: *dec_flags,
                ..Default::default()
            };
            if textio.decoded_chars_used.bytes == 0 {
                return Ok(cookie.build().to_pyobject(vm));
            }
            let decoder_getstate = || {
                let state = vm.call_method(decoder, "getstate", ())?;
                parse_decoder_state(state, vm)
            };
            let decoder_decode = |b: &[u8]| {
                let decoded = vm.call_method(decoder, "decode", (vm.ctx.new_bytes(b.to_vec()),))?;
                let decoded = check_decoded(decoded, vm)?;
                Ok(Utf8size::len_pystr(&decoded))
            };
            let saved_state = vm.call_method(decoder, "getstate", ())?;
            let mut num_to_skip = textio.decoded_chars_used;
            let mut skip_bytes = (textio.b2cratio * num_to_skip.chars as f64) as isize;
            let mut skip_back = 1;
            while skip_bytes > 0 {
                cookie.set_decoder_state(decoder, vm)?;
                let input = &next_input.as_bytes()[..skip_bytes as usize];
                let n_decoded = decoder_decode(input)?;
                if n_decoded.chars <= num_to_skip.chars {
                    let (dec_buffer, dec_flags) = decoder_getstate()?;
                    if dec_buffer.is_empty() {
                        cookie.dec_flags = dec_flags;
                        num_to_skip -= n_decoded;
                        break;
                    }
                    skip_bytes -= dec_buffer.len() as isize;
                    skip_back = 1;
                } else {
                    skip_bytes -= skip_back;
                    skip_back *= 2;
                }
            }
            if skip_bytes <= 0 {
                skip_bytes = 0;
                cookie.set_decoder_state(decoder, vm)?;
            }
            let skip_bytes = skip_bytes as usize;

            cookie.start_pos += skip_bytes as Offset;
            cookie.set_num_to_skip(num_to_skip);

            if num_to_skip.chars != 0 {
                let mut n_decoded = Utf8size::default();
                let mut input = next_input.as_bytes();
                input = &input[skip_bytes..];
                while !input.is_empty() {
                    let (byte1, rest) = input.split_at(1);
                    let n = decoder_decode(byte1)?;
                    n_decoded += n;
                    cookie.bytes_to_feed += 1;
                    let (dec_buffer, dec_flags) = decoder_getstate()?;
                    if dec_buffer.is_empty() && n_decoded.chars < num_to_skip.chars {
                        cookie.start_pos += cookie.bytes_to_feed as Offset;
                        num_to_skip -= n_decoded;
                        cookie.dec_flags = dec_flags;
                        cookie.bytes_to_feed = 0;
                        n_decoded = Utf8size::default();
                    }
                    if n_decoded.chars >= num_to_skip.chars {
                        break;
                    }
                    input = rest;
                }
                if input.is_empty() {
                    let decoded =
                        vm.call_method(decoder, "decode", (vm.ctx.new_bytes(vec![]), true))?;
                    let decoded = check_decoded(decoded, vm)?;
                    let final_decoded_chars = n_decoded.chars + decoded.char_len();
                    cookie.need_eof = true;
                    if final_decoded_chars < num_to_skip.chars {
                        return Err(vm.new_os_error("can't reconstruct logical file position"));
                    }
                }
            }
            vm.call_method(decoder, "setstate", (saved_state,))?;
            cookie.set_num_to_skip(num_to_skip);
            Ok(cookie.build().to_pyobject(vm))
        }

        #[pygetset]
        fn name(&self, vm: &VirtualMachine) -> PyResult {
            let buffer = self.lock(vm)?.buffer.clone();
            buffer.get_attr("name", vm)
        }

        #[pygetset]
        fn encoding(&self, vm: &VirtualMachine) -> PyResult<PyUtf8StrRef> {
            Ok(self.lock(vm)?.encoding.clone())
        }

        #[pygetset]
        fn errors(&self, vm: &VirtualMachine) -> PyResult<PyStrRef> {
            Ok(self.lock(vm)?.errors.clone())
        }

        #[pymethod]
        fn fileno(&self, vm: &VirtualMachine) -> PyResult {
            let buffer = self.lock(vm)?.buffer.clone();
            vm.call_method(&buffer, "fileno", ())
        }

        #[pymethod]
        fn read(&self, size: OptionalSize, vm: &VirtualMachine) -> PyResult<PyStrRef> {
            let mut textio = self.lock(vm)?;
            textio.check_closed(vm)?;
            let decoder = textio
                .decoder
                .clone()
                .ok_or_else(|| new_unsupported_operation(vm, "not readable".to_owned()))?;

            textio.write_pending(vm)?;

            let s = if let Some(mut remaining) = size.to_usize() {
                let mut chunks = Vec::new();
                let mut chunks_bytes = 0;
                loop {
                    if let Some((s, char_len)) = textio.get_decoded_chars(remaining, vm) {
                        chunks_bytes += s.byte_len();
                        chunks.push(s);
                        remaining = remaining.saturating_sub(char_len);
                    }
                    if remaining == 0 {
                        break;
                    }
                    let eof = textio.read_chunk(remaining, vm)?;
                    if eof {
                        break;
                    }
                }
                if chunks.is_empty() {
                    vm.ctx.empty_str.to_owned()
                } else if chunks.len() == 1 {
                    chunks.pop().unwrap()
                } else {
                    let mut ret = Wtf8Buf::with_capacity(chunks_bytes);
                    for chunk in chunks {
                        ret.push_wtf8(chunk.as_wtf8())
                    }
                    PyStr::from(ret).into_ref(&vm.ctx)
                }
            } else {
                let bytes = vm.call_method(&textio.buffer, "read", ())?;
                let decoded = vm.call_method(&decoder, "decode", (bytes, true))?;
                let decoded = check_decoded(decoded, vm)?;
                let ret = textio.take_decoded_chars(Some(decoded), vm);
                textio.snapshot = None;
                ret
            };
            Ok(s)
        }

        #[pymethod]
        fn write(&self, obj: PyStrRef, vm: &VirtualMachine) -> PyResult<usize> {
            let mut textio = self.lock(vm)?;
            textio.check_closed(vm)?;

            let (encoder, encode_func) = textio
                .encoder
                .as_ref()
                .ok_or_else(|| new_unsupported_operation(vm, "not writable".to_owned()))?;

            let char_len = obj.char_len();

            let data = obj.as_wtf8();

            let replace_nl = match textio.newline {
                Newlines::Lf => Some("\n"),
                Newlines::Cr => Some("\r"),
                Newlines::Crlf => Some("\r\n"),
                Newlines::Universal if cfg!(windows) => Some("\r\n"),
                _ => None,
            };
            let has_lf = (replace_nl.is_some() || textio.line_buffering)
                && data.contains_code_point('\n'.into());
            let flush = textio.line_buffering && (has_lf || data.contains_code_point('\r'.into()));
            let chunk = if let Some(replace_nl) = replace_nl {
                if has_lf {
                    PyStr::from(data.replace("\n".as_ref(), replace_nl.as_ref())).into_ref(&vm.ctx)
                } else {
                    obj
                }
            } else {
                obj
            };
            let chunk = if let Some(encode_func) = *encode_func {
                encode_func(chunk)
            } else {
                let b = vm.call_method(encoder, "encode", (chunk.clone(),))?;
                b.downcast::<PyBytes>()
                    .map(PendingWrite::Bytes)
                    .or_else(|obj| {
                        // TODO: not sure if encode() returning the str it was passed is officially
                        // supported or just a quirk of how the CPython code is written
                        if obj.is(&chunk) {
                            Ok(PendingWrite::Utf8(chunk))
                        } else {
                            Err(vm.new_type_error(format!(
                                "encoder should return a bytes object, not '{}'",
                                obj.class().name()
                            )))
                        }
                    })?
            };
            if textio.pending.num_bytes + chunk.as_bytes().len() > textio.chunk_size {
                textio.write_pending(vm)?;
            }
            textio.pending.push(chunk);
            if flush || textio.write_through || textio.pending.num_bytes >= textio.chunk_size {
                textio.write_pending(vm)?;
            }
            if flush {
                let _ = vm.call_method(&textio.buffer, "flush", ());
            }

            Ok(char_len)
        }

        #[pymethod]
        fn flush(&self, vm: &VirtualMachine) -> PyResult {
            let mut textio = self.lock(vm)?;
            textio.check_closed(vm)?;
            textio.telling = textio.seekable;
            textio.write_pending(vm)?;
            vm.call_method(&textio.buffer, "flush", ())
        }

        #[pymethod]
        fn isatty(&self, vm: &VirtualMachine) -> PyResult {
            let textio = self.lock(vm)?;
            textio.check_closed(vm)?;
            vm.call_method(&textio.buffer, "isatty", ())
        }

        #[pymethod]
        fn readline(&self, size: OptionalSize, vm: &VirtualMachine) -> PyResult<PyStrRef> {
            let limit = size.to_usize();

            let mut textio = self.lock(vm)?;
            check_closed(&textio.buffer, vm)?;

            textio.write_pending(vm)?;

            #[derive(Clone)]
            struct SlicedStr(PyStrRef, Range<usize>);

            impl SlicedStr {
                #[inline]
                fn byte_len(&self) -> usize {
                    self.1.len()
                }

                #[inline]
                fn char_len(&self) -> usize {
                    if self.is_full_slice() {
                        self.0.char_len()
                    } else {
                        self.slice().code_points().count()
                    }
                }

                #[inline]
                fn is_full_slice(&self) -> bool {
                    self.1.len() >= self.0.byte_len()
                }

                #[inline]
                fn slice(&self) -> &Wtf8 {
                    &self.0.as_wtf8()[self.1.clone()]
                }

                #[inline]
                fn slice_pystr(self, vm: &VirtualMachine) -> PyStrRef {
                    if self.is_full_slice() {
                        self.0
                    } else {
                        // TODO: try to use Arc::get_mut() on the str?
                        PyStr::from(self.slice()).into_ref(&vm.ctx)
                    }
                }

                fn utf8_len(&self) -> Utf8size {
                    Utf8size {
                        bytes: self.byte_len(),
                        chars: self.char_len(),
                    }
                }
            }

            let mut start;
            let mut end_pos;
            let mut offset_to_buffer;
            let mut chunked = Utf8size::default();
            let mut remaining: Option<SlicedStr> = None;
            let mut chunks = Vec::new();

            let cur_line = 'outer: loop {
                let decoded_chars = loop {
                    match textio.decoded_chars.as_ref() {
                        Some(s) if !s.is_empty() => break s,
                        _ => {}
                    }
                    let eof = textio.read_chunk(0, vm)?;
                    if eof {
                        textio.set_decoded_chars(None);
                        textio.snapshot = None;
                        start = Utf8size::default();
                        end_pos = Utf8size::default();
                        offset_to_buffer = Utf8size::default();
                        break 'outer None;
                    }
                };
                let line = match remaining.take() {
                    None => {
                        start = textio.decoded_chars_used;
                        offset_to_buffer = Utf8size::default();
                        decoded_chars.clone()
                    }
                    Some(remaining) => {
                        assert_eq!(textio.decoded_chars_used.bytes, 0);
                        offset_to_buffer = remaining.utf8_len();
                        let decoded_chars = decoded_chars.as_wtf8();
                        let line = if remaining.is_full_slice() {
                            let mut line = remaining.0;
                            line.concat_in_place(decoded_chars, vm);
                            line
                        } else {
                            let remaining = remaining.slice();
                            let mut s =
                                Wtf8Buf::with_capacity(remaining.len() + decoded_chars.len());
                            s.push_wtf8(remaining);
                            s.push_wtf8(decoded_chars);
                            PyStr::from(s).into_ref(&vm.ctx)
                        };
                        start = Utf8size::default();
                        line
                    }
                };
                let line_from_start = &line.as_wtf8()[start.bytes..];
                let nl_res = textio.newline.find_newline(line_from_start);
                match nl_res {
                    Ok(p) | Err(p) => {
                        end_pos = start + Utf8size::len_str(&line_from_start[..p]);
                        if let Some(limit) = limit {
                            // original CPython logic: end_pos = start + limit - chunked
                            if chunked.chars + end_pos.chars >= limit {
                                end_pos = start
                                    + Utf8size {
                                        chars: limit - chunked.chars,
                                        bytes: crate::common::str::codepoint_range_end(
                                            line_from_start,
                                            limit - chunked.chars,
                                        )
                                        .unwrap(),
                                    };
                                break Some(line);
                            }
                        }
                    }
                }
                if nl_res.is_ok() {
                    break Some(line);
                }
                if end_pos.bytes > start.bytes {
                    let chunk = SlicedStr(line.clone(), start.bytes..end_pos.bytes);
                    chunked += chunk.utf8_len();
                    chunks.push(chunk);
                }
                let line_len = line.byte_len();
                if end_pos.bytes < line_len {
                    remaining = Some(SlicedStr(line, end_pos.bytes..line_len));
                }
                textio.set_decoded_chars(None);
            };

            let cur_line = cur_line.map(|line| {
                textio.decoded_chars_used = end_pos - offset_to_buffer;
                SlicedStr(line, start.bytes..end_pos.bytes)
            });
            // don't need to care about chunked.chars anymore
            let mut chunked = chunked.bytes;
            if let Some(remaining) = remaining {
                chunked += remaining.byte_len();
                chunks.push(remaining);
            }
            let line = if !chunks.is_empty() {
                if let Some(cur_line) = cur_line {
                    chunked += cur_line.byte_len();
                    chunks.push(cur_line);
                }
                let mut s = Wtf8Buf::with_capacity(chunked);
                for chunk in chunks {
                    s.push_wtf8(chunk.slice())
                }
                PyStr::from(s).into_ref(&vm.ctx)
            } else if let Some(cur_line) = cur_line {
                cur_line.slice_pystr(vm)
            } else {
                vm.ctx.empty_str.to_owned()
            };
            Ok(line)
        }

        #[pymethod]
        fn close(zelf: PyRef<Self>, vm: &VirtualMachine) -> PyResult<()> {
            let buffer = zelf.lock(vm)?.buffer.clone();
            if file_closed(&buffer, vm)? {
                return Ok(());
            }
            let flush_res = vm.call_method(zelf.as_object(), "flush", ()).map(drop);
            let close_res = vm.call_method(&buffer, "close", ()).map(drop);
            exception_chain(flush_res, close_res)
        }

        #[pygetset]
        fn closed(&self, vm: &VirtualMachine) -> PyResult {
            let buffer = self.lock(vm)?.buffer.clone();
            buffer.get_attr("closed", vm)
        }

        #[pygetset]
        fn buffer(&self, vm: &VirtualMachine) -> PyResult {
            Ok(self.lock(vm)?.buffer.clone())
        }

        #[pymethod]
        fn __reduce__(zelf: PyObjectRef, vm: &VirtualMachine) -> PyResult {
            Err(vm.new_type_error(format!("cannot pickle '{}' object", zelf.class().name())))
        }
    }

    fn parse_decoder_state(state: PyObjectRef, vm: &VirtualMachine) -> PyResult<(PyBytesRef, i32)> {
        use crate::builtins::{PyTuple, int};
        let state_err = || vm.new_type_error("illegal decoder state");
        let state = state.downcast::<PyTuple>().map_err(|_| state_err())?;
        match state.as_slice() {
            [buf, flags] => {
                let buf = buf.clone().downcast::<PyBytes>().map_err(|obj| {
                    vm.new_type_error(format!(
                        "illegal decoder state: the first item should be a bytes object, not '{}'",
                        obj.class().name()
                    ))
                })?;
                let flags = flags.downcast_ref::<int::PyInt>().ok_or_else(state_err)?;
                let flags = flags.try_to_primitive(vm)?;
                Ok((buf, flags))
            }
            _ => Err(state_err()),
        }
    }

    impl TextIOData {
        fn write_pending(&mut self, vm: &VirtualMachine) -> PyResult<()> {
            if self.pending.num_bytes == 0 {
                return Ok(());
            }
            let data = self.pending.take(vm);
            vm.call_method(&self.buffer, "write", (data,))?;
            Ok(())
        }

        /// returns true on EOF
        fn read_chunk(&mut self, size_hint: usize, vm: &VirtualMachine) -> PyResult<bool> {
            let decoder = self
                .decoder
                .as_ref()
                .ok_or_else(|| new_unsupported_operation(vm, "not readable".to_owned()))?;

            let dec_state = if self.telling {
                let state = vm.call_method(decoder, "getstate", ())?;
                Some(parse_decoder_state(state, vm)?)
            } else {
                None
            };

            let method = if self.has_read1 { "read1" } else { "read" };
            let size_hint = if size_hint > 0 {
                (self.b2cratio.max(1.0) * size_hint as f64) as usize
            } else {
                size_hint
            };
            let chunk_size = std::cmp::max(self.chunk_size, size_hint);
            let input_chunk = vm.call_method(&self.buffer, method, (chunk_size,))?;

            let buf = ArgBytesLike::try_from_borrowed_object(vm, &input_chunk).map_err(|_| {
                vm.new_type_error(format!(
                    "underlying {}() should have returned a bytes-like object, not '{}'",
                    method,
                    input_chunk.class().name()
                ))
            })?;
            let nbytes = buf.borrow_buf().len();
            let eof = nbytes == 0;
            let decoded = vm.call_method(decoder, "decode", (input_chunk, eof))?;
            let decoded = check_decoded(decoded, vm)?;

            let char_len = decoded.char_len();
            self.b2cratio = if char_len > 0 {
                nbytes as f64 / char_len as f64
            } else {
                0.0
            };
            let eof = if char_len > 0 { false } else { eof };
            self.set_decoded_chars(Some(decoded));

            if let Some((dec_buffer, dec_flags)) = dec_state {
                // TODO: inplace append to bytes when refcount == 1
                let mut next_input = dec_buffer.as_bytes().to_vec();
                next_input.extend_from_slice(&buf.borrow_buf());
                self.snapshot = Some((dec_flags, PyBytes::from(next_input).into_ref(&vm.ctx)));
            }

            Ok(eof)
        }

        fn check_closed(&self, vm: &VirtualMachine) -> PyResult<()> {
            check_closed(&self.buffer, vm)
        }

        /// returns str, str.char_len() (it might not be cached in the str yet but we calculate it
        /// anyway in this method)
        fn get_decoded_chars(
            &mut self,
            n: usize,
            vm: &VirtualMachine,
        ) -> Option<(PyStrRef, usize)> {
            if n == 0 {
                return None;
            }
            let decoded_chars = self.decoded_chars.as_ref()?;
            let avail = &decoded_chars.as_wtf8()[self.decoded_chars_used.bytes..];
            if avail.is_empty() {
                return None;
            }
            let avail_chars = decoded_chars.char_len() - self.decoded_chars_used.chars;
            let (chars, chars_used) = if n >= avail_chars {
                if self.decoded_chars_used.bytes == 0 {
                    (decoded_chars.clone(), avail_chars)
                } else {
                    (PyStr::from(avail).into_ref(&vm.ctx), avail_chars)
                }
            } else {
                let s = crate::common::str::get_codepoints(avail, 0..n);
                (PyStr::from(s).into_ref(&vm.ctx), n)
            };
            self.decoded_chars_used += Utf8size {
                bytes: chars.byte_len(),
                chars: chars_used,
            };
            Some((chars, chars_used))
        }

        fn set_decoded_chars(&mut self, s: Option<PyStrRef>) {
            self.decoded_chars = s;
            self.decoded_chars_used = Utf8size::default();
        }

        fn take_decoded_chars(
            &mut self,
            append: Option<PyStrRef>,
            vm: &VirtualMachine,
        ) -> PyStrRef {
            let empty_str = || vm.ctx.empty_str.to_owned();
            let chars_pos = std::mem::take(&mut self.decoded_chars_used).bytes;
            let decoded_chars = match std::mem::take(&mut self.decoded_chars) {
                None => return append.unwrap_or_else(empty_str),
                Some(s) if s.is_empty() => return append.unwrap_or_else(empty_str),
                Some(s) => s,
            };
            let append_len = append.as_ref().map_or(0, |s| s.byte_len());
            if append_len == 0 && chars_pos == 0 {
                return decoded_chars;
            }
            // TODO: in-place editing of `str` when refcount == 1
            let decoded_chars_unused = &decoded_chars.as_wtf8()[chars_pos..];
            let mut s = Wtf8Buf::with_capacity(decoded_chars_unused.len() + append_len);
            s.push_wtf8(decoded_chars_unused);
            if let Some(append) = append {
                s.push_wtf8(append.as_wtf8())
            }
            PyStr::from(s).into_ref(&vm.ctx)
        }
    }

    #[pyattr]
    #[pyclass(name)]
    #[derive(Debug, PyPayload, Default)]
    struct IncrementalNewlineDecoder {
        // TODO: Traverse
        data: PyThreadMutex<Option<IncrementalNewlineDecoderData>>,
    }

    #[derive(Debug)]
    struct IncrementalNewlineDecoderData {
        decoder: PyObjectRef,
        // currently this is used for nothing
        // errors: PyObjectRef,
        pendingcr: bool,
        translate: bool,
        seennl: SeenNewline,
    }

    bitflags! {
        #[derive(Debug, PartialEq, Eq, Copy, Clone)]
        struct SeenNewline: u8 {
            const LF = 1;
            const CR = 2;
            const CRLF = 4;
        }
    }

    impl DefaultConstructor for IncrementalNewlineDecoder {}

    #[derive(FromArgs)]
    struct IncrementalNewlineDecoderArgs {
        #[pyarg(any)]
        decoder: PyObjectRef,
        #[pyarg(any)]
        translate: bool,
        #[pyarg(any, default)]
        errors: Option<PyObjectRef>,
    }

    impl Initializer for IncrementalNewlineDecoder {
        type Args = IncrementalNewlineDecoderArgs;
        fn init(zelf: PyRef<Self>, args: Self::Args, vm: &VirtualMachine) -> PyResult<()> {
            let _ = args.errors;
            let mut data = zelf.lock_opt(vm)?;
            *data = Some(IncrementalNewlineDecoderData {
                decoder: args.decoder,
                translate: args.translate,
                pendingcr: false,
                seennl: SeenNewline::empty(),
            });
            Ok(())
        }
    }

    #[pyclass(with(Constructor, Initializer))]
    impl IncrementalNewlineDecoder {
        fn lock_opt(
            &self,
            vm: &VirtualMachine,
        ) -> PyResult<PyThreadMutexGuard<'_, Option<IncrementalNewlineDecoderData>>> {
            self.data
                .lock()
                .ok_or_else(|| vm.new_runtime_error("reentrant call inside nldecoder"))
        }

        fn lock(
            &self,
            vm: &VirtualMachine,
        ) -> PyResult<PyMappedThreadMutexGuard<'_, IncrementalNewlineDecoderData>> {
            let lock = self.lock_opt(vm)?;
            PyThreadMutexGuard::try_map(lock, |x| x.as_mut())
                .map_err(|_| vm.new_value_error("I/O operation on uninitialized nldecoder"))
        }

        #[pymethod]
        fn decode(&self, args: NewlineDecodeArgs, vm: &VirtualMachine) -> PyResult<PyStrRef> {
            self.lock(vm)?.decode(args.input, args.r#final, vm)
        }

        #[pymethod]
        fn getstate(&self, vm: &VirtualMachine) -> PyResult<(PyObjectRef, u64)> {
            let data = self.lock(vm)?;
            let (buffer, flag) = if vm.is_none(&data.decoder) {
                (vm.ctx.new_bytes(vec![]).into(), 0)
            } else {
                vm.call_method(&data.decoder, "getstate", ())?
                    .try_to_ref::<PyTuple>(vm)?
                    .extract_tuple::<(PyObjectRef, u64)>(vm)?
            };
            let flag = (flag << 1) | (data.pendingcr as u64);
            Ok((buffer, flag))
        }

        #[pymethod]
        fn setstate(&self, state: PyTupleRef, vm: &VirtualMachine) -> PyResult<()> {
            let mut data = self.lock(vm)?;
            let (buffer, flag) = state.extract_tuple::<(PyObjectRef, u64)>(vm)?;
            data.pendingcr = flag & 1 != 0;
            if !vm.is_none(&data.decoder) {
                vm.call_method(&data.decoder, "setstate", ((buffer, flag >> 1),))?;
            }
            Ok(())
        }

        #[pymethod]
        fn reset(&self, vm: &VirtualMachine) -> PyResult<()> {
            let mut data = self.lock(vm)?;
            data.seennl = SeenNewline::empty();
            data.pendingcr = false;
            if !vm.is_none(&data.decoder) {
                vm.call_method(&data.decoder, "reset", ())?;
            }
            Ok(())
        }

        #[pygetset]
        fn newlines(&self, vm: &VirtualMachine) -> PyResult {
            let data = self.lock(vm)?;
            Ok(match data.seennl.bits() {
                1 => "\n".to_pyobject(vm),
                2 => "\r".to_pyobject(vm),
                3 => ("\r", "\n").to_pyobject(vm),
                4 => "\r\n".to_pyobject(vm),
                5 => ("\n", "\r\n").to_pyobject(vm),
                6 => ("\r", "\r\n").to_pyobject(vm),
                7 => ("\r", "\n", "\r\n").to_pyobject(vm),
                _ => vm.ctx.none(),
            })
        }
    }

    #[derive(FromArgs)]
    struct NewlineDecodeArgs {
        #[pyarg(any)]
        input: PyObjectRef,
        #[pyarg(any, default)]
        r#final: bool,
    }

    impl IncrementalNewlineDecoderData {
        fn decode(
            &mut self,
            input: PyObjectRef,
            final_: bool,
            vm: &VirtualMachine,
        ) -> PyResult<PyStrRef> {
            let output = if vm.is_none(&self.decoder) {
                input
            } else {
                vm.call_method(&self.decoder, "decode", (input, final_))?
            };
            let orig_output: PyStrRef = output.try_into_value(vm)?;
            // this being Cow::Owned means we need to allocate a new string
            let mut output = Cow::Borrowed(orig_output.as_wtf8());
            if self.pendingcr && (final_ || !output.is_empty()) {
                output.to_mut().insert(0, '\r'.into());
                self.pendingcr = false;
            }
            if !final_ {
                if let Some(s) = output.strip_suffix("\r".as_ref()) {
                    output = Cow::Owned(s.to_owned());
                    self.pendingcr = true;
                }
            }

            if output.is_empty() {
                return Ok(vm.ctx.empty_str.to_owned());
            }

            if (self.seennl == SeenNewline::LF || self.seennl.is_empty())
                && !output.contains_code_point('\r'.into())
            {
                if self.seennl.is_empty() && output.contains_code_point('\n'.into()) {
                    self.seennl.insert(SeenNewline::LF);
                }
            } else if !self.translate {
                let output = output.as_bytes();
                let mut matches = memchr::memchr2_iter(b'\r', b'\n', output);
                while !self.seennl.is_all() {
                    let Some(i) = matches.next() else { break };
                    match output[i] {
                        b'\n' => self.seennl.insert(SeenNewline::LF),
                        // if c isn't \n, it can only be \r
                        _ if output.get(i + 1) == Some(&b'\n') => {
                            matches.next();
                            self.seennl.insert(SeenNewline::CRLF);
                        }
                        _ => self.seennl.insert(SeenNewline::CR),
                    }
                }
            } else {
                let bytes = output.as_bytes();
                let mut matches = memchr::memchr2_iter(b'\r', b'\n', bytes);
                let mut new_string = Wtf8Buf::with_capacity(output.len());
                let mut last_modification_index = 0;
                while let Some(cr_index) = matches.next() {
                    if bytes[cr_index] == b'\r' {
                        // skip copying the CR
                        let mut next_chunk_index = cr_index + 1;
                        if bytes.get(cr_index + 1) == Some(&b'\n') {
                            matches.next();
                            self.seennl.insert(SeenNewline::CRLF);
                            // skip the LF too
                            next_chunk_index += 1;
                        } else {
                            self.seennl.insert(SeenNewline::CR);
                        }
                        new_string.push_wtf8(&output[last_modification_index..cr_index]);
                        new_string.push_char('\n');
                        last_modification_index = next_chunk_index;
                    } else {
                        self.seennl.insert(SeenNewline::LF);
                    }
                }
                new_string.push_wtf8(&output[last_modification_index..]);
                output = Cow::Owned(new_string);
            }

            Ok(match output {
                Cow::Borrowed(_) => orig_output,
                Cow::Owned(s) => vm.ctx.new_str(s),
            })
        }
    }

    #[pyattr]
    #[pyclass(name = "StringIO", base = "_TextIOBase")]
    #[derive(Debug, PyPayload)]
    struct StringIO {
        buffer: PyRwLock<BufferedIO>,
        closed: AtomicCell<bool>,
    }

    #[derive(FromArgs)]
    struct StringIONewArgs {
        #[pyarg(positional, optional)]
        object: OptionalOption<PyStrRef>,

        // TODO: use this
        #[pyarg(any, default)]
        #[allow(dead_code)]
        newline: Newlines,
    }

    impl Constructor for StringIO {
        type Args = StringIONewArgs;

        #[allow(unused_variables)]
        fn py_new(
            cls: PyTypeRef,
            Self::Args { object, newline }: Self::Args,
            vm: &VirtualMachine,
        ) -> PyResult {
            let raw_bytes = object
                .flatten()
                .map_or_else(Vec::new, |v| v.as_bytes().to_vec());

            Self {
                buffer: PyRwLock::new(BufferedIO::new(Cursor::new(raw_bytes))),
                closed: AtomicCell::new(false),
            }
            .into_ref_with_type(vm, cls)
            .map(Into::into)
        }
    }

    impl StringIO {
        fn buffer(&self, vm: &VirtualMachine) -> PyResult<PyRwLockWriteGuard<'_, BufferedIO>> {
            if !self.closed.load() {
                Ok(self.buffer.write())
            } else {
                Err(io_closed_error(vm))
            }
        }
    }

    #[pyclass(flags(BASETYPE, HAS_DICT), with(Constructor))]
    impl StringIO {
        #[pymethod]
        const fn readable(&self) -> bool {
            true
        }

        #[pymethod]
        const fn writable(&self) -> bool {
            true
        }

        #[pymethod]
        const fn seekable(&self) -> bool {
            true
        }

        #[pygetset]
        fn closed(&self) -> bool {
            self.closed.load()
        }

        #[pymethod]
        fn close(&self) {
            self.closed.store(true);
        }

        // write string to underlying vector
        #[pymethod]
        fn write(&self, data: PyStrRef, vm: &VirtualMachine) -> PyResult<u64> {
            let bytes = data.as_bytes();
            self.buffer(vm)?
                .write(bytes)
                .ok_or_else(|| vm.new_type_error("Error Writing String"))
        }

        // return the entire contents of the underlying
        #[pymethod]
        fn getvalue(&self, vm: &VirtualMachine) -> PyResult<Wtf8Buf> {
            let bytes = self.buffer(vm)?.getvalue();
            Wtf8Buf::from_bytes(bytes).map_err(|_| vm.new_value_error("Error Retrieving Value"))
        }

        // skip to the jth position
        #[pymethod]
        fn seek(
            &self,
            offset: PyObjectRef,
            how: OptionalArg<i32>,
            vm: &VirtualMachine,
        ) -> PyResult<u64> {
            self.buffer(vm)?
                .seek(seekfrom(vm, offset, how)?)
                .map_err(|err| os_err(vm, err))
        }

        // Read k bytes from the object and return.
        // If k is undefined || k == -1, then we read all bytes until the end of the file.
        // This also increments the stream position by the value of k
        #[pymethod]
        fn read(&self, size: OptionalSize, vm: &VirtualMachine) -> PyResult<Wtf8Buf> {
            let data = self.buffer(vm)?.read(size.to_usize()).unwrap_or_default();

            let value = Wtf8Buf::from_bytes(data)
                .map_err(|_| vm.new_value_error("Error Retrieving Value"))?;
            Ok(value)
        }

        #[pymethod]
        fn tell(&self, vm: &VirtualMachine) -> PyResult<u64> {
            Ok(self.buffer(vm)?.tell())
        }

        #[pymethod]
        fn readline(&self, size: OptionalSize, vm: &VirtualMachine) -> PyResult<Wtf8Buf> {
            // TODO size should correspond to the number of characters, at the moments its the number of
            // bytes.
            let input = self.buffer(vm)?.readline(size.to_usize(), vm)?;
            Wtf8Buf::from_bytes(input).map_err(|_| vm.new_value_error("Error Retrieving Value"))
        }

        #[pymethod]
        fn truncate(&self, pos: OptionalSize, vm: &VirtualMachine) -> PyResult<usize> {
            let mut buffer = self.buffer(vm)?;
            let pos = pos.try_usize(vm)?;
            Ok(buffer.truncate(pos))
        }

        #[pygetset]
        const fn line_buffering(&self) -> bool {
            false
        }
    }

    #[pyattr]
    #[pyclass(name = "BytesIO", base = "_BufferedIOBase")]
    #[derive(Debug, PyPayload)]
    struct BytesIO {
        buffer: PyRwLock<BufferedIO>,
        closed: AtomicCell<bool>,
        exports: AtomicCell<usize>,
    }

    impl Constructor for BytesIO {
        type Args = OptionalArg<Option<PyBytesRef>>;

        fn py_new(cls: PyTypeRef, object: Self::Args, vm: &VirtualMachine) -> PyResult {
            let raw_bytes = object
                .flatten()
                .map_or_else(Vec::new, |input| input.as_bytes().to_vec());

            Self {
                buffer: PyRwLock::new(BufferedIO::new(Cursor::new(raw_bytes))),
                closed: AtomicCell::new(false),
                exports: AtomicCell::new(0),
            }
            .into_ref_with_type(vm, cls)
            .map(Into::into)
        }
    }

    impl BytesIO {
        fn buffer(&self, vm: &VirtualMachine) -> PyResult<PyRwLockWriteGuard<'_, BufferedIO>> {
            if !self.closed.load() {
                Ok(self.buffer.write())
            } else {
                Err(io_closed_error(vm))
            }
        }
    }

    #[pyclass(flags(BASETYPE, HAS_DICT), with(PyRef, Constructor))]
    impl BytesIO {
        #[pymethod]
        const fn readable(&self) -> bool {
            true
        }

        #[pymethod]
        const fn writable(&self) -> bool {
            true
        }

        #[pymethod]
        const fn seekable(&self) -> bool {
            true
        }

        #[pymethod]
        fn write(&self, data: ArgBytesLike, vm: &VirtualMachine) -> PyResult<u64> {
            let mut buffer = self.try_resizable(vm)?;
            data.with_ref(|b| buffer.write(b))
                .ok_or_else(|| vm.new_type_error("Error Writing Bytes"))
        }

        // Retrieves the entire bytes object value from the underlying buffer
        #[pymethod]
        fn getvalue(&self, vm: &VirtualMachine) -> PyResult<PyBytesRef> {
            let bytes = self.buffer(vm)?.getvalue();
            Ok(vm.ctx.new_bytes(bytes))
        }

        // Takes an integer k (bytes) and returns them from the underlying buffer
        // If k is undefined || k == -1, then we read all bytes until the end of the file.
        // This also increments the stream position by the value of k
        #[pymethod]
        #[pymethod(name = "read1")]
        fn read(&self, size: OptionalSize, vm: &VirtualMachine) -> PyResult<Vec<u8>> {
            let buf = self.buffer(vm)?.read(size.to_usize()).unwrap_or_default();
            Ok(buf)
        }

        #[pymethod]
        fn readinto(&self, obj: ArgMemoryBuffer, vm: &VirtualMachine) -> PyResult<usize> {
            let mut buf = self.buffer(vm)?;
            let ret = buf
                .cursor
                .read(&mut obj.borrow_buf_mut())
                .map_err(|_| vm.new_value_error("Error readinto from Take"))?;

            Ok(ret)
        }

        //skip to the jth position
        #[pymethod]
        fn seek(
            &self,
            offset: PyObjectRef,
            how: OptionalArg<i32>,
            vm: &VirtualMachine,
        ) -> PyResult<u64> {
            self.buffer(vm)?
                .seek(seekfrom(vm, offset, how)?)
                .map_err(|err| os_err(vm, err))
        }

        #[pymethod]
        fn tell(&self, vm: &VirtualMachine) -> PyResult<u64> {
            Ok(self.buffer(vm)?.tell())
        }

        #[pymethod]
        fn readline(&self, size: OptionalSize, vm: &VirtualMachine) -> PyResult<Vec<u8>> {
            self.buffer(vm)?.readline(size.to_usize(), vm)
        }

        #[pymethod]
        fn truncate(&self, pos: OptionalSize, vm: &VirtualMachine) -> PyResult<usize> {
            if self.closed.load() {
                return Err(io_closed_error(vm));
            }
            let mut buffer = self.try_resizable(vm)?;
            let pos = pos.try_usize(vm)?;
            Ok(buffer.truncate(pos))
        }

        #[pygetset]
        fn closed(&self) -> bool {
            self.closed.load()
        }

        #[pymethod]
        fn close(&self, vm: &VirtualMachine) -> PyResult<()> {
            drop(self.try_resizable(vm)?);
            self.closed.store(true);
            Ok(())
        }
    }

    #[pyclass]
    impl PyRef<BytesIO> {
        #[pymethod]
        fn getbuffer(self, vm: &VirtualMachine) -> PyResult<PyMemoryView> {
            let len = self.buffer.read().cursor.get_ref().len();
            let buffer = PyBuffer::new(
                self.into(),
                BufferDescriptor::simple(len, false),
                &BYTES_IO_BUFFER_METHODS,
            );
            let view = PyMemoryView::from_buffer(buffer, vm)?;
            Ok(view)
        }
    }

    static BYTES_IO_BUFFER_METHODS: BufferMethods = BufferMethods {
        obj_bytes: |buffer| {
            let zelf = buffer.obj_as::<BytesIO>();
            PyRwLockReadGuard::map(zelf.buffer.read(), |x| x.cursor.get_ref().as_slice()).into()
        },
        obj_bytes_mut: |buffer| {
            let zelf = buffer.obj_as::<BytesIO>();
            PyRwLockWriteGuard::map(zelf.buffer.write(), |x| x.cursor.get_mut().as_mut_slice())
                .into()
        },

        release: |buffer| {
            buffer.obj_as::<BytesIO>().exports.fetch_sub(1);
        },

        retain: |buffer| {
            buffer.obj_as::<BytesIO>().exports.fetch_add(1);
        },
    };

    impl BufferResizeGuard for BytesIO {
        type Resizable<'a> = PyRwLockWriteGuard<'a, BufferedIO>;

        fn try_resizable_opt(&self) -> Option<Self::Resizable<'_>> {
            let w = self.buffer.write();
            (self.exports.load() == 0).then_some(w)
        }
    }

    #[repr(u8)]
    #[derive(Debug)]
    enum FileMode {
        Read = b'r',
        Write = b'w',
        Exclusive = b'x',
        Append = b'a',
    }

    #[repr(u8)]
    #[derive(Debug)]
    enum EncodeMode {
        Text = b't',
        Bytes = b'b',
    }

    #[derive(Debug)]
    struct Mode {
        file: FileMode,
        encode: EncodeMode,
        plus: bool,
    }

    impl std::str::FromStr for Mode {
        type Err = ParseModeError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut file = None;
            let mut encode = None;
            let mut plus = false;
            macro_rules! set_mode {
                ($var:ident, $mode:path, $err:ident) => {{
                    match $var {
                        Some($mode) => return Err(ParseModeError::InvalidMode),
                        Some(_) => return Err(ParseModeError::$err),
                        None => $var = Some($mode),
                    }
                }};
            }

            for ch in s.chars() {
                match ch {
                    '+' => {
                        if plus {
                            return Err(ParseModeError::InvalidMode);
                        }
                        plus = true
                    }
                    't' => set_mode!(encode, EncodeMode::Text, MultipleEncode),
                    'b' => set_mode!(encode, EncodeMode::Bytes, MultipleEncode),
                    'r' => set_mode!(file, FileMode::Read, MultipleFile),
                    'a' => set_mode!(file, FileMode::Append, MultipleFile),
                    'w' => set_mode!(file, FileMode::Write, MultipleFile),
                    'x' => set_mode!(file, FileMode::Exclusive, MultipleFile),
                    _ => return Err(ParseModeError::InvalidMode),
                }
            }

            let file = file.ok_or(ParseModeError::NoFile)?;
            let encode = encode.unwrap_or(EncodeMode::Text);

            Ok(Self { file, encode, plus })
        }
    }

    impl Mode {
        const fn rawmode(&self) -> &'static str {
            match (&self.file, self.plus) {
                (FileMode::Read, true) => "rb+",
                (FileMode::Read, false) => "rb",
                (FileMode::Write, true) => "wb+",
                (FileMode::Write, false) => "wb",
                (FileMode::Exclusive, true) => "xb+",
                (FileMode::Exclusive, false) => "xb",
                (FileMode::Append, true) => "ab+",
                (FileMode::Append, false) => "ab",
            }
        }
    }

    enum ParseModeError {
        InvalidMode,
        MultipleFile,
        MultipleEncode,
        NoFile,
    }

    impl ParseModeError {
        fn error_msg(&self, mode_string: &str) -> String {
            match self {
                Self::InvalidMode => format!("invalid mode: '{mode_string}'"),
                Self::MultipleFile => {
                    "must have exactly one of create/read/write/append mode".to_owned()
                }
                Self::MultipleEncode => "can't have text and binary mode at once".to_owned(),
                Self::NoFile => {
                    "Must have exactly one of create/read/write/append mode and at most one plus"
                        .to_owned()
                }
            }
        }
    }

    #[derive(FromArgs)]
    struct IoOpenArgs {
        file: PyObjectRef,
        #[pyarg(any, optional)]
        mode: OptionalArg<PyUtf8StrRef>,
        #[pyarg(flatten)]
        opts: OpenArgs,
    }

    #[pyfunction]
    fn open(args: IoOpenArgs, vm: &VirtualMachine) -> PyResult {
        io_open(
            args.file,
            args.mode.as_ref().into_option().map(|s| s.as_str()),
            args.opts,
            vm,
        )
    }

    #[pyfunction]
    fn open_code(file: PyObjectRef, vm: &VirtualMachine) -> PyResult {
        // TODO: lifecycle hooks or something?
        io_open(file, Some("rb"), OpenArgs::default(), vm)
    }

    #[derive(FromArgs)]
    pub struct OpenArgs {
        #[pyarg(any, default = -1)]
        pub buffering: isize,
        #[pyarg(any, default)]
        pub encoding: Option<PyUtf8StrRef>,
        #[pyarg(any, default)]
        pub errors: Option<PyStrRef>,
        #[pyarg(any, default)]
        pub newline: Option<PyStrRef>,
        #[pyarg(any, default = true)]
        pub closefd: bool,
        #[pyarg(any, default)]
        pub opener: Option<PyObjectRef>,
    }

    impl Default for OpenArgs {
        fn default() -> Self {
            Self {
                buffering: -1,
                encoding: None,
                errors: None,
                newline: None,
                closefd: true,
                opener: None,
            }
        }
    }

    pub fn io_open(
        file: PyObjectRef,
        mode: Option<&str>,
        opts: OpenArgs,
        vm: &VirtualMachine,
    ) -> PyResult {
        // mode is optional: 'rt' is the default mode (open from reading text)
        let mode_string = mode.unwrap_or("r");
        let mode = mode_string
            .parse::<Mode>()
            .map_err(|e| vm.new_value_error(e.error_msg(mode_string)))?;

        if let EncodeMode::Bytes = mode.encode {
            let msg = if opts.encoding.is_some() {
                Some("binary mode doesn't take an encoding argument")
            } else if opts.errors.is_some() {
                Some("binary mode doesn't take an errors argument")
            } else if opts.newline.is_some() {
                Some("binary mode doesn't take a newline argument")
            } else {
                None
            };
            if let Some(msg) = msg {
                return Err(vm.new_value_error(msg));
            }
        }

        // check file descriptor validity
        #[cfg(unix)]
        if let Ok(crate::ospath::OsPathOrFd::Fd(fd)) = file.clone().try_into_value(vm) {
            nix::fcntl::fcntl(fd, nix::fcntl::F_GETFD)
                .map_err(|_| crate::stdlib::os::errno_err(vm))?;
        }

        // Construct a FileIO (subclass of RawIOBase)
        // This is subsequently consumed by a Buffered Class.
        let file_io_class: &Py<PyType> = {
            cfg_if::cfg_if! {
                if #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))] {
                    Some(super::fileio::FileIO::static_type())
                } else {
                    None
                }
            }
        }
        .ok_or_else(|| {
            new_unsupported_operation(
                vm,
                "Couldn't get FileIO, io.open likely isn't supported on your platform".to_owned(),
            )
        })?;
        let raw = PyType::call(
            file_io_class,
            (file, mode.rawmode(), opts.closefd, opts.opener).into_args(vm),
            vm,
        )?;

        let isatty = opts.buffering < 0 && {
            let atty = vm.call_method(&raw, "isatty", ())?;
            bool::try_from_object(vm, atty)?
        };

        let line_buffering = opts.buffering == 1 || isatty;

        let buffering = if opts.buffering < 0 || opts.buffering == 1 {
            DEFAULT_BUFFER_SIZE
        } else {
            opts.buffering as usize
        };

        if buffering == 0 {
            let ret = match mode.encode {
                EncodeMode::Text => Err(vm.new_value_error("can't have unbuffered text I/O")),
                EncodeMode::Bytes => Ok(raw),
            };
            return ret;
        }

        let cls = if mode.plus {
            BufferedRandom::static_type()
        } else if let FileMode::Read = mode.file {
            BufferedReader::static_type()
        } else {
            BufferedWriter::static_type()
        };
        let buffered = PyType::call(cls, (raw, buffering).into_args(vm), vm)?;

        match mode.encode {
            EncodeMode::Text => {
                let tio = TextIOWrapper::static_type();
                let wrapper = PyType::call(
                    tio,
                    (
                        buffered,
                        opts.encoding,
                        opts.errors,
                        opts.newline,
                        line_buffering,
                    )
                        .into_args(vm),
                    vm,
                )?;
                wrapper.set_attr("mode", vm.new_pyobj(mode_string), vm)?;
                Ok(wrapper)
            }
            EncodeMode::Bytes => Ok(buffered),
        }
    }

    rustpython_common::static_cell! {
        pub(super) static UNSUPPORTED_OPERATION: PyTypeRef;
    }

    pub(super) fn make_unsupportedop(ctx: &Context) -> PyTypeRef {
        use crate::types::PyTypeSlots;
        PyType::new_heap(
            "UnsupportedOperation",
            vec![
                ctx.exceptions.os_error.to_owned(),
                ctx.exceptions.value_error.to_owned(),
            ],
            Default::default(),
            PyTypeSlots::heap_default(),
            ctx.types.type_type.to_owned(),
            ctx,
        )
        .unwrap()
    }

    #[pyfunction]
    fn text_encoding(
        encoding: PyObjectRef,
        _stacklevel: OptionalArg<i32>,
        vm: &VirtualMachine,
    ) -> PyResult<PyStrRef> {
        if vm.is_none(&encoding) {
            // TODO: This is `locale` encoding - but we don't have locale encoding yet
            return Ok(vm.ctx.new_str("utf-8"));
        }
        encoding.try_into_value(vm)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_buffered_read() {
            let data = vec![1, 2, 3, 4];
            let bytes = None;
            let mut buffered = BufferedIO {
                cursor: Cursor::new(data.clone()),
            };

            assert_eq!(buffered.read(bytes).unwrap(), data);
        }

        #[test]
        fn test_buffered_seek() {
            let data = vec![1, 2, 3, 4];
            let count: u64 = 2;
            let mut buffered = BufferedIO {
                cursor: Cursor::new(data),
            };

            assert_eq!(buffered.seek(SeekFrom::Start(count)).unwrap(), count);
            assert_eq!(buffered.read(Some(count as usize)).unwrap(), vec![3, 4]);
        }

        #[test]
        fn test_buffered_value() {
            let data = vec![1, 2, 3, 4];
            let buffered = BufferedIO {
                cursor: Cursor::new(data.clone()),
            };

            assert_eq!(buffered.getvalue(), data);
        }
    }
}

// disable FileIO on WASM
#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
#[pymodule]
mod fileio {
    use super::{_io::*, Offset};
    use crate::{
        AsObject, Py, PyObjectRef, PyPayload, PyRef, PyResult, TryFromObject, VirtualMachine,
        builtins::{PyBaseExceptionRef, PyUtf8Str, PyUtf8StrRef},
        common::crt_fd::Fd,
        convert::ToPyException,
        function::{ArgBytesLike, ArgMemoryBuffer, OptionalArg, OptionalOption},
        ospath::{IOErrorBuilder, OsPath, OsPathOrFd},
        stdlib::os,
        types::{Constructor, DefaultConstructor, Initializer, Representable},
    };
    use crossbeam_utils::atomic::AtomicCell;
    use std::io::{Read, Write};

    bitflags::bitflags! {
        #[derive(Copy, Clone, Debug, PartialEq)]
        struct Mode: u8 {
            const CREATED   = 0b0001;
            const READABLE  = 0b0010;
            const WRITABLE  = 0b0100;
            const APPENDING = 0b1000;
        }
    }

    enum ModeError {
        Invalid,
        BadRwa,
    }

    impl ModeError {
        fn error_msg(&self, mode_str: &str) -> String {
            match self {
                Self::Invalid => format!("invalid mode: {mode_str}"),
                Self::BadRwa => {
                    "Must have exactly one of create/read/write/append mode and at most one plus"
                        .to_owned()
                }
            }
        }
    }

    fn compute_mode(mode_str: &str) -> Result<(Mode, i32), ModeError> {
        let mut flags = 0;
        let mut plus = false;
        let mut rwa = false;
        let mut mode = Mode::empty();
        for c in mode_str.bytes() {
            match c {
                b'x' => {
                    if rwa {
                        return Err(ModeError::BadRwa);
                    }
                    rwa = true;
                    mode.insert(Mode::WRITABLE | Mode::CREATED);
                    flags |= libc::O_EXCL | libc::O_CREAT;
                }
                b'r' => {
                    if rwa {
                        return Err(ModeError::BadRwa);
                    }
                    rwa = true;
                    mode.insert(Mode::READABLE);
                }
                b'w' => {
                    if rwa {
                        return Err(ModeError::BadRwa);
                    }
                    rwa = true;
                    mode.insert(Mode::WRITABLE);
                    flags |= libc::O_CREAT | libc::O_TRUNC;
                }
                b'a' => {
                    if rwa {
                        return Err(ModeError::BadRwa);
                    }
                    rwa = true;
                    mode.insert(Mode::WRITABLE | Mode::APPENDING);
                    flags |= libc::O_APPEND | libc::O_CREAT;
                }
                b'+' => {
                    if plus {
                        return Err(ModeError::BadRwa);
                    }
                    plus = true;
                    mode.insert(Mode::READABLE | Mode::WRITABLE);
                }
                b'b' => {}
                _ => return Err(ModeError::Invalid),
            }
        }

        if !rwa {
            return Err(ModeError::BadRwa);
        }

        if mode.contains(Mode::READABLE | Mode::WRITABLE) {
            flags |= libc::O_RDWR
        } else if mode.contains(Mode::READABLE) {
            flags |= libc::O_RDONLY
        } else {
            flags |= libc::O_WRONLY
        }

        #[cfg(windows)]
        {
            flags |= libc::O_BINARY | libc::O_NOINHERIT;
        }
        #[cfg(unix)]
        {
            flags |= libc::O_CLOEXEC
        }

        Ok((mode, flags as _))
    }

    #[pyattr]
    #[pyclass(module = "io", name, base = "_RawIOBase")]
    #[derive(Debug, PyPayload)]
    pub(super) struct FileIO {
        fd: AtomicCell<i32>,
        closefd: AtomicCell<bool>,
        mode: AtomicCell<Mode>,
        seekable: AtomicCell<Option<bool>>,
    }

    #[derive(FromArgs)]
    pub struct FileIOArgs {
        #[pyarg(positional)]
        name: PyObjectRef,
        #[pyarg(any, default)]
        mode: Option<PyUtf8StrRef>,
        #[pyarg(any, default = true)]
        closefd: bool,
        #[pyarg(any, default)]
        opener: Option<PyObjectRef>,
    }

    impl Default for FileIO {
        fn default() -> Self {
            Self {
                fd: AtomicCell::new(-1),
                closefd: AtomicCell::new(true),
                mode: AtomicCell::new(Mode::empty()),
                seekable: AtomicCell::new(None),
            }
        }
    }

    impl DefaultConstructor for FileIO {}

    impl Initializer for FileIO {
        type Args = FileIOArgs;

        fn init(zelf: PyRef<Self>, args: Self::Args, vm: &VirtualMachine) -> PyResult<()> {
            // TODO: let atomic_flag_works
            let name = args.name;
            let arg_fd = if let Some(i) = name.downcast_ref::<crate::builtins::PyInt>() {
                let fd = i.try_to_primitive(vm)?;
                if fd < 0 {
                    return Err(vm.new_value_error("negative file descriptor"));
                }
                Some(fd)
            } else {
                None
            };

            let mode_obj = args
                .mode
                .unwrap_or_else(|| PyUtf8Str::from("rb").into_ref(&vm.ctx));
            let mode_str = mode_obj.as_str();
            let (mode, flags) =
                compute_mode(mode_str).map_err(|e| vm.new_value_error(e.error_msg(mode_str)))?;
            zelf.mode.store(mode);

            let (fd, filename) = if let Some(fd) = arg_fd {
                zelf.closefd.store(args.closefd);
                (fd, OsPathOrFd::Fd(fd))
            } else {
                zelf.closefd.store(true);
                if !args.closefd {
                    return Err(vm.new_value_error("Cannot use closefd=False with file name"));
                }

                if let Some(opener) = args.opener {
                    let fd = opener.call((name.clone(), flags), vm)?;
                    if !fd.fast_isinstance(vm.ctx.types.int_type) {
                        return Err(vm.new_type_error("expected integer from opener"));
                    }
                    let fd = i32::try_from_object(vm, fd)?;
                    if fd < 0 {
                        return Err(vm.new_value_error(format!("opener returned {fd}")));
                    }
                    (
                        fd,
                        OsPathOrFd::try_from_object(vm, name.clone()).unwrap_or(OsPathOrFd::Fd(fd)),
                    )
                } else {
                    let path = OsPath::try_from_object(vm, name.clone())?;
                    #[cfg(any(unix, target_os = "wasi"))]
                    let fd = Fd::open(&path.clone().into_cstring(vm)?, flags, 0o666);
                    #[cfg(windows)]
                    let fd = Fd::wopen(&path.to_wide_cstring(vm)?, flags, 0o666);
                    let filename = OsPathOrFd::Path(path);
                    match fd {
                        Ok(fd) => (fd.0, filename),
                        Err(e) => return Err(IOErrorBuilder::with_filename(&e, filename, vm)),
                    }
                }
            };
            zelf.fd.store(fd);

            // TODO: _Py_set_inheritable

            let fd_fstat = crate::common::fileutils::fstat(fd);

            #[cfg(windows)]
            {
                if let Err(err) = fd_fstat {
                    return Err(IOErrorBuilder::with_filename(&err, filename, vm));
                }
            }
            #[cfg(any(unix, target_os = "wasi"))]
            {
                match fd_fstat {
                    Ok(status) => {
                        if (status.st_mode & libc::S_IFMT) == libc::S_IFDIR {
                            let err = std::io::Error::from_raw_os_error(libc::EISDIR);
                            return Err(IOErrorBuilder::with_filename(&err, filename, vm));
                        }
                    }
                    Err(err) => {
                        if err.raw_os_error() == Some(libc::EBADF) {
                            return Err(IOErrorBuilder::with_filename(&err, filename, vm));
                        }
                    }
                }
            }

            #[cfg(windows)]
            crate::stdlib::msvcrt::setmode_binary(fd);
            zelf.as_object().set_attr("name", name, vm)?;

            if mode.contains(Mode::APPENDING) {
                let _ = os::lseek(fd as _, 0, libc::SEEK_END, vm);
            }

            Ok(())
        }
    }

    impl Representable for FileIO {
        #[inline]
        fn repr_str(zelf: &Py<Self>, vm: &VirtualMachine) -> PyResult<String> {
            let fd = zelf.fd.load();
            if fd < 0 {
                return Ok("<_io.FileIO [closed]>".to_owned());
            }
            let name_repr = repr_file_obj_name(zelf.as_object(), vm)?;
            let mode = zelf.mode();
            let closefd = if zelf.closefd.load() { "True" } else { "False" };
            let repr = if let Some(name_repr) = name_repr {
                format!("<_io.FileIO name={name_repr} mode='{mode}' closefd={closefd}>")
            } else {
                format!("<_io.FileIO fd={fd} mode='{mode}' closefd={closefd}>")
            };
            Ok(repr)
        }
    }

    #[pyclass(
        with(Constructor, Initializer, Representable),
        flags(BASETYPE, HAS_DICT)
    )]
    impl FileIO {
        fn io_error(
            zelf: &Py<Self>,
            error: std::io::Error,
            vm: &VirtualMachine,
        ) -> PyBaseExceptionRef {
            let exc = error.to_pyexception(vm);
            if let Ok(name) = zelf.as_object().get_attr("name", vm) {
                exc.as_object()
                    .set_attr("filename", name, vm)
                    .expect("OSError.filename set must success");
            }
            exc
        }

        #[pygetset]
        fn closed(&self) -> bool {
            self.fd.load() < 0
        }

        #[pygetset]
        fn closefd(&self) -> bool {
            self.closefd.load()
        }

        #[pymethod]
        fn fileno(&self, vm: &VirtualMachine) -> PyResult<i32> {
            let fd = self.fd.load();
            if fd >= 0 {
                Ok(fd)
            } else {
                Err(io_closed_error(vm))
            }
        }

        fn get_fd(&self, vm: &VirtualMachine) -> PyResult<Fd> {
            self.fileno(vm).map(Fd)
        }

        #[pymethod]
        fn readable(&self) -> bool {
            self.mode.load().contains(Mode::READABLE)
        }

        #[pymethod]
        fn writable(&self) -> bool {
            self.mode.load().contains(Mode::WRITABLE)
        }

        #[pygetset]
        fn mode(&self) -> &'static str {
            let mode = self.mode.load();
            if mode.contains(Mode::CREATED) {
                if mode.contains(Mode::READABLE) {
                    "xb+"
                } else {
                    "xb"
                }
            } else if mode.contains(Mode::APPENDING) {
                if mode.contains(Mode::READABLE) {
                    "ab+"
                } else {
                    "ab"
                }
            } else if mode.contains(Mode::READABLE) {
                if mode.contains(Mode::WRITABLE) {
                    "rb+"
                } else {
                    "rb"
                }
            } else {
                "wb"
            }
        }

        #[pymethod]
        fn read(
            zelf: &Py<Self>,
            read_byte: OptionalSize,
            vm: &VirtualMachine,
        ) -> PyResult<Vec<u8>> {
            if !zelf.mode.load().contains(Mode::READABLE) {
                return Err(new_unsupported_operation(
                    vm,
                    "File or stream is not readable".to_owned(),
                ));
            }
            let mut handle = zelf.get_fd(vm)?;
            let bytes = if let Some(read_byte) = read_byte.to_usize() {
                let mut bytes = vec![0; read_byte];
                let n = handle
                    .read(&mut bytes)
                    .map_err(|err| Self::io_error(zelf, err, vm))?;
                bytes.truncate(n);
                bytes
            } else {
                let mut bytes = vec![];
                handle
                    .read_to_end(&mut bytes)
                    .map_err(|err| Self::io_error(zelf, err, vm))?;
                bytes
            };

            Ok(bytes)
        }

        #[pymethod]
        fn readinto(zelf: &Py<Self>, obj: ArgMemoryBuffer, vm: &VirtualMachine) -> PyResult<usize> {
            if !zelf.mode.load().contains(Mode::READABLE) {
                return Err(new_unsupported_operation(
                    vm,
                    "File or stream is not readable".to_owned(),
                ));
            }

            let handle = zelf.get_fd(vm)?;

            let mut buf = obj.borrow_buf_mut();
            let mut f = handle.take(buf.len() as _);
            let ret = f
                .read(&mut buf)
                .map_err(|err| Self::io_error(zelf, err, vm))?;

            Ok(ret)
        }

        #[pymethod]
        fn write(zelf: &Py<Self>, obj: ArgBytesLike, vm: &VirtualMachine) -> PyResult<usize> {
            if !zelf.mode.load().contains(Mode::WRITABLE) {
                return Err(new_unsupported_operation(
                    vm,
                    "File or stream is not writable".to_owned(),
                ));
            }

            let mut handle = zelf.get_fd(vm)?;

            let len = obj
                .with_ref(|b| handle.write(b))
                .map_err(|err| Self::io_error(zelf, err, vm))?;

            //return number of bytes written
            Ok(len)
        }

        #[pymethod]
        fn close(zelf: &Py<Self>, vm: &VirtualMachine) -> PyResult<()> {
            let res = iobase_close(zelf.as_object(), vm);
            if !zelf.closefd.load() {
                zelf.fd.store(-1);
                return res;
            }
            let fd = zelf.fd.swap(-1);
            if fd >= 0 {
                Fd(fd)
                    .close()
                    .map_err(|err| Self::io_error(zelf, err, vm))?;
            }
            res
        }

        #[pymethod]
        fn seekable(&self, vm: &VirtualMachine) -> PyResult<bool> {
            let fd = self.fileno(vm)?;
            Ok(self.seekable.load().unwrap_or_else(|| {
                let seekable = os::lseek(fd, 0, libc::SEEK_CUR, vm).is_ok();
                self.seekable.store(Some(seekable));
                seekable
            }))
        }

        #[pymethod]
        fn seek(
            &self,
            offset: PyObjectRef,
            how: OptionalArg<i32>,
            vm: &VirtualMachine,
        ) -> PyResult<Offset> {
            let how = how.unwrap_or(0);
            let fd = self.fileno(vm)?;
            let offset = get_offset(offset, vm)?;

            os::lseek(fd, offset, how, vm)
        }

        #[pymethod]
        fn tell(&self, vm: &VirtualMachine) -> PyResult<Offset> {
            let fd = self.fileno(vm)?;
            os::lseek(fd, 0, libc::SEEK_CUR, vm)
        }

        #[pymethod]
        fn truncate(&self, len: OptionalOption, vm: &VirtualMachine) -> PyResult<Offset> {
            let fd = self.fileno(vm)?;
            let len = match len.flatten() {
                Some(l) => get_offset(l, vm)?,
                None => os::lseek(fd, 0, libc::SEEK_CUR, vm)?,
            };
            os::ftruncate(fd, len, vm)?;
            Ok(len)
        }

        #[pymethod]
        fn isatty(&self, vm: &VirtualMachine) -> PyResult<bool> {
            let fd = self.fileno(vm)?;
            Ok(os::isatty(fd))
        }

        #[pymethod]
        fn __reduce__(zelf: PyObjectRef, vm: &VirtualMachine) -> PyResult {
            Err(vm.new_type_error(format!("cannot pickle '{}' object", zelf.class().name())))
        }
    }
}
