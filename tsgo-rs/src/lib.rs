use std::{
    borrow::Cow,
    ffi::{CStr, c_char, c_void},
    mem::ManuallyDrop,
    path::{Path, PathBuf},
};

use bindings::{compiler_host, ffi_string, ffi_string_array, program_options};

pub mod bindings {
    #![allow(non_snake_case, unsafe_op_in_unsafe_fn, nonstandard_style)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

trait StringExt: Sized {
    fn to_ffi(self) -> ffi_string;
}

impl StringExt for String {
    fn to_ffi(self) -> ffi_string {
        let mut bytes = ManuallyDrop::new(self.into_bytes());
        let buf = bytes.as_mut_ptr();
        let length = bytes.len();
        let capacity = bytes.capacity();

        ffi_string {
            buf: buf.cast(),
            length: length.try_into().unwrap(),
            capacity: capacity.try_into().unwrap(),
        }
    }
}

trait VecStringExt: Sized {
    fn to_ffi(self) -> ffi_string_array;
}

impl VecStringExt for Vec<String> {
    fn to_ffi(self) -> ffi_string_array {
        let mut new = Vec::with_capacity(self.len());
        for value in self {
            new.push(value.to_ffi());
        }
        let mut slice = ManuallyDrop::new(new.into_boxed_slice());
        ffi_string_array {
            arr: slice.as_mut_ptr(),
            length: slice.len() as i32,
        }
    }
}

struct GoArray<T> {
    pub len: usize,
    pub data: *mut T,
}

impl<T> From<bindings::go_array> for GoArray<T> {
    fn from(value: bindings::go_array) -> Self {
        Self {
            len: value.len,
            data: value.data.cast::<T>(),
        }
    }
}

trait FromGo<T> {
    fn from_go(go: T) -> Self;
}

impl FromGo<*mut c_char> for String {
    fn from_go(go: *mut c_char) -> Self {
        unsafe { CStr::from_ptr(go) }.to_string_lossy().into_owned()
    }
}

impl<T, U> FromGo<GoArray<U>> for Vec<T>
where
    T: FromGo<U>,
{
    fn from_go(go: GoArray<U>) -> Self {
        let mut v = Vec::with_capacity(go.len);
        for i in 0..go.len {
            v.push(T::from_go(unsafe { std::ptr::read(go.data.add(i)) }));
        }
        v
    }
}

#[derive(Debug)]
pub struct Diagnostic {
    pub message: String,
    pub file_name: String,
}

impl FromGo<bindings::diagnostic> for Diagnostic {
    fn from_go(d: bindings::diagnostic) -> Self {
        let ret = Self {
            message: String::from_go(d.message),
            file_name: String::from_go(d.file_name),
        };
        unsafe {
            bindings::free(d.message.cast());
            bindings::free(d.file_name.cast());
        }
        ret
    }
}

extern "C" fn free_ffi_string(s: ffi_string) {
    unsafe {
        let _ = Vec::<u8>::from_raw_parts(s.buf.cast(), s.length as usize, s.capacity as usize);
    }
}

extern "C" fn free_ffi_string_array(a: ffi_string_array) {
    for i in 0..a.length {
        let s = unsafe { *a.arr.add(i as usize) };
        free_ffi_string(s);
    }

    let _ = unsafe { Vec::from_raw_parts(a.arr, a.length as usize, a.length as usize) };
}

// compiler host methods
extern "C" fn get_source_file_text(
    data: *mut c_void,
    c_file_name: *mut c_char,
    c_path: *mut c_char,
    language_version: i32,
) -> ffi_string {
    let file_name = unsafe { CStr::from_ptr(c_file_name) }.to_string_lossy();
    let path = unsafe { CStr::from_ptr(c_path) }.to_string_lossy();
    unsafe { &*data.cast::<CompilerHostFacade>() }
        .inner
        .get_source_file_text(&file_name, &path, language_version)
        .to_ffi()
}

extern "C" fn default_library_path(data: *mut c_void) -> ffi_string {
    unsafe { &*data.cast::<CompilerHostFacade>() }
        .inner
        .default_library_path()
        .to_ffi()
}

extern "C" fn get_current_directory(data: *mut c_void) -> ffi_string {
    unsafe { &*data.cast::<CompilerHostFacade>() }
        .inner
        .get_current_directory()
        .to_ffi()
}

extern "C" fn new_line(data: *mut c_void) -> ffi_string {
    unsafe { &*data.cast::<CompilerHostFacade>() }
        .inner
        .new_line()
        .into_owned()
        .to_ffi()
}

struct CompilerHostFacade {
    inner: Box<dyn CompilerHost>,
}

pub trait CompilerHost {
    fn default_library_path(&self) -> String;
    fn get_current_directory(&self) -> String;
    fn new_line(&self) -> Cow<'static, str>;
    fn get_source_file_text(&self, file_name: &str, path: &str, language_version: i32) -> String;
}

pub fn get_diagnostics<T: CompilerHost + 'static>(
    host: T,
    config_path: &Path,
    roots: Vec<PathBuf>,
) {
    let program = unsafe {
        bindings::NewProgram(program_options {
            host: compiler_host {
                get_source_file_text: Some(get_source_file_text),
                free_ffi_string: Some(free_ffi_string),
                new_line: Some(new_line),
                get_current_directory: Some(get_current_directory),
                default_library_path: Some(default_library_path),
                free_ffi_string_array: Some(free_ffi_string_array),
                data: Box::into_raw(Box::new(CompilerHostFacade {
                    inner: Box::new(host),
                }))
                .cast(),
            },
            root_files: roots
                .into_iter()
                .map(|p| p.canonicalize().unwrap().to_string_lossy().into_owned())
                .collect::<Vec<String>>()
                .to_ffi(),
            config_file_name: config_path
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .into_owned()
                .to_ffi(),
        })
    };

    let diagnostics = unsafe { bindings::GetSyntacticDiagnostics(program, 0) };
    let diagnostics: GoArray<bindings::diagnostic> = diagnostics.into();

    let diagnostics = Vec::<Diagnostic>::from_go(diagnostics);
    eprintln!("diagnostics: {diagnostics:#?}");
}
