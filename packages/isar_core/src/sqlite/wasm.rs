use libsqlite3_sys::{SQLITE_IOERR, sqlite3_file, sqlite3_vfs, sqlite3_vfs_register};
use std::os::raw::{c_char, c_int, c_void};
use std::panic;
use std::ptr::null_mut;

unsafe extern "C" {
    pub fn js_error(ptr: *const u8);
}

#[inline]
pub(crate) fn set_wasm_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(hook_impl));
    });
}

fn hook_impl(info: &panic::PanicInfo) {
    let mut msg = info.to_string();
    unsafe {
        js_error(msg.as_ptr() as *const u8);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqlite3_os_init() -> c_int {
    let vfs = sqlite3_vfs {
        iVersion: 1,
        szOsFile: 0,
        mxPathname: 1024,
        pNext: null_mut(),
        zName: "libsqlite3-sys\0".as_ptr() as *const c_char,
        pAppData: null_mut(),
        xOpen: Some(wasm_vfs_open),
        xDelete: Some(wasm_vfs_delete),
        xAccess: Some(wasm_vfs_access),
        xFullPathname: Some(wasm_vfs_fullpathname),
        xDlOpen: Some(wasm_vfs_dlopen),
        xDlError: Some(wasm_vfs_dlerror),
        xDlSym: Some(wasm_vfs_dlsym),
        xDlClose: Some(wasm_vfs_dlclose),
        xRandomness: Some(wasm_vfs_randomness),
        xSleep: Some(wasm_vfs_sleep),
        xCurrentTime: Some(wasm_vfs_currenttime),
        xGetLastError: None,
        xCurrentTimeInt64: None,
        xSetSystemCall: None,
        xGetSystemCall: None,
        xNextSystemCall: None,
    };

    sqlite3_vfs_register(Box::leak(Box::new(vfs)), 1)
}

const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

const ALIGN: usize = max(
    8, // wasm32 max_align_t
    max(std::mem::size_of::<usize>(), std::mem::align_of::<usize>()),
);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    let layout = match std::alloc::Layout::from_size_align(size + ALIGN, ALIGN) {
        Ok(layout) => layout,
        Err(_) => return null_mut(),
    };

    let ptr = std::alloc::alloc(layout);
    if ptr.is_null() {
        return null_mut();
    }

    *(ptr as *mut usize) = size;
    ptr.offset(ALIGN as isize)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut u8) {
    let ptr = ptr.offset(-(ALIGN as isize));
    let size = *(ptr as *mut usize);
    let layout = std::alloc::Layout::from_size_align_unchecked(size + ALIGN, ALIGN);

    std::alloc::dealloc(ptr, layout);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut u8, new_size: usize) -> *mut u8 {
    let ptr = ptr.offset(-(ALIGN as isize));
    let size = *(ptr as *mut usize);
    let layout = std::alloc::Layout::from_size_align_unchecked(size + ALIGN, ALIGN);

    let ptr = std::alloc::realloc(ptr, layout, new_size + ALIGN);
    if ptr.is_null() {
        return null_mut();
    }

    *(ptr as *mut usize) = new_size;
    ptr.offset(ALIGN as isize)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wasm_vfs_open(
    _arg1: *mut sqlite3_vfs,
    _zName: *const c_char,
    _arg2: *mut sqlite3_file,
    _flags: c_int,
    _pOutFlags: *mut c_int,
) -> c_int {
    SQLITE_IOERR
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wasm_vfs_delete(
    _arg1: *mut sqlite3_vfs,
    _zName: *const c_char,
    _syncDir: c_int,
) -> c_int {
    SQLITE_IOERR
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wasm_vfs_access(
    _arg1: *mut sqlite3_vfs,
    _zName: *const c_char,
    _flags: c_int,
    _pResOut: *mut c_int,
) -> c_int {
    SQLITE_IOERR
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wasm_vfs_fullpathname(
    _arg1: *mut sqlite3_vfs,
    _zName: *const c_char,
    _nOut: c_int,
    _zOut: *mut c_char,
) -> c_int {
    SQLITE_IOERR
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wasm_vfs_dlopen(
    _arg1: *mut sqlite3_vfs,
    _zFilename: *const c_char,
) -> *mut c_void {
    null_mut()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wasm_vfs_dlerror(
    _arg1: *mut sqlite3_vfs,
    _nByte: c_int,
    _zErrMsg: *mut c_char,
) {
    // no-op
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wasm_vfs_dlsym(
    _arg1: *mut sqlite3_vfs,
    _arg2: *mut c_void,
    _zSymbol: *const c_char,
) -> ::std::option::Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut c_void, *const i8)> {
    None
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wasm_vfs_dlclose(_arg1: *mut sqlite3_vfs, _arg2: *mut c_void) {
    // no-op
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wasm_vfs_sleep(_arg1: *mut sqlite3_vfs, microseconds: c_int) -> c_int {
    0
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wasm_vfs_randomness(
    _arg1: *mut sqlite3_vfs,
    nByte: c_int,
    zByte: *mut c_char,
) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wasm_vfs_currenttime(_arg1: *mut sqlite3_vfs, pTime: *mut f64) -> c_int {
    0
}
