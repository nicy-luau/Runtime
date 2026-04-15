/*
Copyright (C) 2026 Yanlvl99 | Nicy Luau Runtime Development

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use libloading::{Library, Symbol};
use mlua_sys::luau::{compat, lauxlib, lua, lualib};
use std::ffi::CStr;
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

mod error;
mod ffi_exports;
mod require_resolver;
mod task_scheduler;

pub use error::{ErrorReporter, NicyError, RequireChain};

const RUNTIME_VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
const RUNTIME_VERSION_LABEL: &[u8] =
    concat!("Nicy Runtime ", env!("CARGO_PKG_VERSION"), "\0").as_bytes();

/// Registry reference to the error handler function.
/// Stored to allow proper cleanup via luaL_unref during shutdown.
static ERR_HANDLER_REF: AtomicI32 = AtomicI32::new(0);

/// Monotonic counter for generating unique temp file names.
/// FIX (IMP-4): Prevents race conditions when multiple threads/processes
/// call os_tmpname simultaneously.
static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Thin wrappers around Luau C API functions used throughout the runtime.
/// Provides a consistent calling convention for lib.rs internal usage.
#[allow(non_snake_case)]
mod api {
    use super::{compat, lauxlib, lua, lualib};
    use std::os::raw::{c_char, c_int};

    pub type LuauState = lua::lua_State;
    pub const LUA_REGISTRYINDEX: c_int = lua::LUA_REGISTRYINDEX;
    pub const LUA_TNIL: c_int = lua::LUA_TNIL;
    pub const LUA_TTABLE: c_int = lua::LUA_TTABLE;
    pub const LUA_TFUNCTION: c_int = lua::LUA_TFUNCTION;

    pub unsafe fn lua_pushnil(l: *mut LuauState) {
        unsafe { lua::lua_pushnil(l) };
    }

    pub unsafe fn lua_pop(l: *mut LuauState, n: c_int) {
        unsafe { lua::lua_pop(l, n) };
    }

    pub unsafe fn lua_pushlstring(l: *mut LuauState, s: *const c_char, len: usize) {
        unsafe { compat::lua_pushlstring(l, s, len) };
    }

    pub unsafe fn lua_tostring(l: *mut LuauState, idx: c_int) -> *const c_char {
        unsafe { lua::lua_tostring(l, idx) }
    }

    pub unsafe fn lua_gettop(l: *mut LuauState) -> c_int {
        unsafe { lua::lua_gettop(l) }
    }

    pub unsafe fn lua_getfield(l: *mut LuauState, idx: c_int, k: *const c_char) {
        unsafe { lua::lua_getfield(l, idx, k) };
    }

    pub unsafe fn lua_type(l: *mut LuauState, idx: c_int) -> c_int {
        unsafe { lua::lua_type(l, idx) }
    }

    pub unsafe fn lua_settop(l: *mut LuauState, idx: c_int) {
        unsafe { lua::lua_settop(l, idx) };
    }

    pub unsafe fn lua_createtable(l: *mut LuauState, narr: c_int, nrec: c_int) {
        unsafe { lua::lua_createtable(l, narr, nrec) };
    }

    pub unsafe fn lua_pushstring(l: *mut LuauState, s: *const c_char) {
        unsafe { compat::lua_pushstring(l, s) };
    }

    pub unsafe fn lua_setfield(l: *mut LuauState, idx: c_int, k: *const c_char) {
        unsafe { lua::lua_setfield(l, idx, k) };
    }

    pub unsafe fn lua_setmetatable(l: *mut LuauState, idx: c_int) -> c_int {
        unsafe { lua::lua_setmetatable(l, idx) }
    }

    pub unsafe fn lua_pushvalue(l: *mut LuauState, idx: c_int) {
        unsafe { lua::lua_pushvalue(l, idx) };
    }

    pub unsafe fn lua_absindex(l: *mut LuauState, idx: c_int) -> c_int {
        unsafe { lua::lua_absindex(l, idx) }
    }

    pub unsafe fn lua_gettable(l: *mut LuauState, idx: c_int) {
        unsafe { lua::lua_gettable(l, idx) };
    }

    pub unsafe fn lua_remove(l: *mut LuauState, idx: c_int) {
        unsafe { lua::lua_remove(l, idx) };
    }

    pub unsafe fn luaL_checkstring(l: *mut LuauState, narg: c_int) -> *const c_char {
        unsafe { lauxlib::luaL_checkstring(l, narg) }
    }

    pub unsafe fn lua_pushcfunction(l: *mut LuauState, f: lua::lua_CFunction) {
        unsafe { lua::lua_pushcfunction(l, f) };
    }

    pub unsafe fn lua_settable(l: *mut LuauState, idx: c_int) {
        unsafe { lua::lua_settable(l, idx) };
    }

    pub unsafe fn lua_pushboolean(l: *mut LuauState, b: c_int) {
        unsafe { lua::lua_pushboolean(l, b) };
    }

    pub unsafe fn lua_setglobal(l: *mut LuauState, k: *const c_char) {
        unsafe { lua::lua_setglobal(l, k) };
    }

    pub unsafe fn lua_getglobal(l: *mut LuauState, k: *const c_char) {
        unsafe { lua::lua_getglobal(l, k) };
    }

    pub unsafe fn luaL_newstate() -> *mut LuauState {
        unsafe { lauxlib::luaL_newstate() }
    }

    pub unsafe fn luaL_openlibs(l: *mut LuauState) {
        unsafe { lualib::luaL_openlibs(l) };
    }

    pub unsafe fn lua_close(l: *mut LuauState) {
        unsafe { lua::lua_close(l) };
    }

    pub unsafe fn luaL_loadbuffer(
        l: *mut LuauState,
        buff: *const c_char,
        sz: usize,
        name: *const c_char,
    ) -> c_int {
        unsafe { compat::luaL_loadbuffer(l, buff, sz, name) }
    }

    pub unsafe fn lua_pcall(
        l: *mut LuauState,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
    ) -> c_int {
        unsafe { lua::lua_pcall(l, nargs, nresults, errfunc) }
    }
}

#[cfg(windows)]
mod hires_timer {
    use std::env;
    use std::os::raw::c_uint;

    type MmResult = u32;

    #[link(name = "winmm")]
    unsafe extern "system" {
        fn timeBeginPeriod(uPeriod: c_uint) -> MmResult;
        fn timeEndPeriod(uPeriod: c_uint) -> MmResult;
    }

    pub struct Guard {
        enabled: bool,
    }

    impl Guard {
        pub fn maybe_enable() -> Self {
            let enabled = match env::var_os("NICY_HIRES_TIMER") {
                Some(v) => v != "0",
                None => false,
            };
            if enabled {
                unsafe {
                    timeBeginPeriod(1);
                }
            }

            Self { enabled }
        }
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            if self.enabled {
                unsafe {
                    timeEndPeriod(1);
                }
            }
        }
    }
}

type LuauState = api::LuauState;

unsafe fn push_loadlib_error(l: *mut LuauState, msg: &str) -> c_int {
    let filtered = msg.replace('\0', "?");

    // Also emit a warn so the error is visible even if the script doesn't check the return value
    ErrorReporter::warn(&format!("runtime.loadlib failed: {}", filtered));

    unsafe { api::lua_pushnil(l) };
    unsafe {
        api::lua_pushlstring(
            l,
            filtered.as_ptr() as *const c_char,
            filtered.as_bytes().len(),
        )
    };
    2
}

static LOADED_LIBS: OnceLock<Mutex<Vec<Library>>> = OnceLock::new();

fn loaded_libs() -> &'static Mutex<Vec<Library>> {
    LOADED_LIBS.get_or_init(|| Mutex::new(Vec::new()))
}

/// CRITICAL FIX (C-1, C-5): Clear all loaded libraries.
/// This unloads native DLLs and prevents memory leaks between nicy_start calls.
pub fn shutdown_loaded_libs() {
    if let Some(libs) = LOADED_LIBS.get() {
        let mut guard = libs.lock().unwrap_or_else(|e| e.into_inner());
        // Drop all libraries - this calls dlclose/FreeLibrary on each
        guard.clear();
    }
}

fn panic_payload_to_string(p: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = p.downcast_ref::<&str>() {
        return (*s).to_string();
    }
    if let Some(s) = p.downcast_ref::<String>() {
        return s.clone();
    }
    "non-string panic payload".to_string()
}

/// Log a panic error and terminate. Used in catch_unwind error handlers
/// for FFI entry points (nicy_start, nicy_eval, nicy_compile).
fn log_panic(context: &'static str, p: Box<dyn std::any::Any + Send>) {
    let err = NicyError::panic_error(context, panic_payload_to_string(p));
    ErrorReporter::fatal(&err);
}

/// Strip the `--!native` directive from the first line of Luau source code.
/// Returns whether native compilation was requested and the cleaned source.
pub(crate) fn strip_native_directive(source: &str) -> (bool, String) {
    let mut lines = source.lines();
    let first = lines.next().unwrap_or("");
    let enabled = first.trim().starts_with("--!native");
    if !enabled {
        return (false, source.to_string());
    }
    let rest = lines.collect::<Vec<_>>().join("\n");
    (true, rest)
}

/// Parsed compiler directives from Luau source code.
struct CompilerDirectives {
    native: bool,
    optimization_level: i32,
    coverage: bool,
    profile: bool,
    type_info_level: i32,
}

/// Parse all Luau compiler directives from source code.
/// Supports: --!native, --!optimize N, --!coverage, --!profile, --!typeinfo N
/// Also skips non-compiler directives like --!strict, --!nocheck, --!warning.
/// Directives can appear on multiple consecutive lines at the top of the file.
fn parse_compiler_directives(source: &str) -> (CompilerDirectives, String) {
    let mut directives = CompilerDirectives {
        native: false,
        optimization_level: 1, // default
        coverage: false,
        profile: false,
        type_info_level: 0, // default
    };
    let mut directive_end_line: usize = 0;

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "--!native" || trimmed.starts_with("--!native ") {
            directives.native = true;
            directive_end_line = i + 1;
        } else if let Some(rest) = trimmed.strip_prefix("--!optimize") {
            let rest = rest.trim();
            if let Some(num_str) = rest.strip_prefix(' ') {
                if let Ok(n) = num_str.trim().parse::<i32>() {
                    directives.optimization_level = n.clamp(0, 2);
                    directive_end_line = i + 1;
                }
            }
        } else if trimmed == "--!coverage" || trimmed.starts_with("--!coverage ") {
            directives.coverage = true;
            directive_end_line = i + 1;
        } else if trimmed == "--!profile" || trimmed.starts_with("--!profile ") {
            directives.profile = true;
            directive_end_line = i + 1;
        } else if let Some(rest) = trimmed.strip_prefix("--!typeinfo") {
            let rest = rest.trim();
            if let Some(num_str) = rest.strip_prefix(' ') {
                if let Ok(n) = num_str.trim().parse::<i32>() {
                    directives.type_info_level = n.clamp(0, 1);
                    directive_end_line = i + 1;
                }
            }
        } else if trimmed.starts_with("--!") {
            // Unknown or non-compiler directive (e.g. --!strict, --!nocheck)
            // Skip it but continue scanning for more directives
            directive_end_line = i + 1;
        } else {
            // First non-directive line
            break;
        }
    }

    let code = source
        .lines()
        .skip(directive_end_line)
        .collect::<Vec<_>>()
        .join("\n");

    (directives, code)
}

unsafe fn string_from_stack(l: *mut LuauState, idx: c_int) -> String {
    let p = unsafe { api::lua_tostring(l, idx) };
    if p.is_null() {
        return "nil".to_string();
    }
    unsafe { CStr::from_ptr(p) }.to_string_lossy().to_string()
}

unsafe extern "C-unwind" fn nicy_runtime_warn(l: *mut LuauState) -> c_int {
    let top = unsafe { api::lua_gettop(l) };
    if top <= 0 {
        ErrorReporter::warn("");
        return 0;
    }
    let mut parts = Vec::with_capacity(top as usize);
    for i in 1..=top {
        parts.push(unsafe { string_from_stack(l, i) });
    }
    ErrorReporter::warn(&parts.join(" "));
    0
}

unsafe fn get_or_create_extension_cache_table(l: *mut LuauState) {
    unsafe {
        api::lua_getfield(
            l,
            api::LUA_REGISTRYINDEX,
            b"nicy_ext_cache\0".as_ptr() as *const c_char,
        )
    };
    if unsafe { api::lua_type(l, -1) } != api::LUA_TTABLE {
        // Table doesn't exist: remove nil and create new
        unsafe { api::lua_settop(l, -2) };

        unsafe { api::lua_createtable(l, 0, 0) };
        unsafe { api::lua_createtable(l, 0, 1) };
        unsafe { api::lua_pushstring(l, b"v\0".as_ptr() as *const c_char) };
        unsafe { api::lua_setfield(l, -2, b"__mode\0".as_ptr() as *const c_char) };
        unsafe { api::lua_setmetatable(l, -2) };

        unsafe { api::lua_pushvalue(l, -1) };
        unsafe {
            api::lua_setfield(
                l,
                api::LUA_REGISTRYINDEX,
                b"nicy_ext_cache\0".as_ptr() as *const c_char,
            )
        };
    } else {
        // Table already exists: remove from stack to balance
        unsafe { api::lua_pop(l, 1) };
    }
}

unsafe extern "C-unwind" fn nicy_runtime_loadlib(l: *mut LuauState) -> c_int {
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        let path_ptr = api::luaL_checkstring(l, 1);
        if path_ptr.is_null() {
            return Err("invalid path".to_string());
        }

        let path_spec = CStr::from_ptr(path_ptr)
            .to_str()
            .map_err(|_| "invalid path encoding".to_string())?;

        let resolved_path = require_resolver::resolve_loadlib_path(l, path_spec)?;
        let resolved_key = resolved_path.to_string_lossy().to_string();

        // Check if file exists before attempting to load
        if !resolved_path.exists() {
            return Err(format!("file not found: '{}'", resolved_path.display()));
        }

        get_or_create_extension_cache_table(l);
        let cache_idx = api::lua_absindex(l, -1);
        api::lua_pushlstring(
            l,
            resolved_key.as_ptr() as *const c_char,
            resolved_key.as_bytes().len(),
        );
        api::lua_gettable(l, cache_idx);
        if api::lua_type(l, -1) != api::LUA_TNIL {
            api::lua_remove(l, cache_idx);
            return Ok(1);
        }
        api::lua_settop(l, -2);

        // FIX: Wrap Library::new in catch_unwind to prevent SEH crashes on Windows
        // Library::new can trigger Windows loader exceptions (STATUS_ENTRYPOINT_NOT_FOUND,
        // DLL_INIT_FAILED, etc.) which bypass Rust's panic system
        let lib_result = catch_unwind(AssertUnwindSafe(|| {
            Library::new(&resolved_path).map_err(|e| {
                format!(
                    "failed to load library '{}': {}",
                    resolved_path.display(),
                    e
                )
            })
        }));

        let lib = match lib_result {
            Ok(Ok(lib)) => lib,
            Ok(Err(msg)) => {
                return Err(msg);
            }
            Err(_) => {
                #[cfg(windows)]
                let detail = format!(
                    "failed to load library '{}': native DLL crashed during load (possible missing dependency or SEH exception)",
                    resolved_path.display()
                );
                #[cfg(not(windows))]
                let detail = format!(
                    "failed to load library '{}': native library crashed during load",
                    resolved_path.display()
                );
                ErrorReporter::report(&NicyError::PanicError {
                    context: "loadlib",
                    payload: detail.clone(),
                });
                return Err(detail);
            }
        };

        let init_fn: Symbol<unsafe extern "C-unwind" fn(*mut LuauState) -> c_int> = lib
            .get(b"nicydynamic_init")
            .or_else(|_| lib.get(b"nicydinamic_init"))
            .map_err(|e| format!("missing extension init symbol (nicydynamic_init): {}", e))?;

        let init_res = catch_unwind(AssertUnwindSafe(|| init_fn(l)));
        let res = match init_res {
            Ok(v) => v,
            Err(panic_err) => {
                let msg = panic_payload_to_string(panic_err);
                #[cfg(windows)]
                let detail = format!("{} - native DLL crashed (possible SEH exception)", msg);
                #[cfg(not(windows))]
                let detail = format!("{} - native DLL crashed", msg);
                ErrorReporter::report(&NicyError::PanicError {
                    context: "loadlib",
                    payload: detail.clone(),
                });
                return Err(format!("extension panic during init: {}", detail));
            }
        };

        if res != 1 {
            return Err("invalid extension return count (expected 1)".to_string());
        }

        get_or_create_extension_cache_table(l);
        let cache_idx = api::lua_absindex(l, -1);
        api::lua_pushlstring(
            l,
            resolved_key.as_ptr() as *const c_char,
            resolved_key.as_bytes().len(),
        );
        api::lua_pushvalue(l, -3);
        api::lua_settable(l, cache_idx);
        api::lua_remove(l, cache_idx);

        loaded_libs()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(lib);
        Ok(1)
    }));

    match result {
        Ok(Ok(n)) => n,
        Ok(Err(msg)) => unsafe { push_loadlib_error(l, &msg) },
        Err(p) => unsafe {
            push_loadlib_error(
                l,
                &format!(
                    "runtime panic in runtime.loadlib: {}",
                    panic_payload_to_string(p)
                ),
            )
        },
    }
}

unsafe extern "C-unwind" fn nicy_runtime_has_jit(l: *mut LuauState) -> c_int {
    let top = unsafe { api::lua_gettop(l) };
    let spec = if top >= 1 {
        let ptr = unsafe { api::luaL_checkstring(l, 1) };
        if ptr.is_null() {
            None
        } else {
            unsafe { CStr::from_ptr(ptr) }.to_str().ok()
        }
    } else {
        None
    };

    let enabled = require_resolver::has_jit(l, spec);
    unsafe { api::lua_pushboolean(l, enabled as c_int) };
    1
}

/// Safe pcall wrapper that tracks pcall state for error reporting
pub unsafe fn safe_pcall(
    l: *mut LuauState,
    nargs: c_int,
    nresults: c_int,
    errfunc: c_int,
) -> c_int {
    ErrorReporter::enter_pcall(l);
    let result = unsafe { api::lua_pcall(l, nargs, nresults, errfunc) };
    ErrorReporter::exit_pcall(l);
    result
}

/// Install a Lua error handler that formats errors nicely.
/// FIX (SEC-7, CODE-6): Uses luaL_ref to generate a unique registry index
/// instead of hardcoded magic number 1, which could collide with other
/// integer-keyed registry entries.
unsafe fn install_error_handler(l: *mut LuauState) -> c_int {
    use std::ffi::CString;

    // Create error handler function in Lua
    let handler_code = r#"
        return function(err)
            local traceback = debug.traceback(tostring(err), 2)
            return traceback
        end
    "#;

    let c_handler = CString::new(handler_code)
        .expect("error handler Lua code must not contain interior null bytes");
    let status = unsafe {
        compat::luaL_loadbuffer(
            l,
            c_handler.as_ptr(),
            c_handler.as_bytes().len(),
            b"error_handler\0".as_ptr() as *const _,
        )
    };

    if status != 0 {
        // Load failed: error message is on stack, pop it
        unsafe { lua::lua_pop(l, 1) };
        return 0;
    }

    // Load succeeded: function is on stack, call it
    let pcall_status = unsafe { api::lua_pcall(l, 0, 1, 0) };
    if pcall_status != 0 {
        // Pcall failed: error message is on stack, pop it
        unsafe { lua::lua_pop(l, 1) };
        return 0;
    }

    // FIX (CODE-6): Use luaL_ref to generate a unique registry index
    // instead of hardcoded magic number 1 (lua_rawseti at index 1)
    let ref_id = unsafe { lauxlib::luaL_ref(l, lua::LUA_REGISTRYINDEX) };

    // Store the registry reference for cleanup during shutdown
    ERR_HANDLER_REF.store(ref_id, Ordering::SeqCst);
    1
}

unsafe fn push_nicy_table(l: *mut LuauState, entry_path: &PathBuf) {
    let luau_ver_ptr = nicy_luau_version();

    unsafe {
        api::lua_getglobal(l, b"_G\0".as_ptr() as *const c_char);
        if api::lua_type(l, -1) == api::LUA_TTABLE {
            api::lua_pushstring(l, luau_ver_ptr);
            api::lua_setfield(l, -2, b"_VERSION\0".as_ptr() as *const c_char);
        }
        api::lua_pop(l, 1);
    }

    unsafe { api::lua_createtable(l, 0, 5) };

    unsafe { api::lua_pushstring(l, RUNTIME_VERSION.as_ptr() as *const c_char) };
    unsafe { api::lua_setfield(l, -2, b"version\0".as_ptr() as *const c_char) };

    unsafe { api::lua_pushcfunction(l, nicy_runtime_loadlib) };
    unsafe { api::lua_setfield(l, -2, b"loadlib\0".as_ptr() as *const c_char) };
    unsafe { api::lua_pushcfunction(l, nicy_runtime_has_jit) };
    unsafe { api::lua_setfield(l, -2, b"hasJIT\0".as_ptr() as *const c_char) };

    let entry_file = entry_path.to_string_lossy().to_string();
    unsafe {
        api::lua_pushlstring(
            l,
            entry_file.as_ptr() as *const c_char,
            entry_file.as_bytes().len(),
        )
    };
    unsafe { api::lua_setfield(l, -2, b"entry_file\0".as_ptr() as *const c_char) };

    let entry_dir = entry_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    unsafe {
        api::lua_pushlstring(
            l,
            entry_dir.as_ptr() as *const c_char,
            entry_dir.as_bytes().len(),
        )
    };
    unsafe { api::lua_setfield(l, -2, b"entry_dir\0".as_ptr() as *const c_char) };

    unsafe { api::lua_setglobal(l, b"runtime\0".as_ptr() as *const c_char) };

    unsafe { api::lua_getglobal(l, b"warn\0".as_ptr() as *const c_char) };
    if unsafe { api::lua_type(l, -1) } == api::LUA_TNIL {
        unsafe { api::lua_settop(l, -2) };
        unsafe { api::lua_pushcfunction(l, nicy_runtime_warn) };
        unsafe { api::lua_setglobal(l, b"warn\0".as_ptr() as *const c_char) };
    } else {
        unsafe { api::lua_settop(l, -2) };
    }
}

// ── OS library extensions ───────────────────────────────────────

unsafe extern "C-unwind" fn os_exit(l: *mut LuauState) -> c_int {
    let code = if unsafe { api::lua_gettop(l) } >= 1 {
        unsafe { lua::lua_tonumber(l, 1) as i32 }
    } else {
        0
    };
    std::process::exit(code);
}

unsafe extern "C-unwind" fn os_getenv(l: *mut LuauState) -> c_int {
    let name_ptr = unsafe { api::luaL_checkstring(l, 1) };
    if name_ptr.is_null() {
        unsafe { api::lua_pushnil(l) };
        return 1;
    }
    let name = unsafe { CStr::from_ptr(name_ptr) }.to_string_lossy();
    match std::env::var_os(&*name) {
        Some(val) => {
            let val_str = val.to_string_lossy();
            unsafe { api::lua_pushlstring(l, val_str.as_ptr() as *const c_char, val_str.len()) };
        }
        None => unsafe { api::lua_pushnil(l) },
    }
    1
}

unsafe extern "C-unwind" fn os_remove(l: *mut LuauState) -> c_int {
    let path_ptr = unsafe { api::luaL_checkstring(l, 1) };
    if path_ptr.is_null() {
        unsafe { api::lua_pushboolean(l, 0) };
        return 1;
    }
    let path = unsafe { CStr::from_ptr(path_ptr) }.to_string_lossy();
    let success = fs::remove_file(&*path).is_ok() || fs::remove_dir(&*path).is_ok();
    unsafe { api::lua_pushboolean(l, if success { 1 } else { 0 }) };
    1
}

unsafe extern "C-unwind" fn os_rename(l: *mut LuauState) -> c_int {
    let old_ptr = unsafe { api::luaL_checkstring(l, 1) };
    let new_ptr = unsafe { api::luaL_checkstring(l, 2) };
    if old_ptr.is_null() || new_ptr.is_null() {
        unsafe { api::lua_pushboolean(l, 0) };
        return 1;
    }
    let old_path = unsafe { CStr::from_ptr(old_ptr) }
        .to_string_lossy()
        .to_string();
    let new_path = unsafe { CStr::from_ptr(new_ptr) }
        .to_string_lossy()
        .to_string();
    let success = fs::rename(&old_path, &new_path).is_ok();
    unsafe { api::lua_pushboolean(l, if success { 1 } else { 0 }) };
    1
}

unsafe extern "C-unwind" fn os_sleep(l: *mut LuauState) -> c_int {
    let ms = if unsafe { api::lua_gettop(l) } >= 1 {
        unsafe { lauxlib::luaL_checknumber(l, 1) as u64 }
    } else {
        0
    };
    std::thread::sleep(std::time::Duration::from_millis(ms));
    0
}

unsafe extern "C-unwind" fn os_tmpname(l: *mut LuauState) -> c_int {
    let tmp = std::env::temp_dir();
    // FIX (IMP-4): Add monotonic counter to prevent race conditions.
    // PID + timestamp alone can collide when multiple threads call this
    // simultaneously within the same nanosecond.
    let counter = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = format!(
        "nicy_tmp_{}_{}_{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        counter
    );
    let full_path = tmp.join(&name);
    unsafe {
        api::lua_pushlstring(
            l,
            full_path.to_string_lossy().as_ptr() as *const c_char,
            full_path.to_string_lossy().len(),
        )
    };
    1
}

/// Extend the standard `os` library with additional functions.
/// Must be called AFTER luaL_openlibs so that the `os` table already exists.
unsafe fn extend_os_library(l: *mut LuauState) {
    unsafe { api::lua_getglobal(l, b"os\0".as_ptr() as *const c_char) };

    unsafe { api::lua_pushcfunction(l, os_exit) };
    unsafe { api::lua_setfield(l, -2, b"exit\0".as_ptr() as *const c_char) };

    unsafe { api::lua_pushcfunction(l, os_getenv) };
    unsafe { api::lua_setfield(l, -2, b"getenv\0".as_ptr() as *const c_char) };

    unsafe { api::lua_pushcfunction(l, os_remove) };
    unsafe { api::lua_setfield(l, -2, b"remove\0".as_ptr() as *const c_char) };

    unsafe { api::lua_pushcfunction(l, os_rename) };
    unsafe { api::lua_setfield(l, -2, b"rename\0".as_ptr() as *const c_char) };

    unsafe { api::lua_pushcfunction(l, os_sleep) };
    unsafe { api::lua_setfield(l, -2, b"sleep\0".as_ptr() as *const c_char) };

    unsafe { api::lua_pushcfunction(l, os_tmpname) };
    unsafe { api::lua_setfield(l, -2, b"tmpname\0".as_ptr() as *const c_char) };

    unsafe { api::lua_pop(l, 1) };
}

// ── collectgarbage implementation ──────────────────────────────────

// Luau GC constants (from lua.h)
const LUA_GCSTOP: c_int = 0;
const LUA_GCRESTART: c_int = 1;
const LUA_GCCOLLECT: c_int = 2;
const LUA_GCCOUNT: c_int = 3;
const LUA_GCISRUNNING: c_int = 5;
const LUA_GCSTEP: c_int = 6;

unsafe extern "C-unwind" fn nicy_collectgarbage(l: *mut LuauState) -> c_int {
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        let opt_ptr = api::luaL_checkstring(l, 1);
        if opt_ptr.is_null() {
            return Err("invalid option".to_string());
        }
        let opt = CStr::from_ptr(opt_ptr)
            .to_str()
            .map_err(|_| "invalid option encoding".to_string())?;

        match opt {
            "count" => {
                let raw_kb = lua::lua_gc(l, LUA_GCCOUNT, 0);
                lua::lua_pushnumber(l, raw_kb as f64);
                Ok(1)
            }
            "collect" => {
                lua::lua_gc(l, LUA_GCCOLLECT, 0);
                Ok(0)
            }
            "isrunning" => {
                let running = lua::lua_gc(l, LUA_GCISRUNNING, 0);
                api::lua_pushboolean(l, if running != 0 { 1 } else { 0 });
                Ok(1)
            }
            "stop" => {
                lua::lua_gc(l, LUA_GCSTOP, 0);
                Ok(0)
            }
            "restart" => {
                lua::lua_gc(l, LUA_GCRESTART, 0);
                Ok(0)
            }
            "step" => {
                let size = if api::lua_gettop(l) >= 2 {
                    (lauxlib::luaL_checkinteger_(l, 2)) as c_int
                } else {
                    100 // default step size
                };
                let result = lua::lua_gc(l, LUA_GCSTEP, size);
                api::lua_pushboolean(l, if result != 0 { 1 } else { 0 });
                Ok(1)
            }
            _ => Err(format!("invalid option '{}'", opt)),
        }
    }));

    match result {
        Ok(Ok(n)) => n,
        Ok(Err(msg)) => unsafe {
            ErrorReporter::warn(&format!("collectgarbage: {}", msg));
            api::lua_pushnil(l);
            api::lua_pushlstring(l, msg.as_ptr() as *const c_char, msg.as_bytes().len());
            2
        },
        Err(p) => unsafe {
            let msg = format!("panic in collectgarbage: {}", panic_payload_to_string(p));
            ErrorReporter::report(&NicyError::PanicError {
                context: "collectgarbage",
                payload: msg.clone(),
            });
            api::lua_pushnil(l);
            api::lua_pushlstring(l, msg.as_ptr() as *const c_char, msg.as_bytes().len());
            2
        },
    }
}

/// Inject `collectgarbage` as a global function.
/// Luau's luaL_openlibs does NOT include collectgarbage, so we add it manually.
unsafe fn extend_collectgarbage(l: *mut LuauState) {
    unsafe { api::lua_pushcfunction(l, nicy_collectgarbage) };
    unsafe { api::lua_setglobal(l, b"collectgarbage\0".as_ptr() as *const c_char) };
}

#[unsafe(no_mangle)]
pub extern "C" fn nicy_start(path_ptr: *const c_char) {
    error::auto_init_logging();
    if let Err(p) = catch_unwind(AssertUnwindSafe(|| {
        if path_ptr.is_null() {
            ErrorReporter::fatal(&NicyError::panic_error("nicy_start", "path_ptr is null"));
            return;
        }

        #[cfg(windows)]
        let _hires_timer = hires_timer::Guard::maybe_enable();

        let c_str = unsafe { CStr::from_ptr(path_ptr) };
        let path_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => {
                let path = unsafe { CStr::from_ptr(path_ptr) }
                    .to_string_lossy()
                    .to_string();
                ErrorReporter::report(&NicyError::file_error(
                    &path,
                    "read",
                    "invalid path encoding".to_string(),
                ));
                return;
            }
        };

        let entry_path = match fs::canonicalize(path_str) {
            Ok(p) => p,
            Err(_) => PathBuf::from(path_str),
        };

        let ext = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if ext != "luau" && ext != "lua" && ext != "luauc" {
            ErrorReporter::report(&NicyError::file_error(
                entry_path.to_string_lossy().to_string(),
                "open",
                format!(
                    "unsupported file extension '.{}', expected '.luau', '.lua', or '.luauc'",
                    ext
                ),
            ));
            return;
        }

        let is_bytecode = ext == "luauc";

        let (mut code_bytes, code_is_bytecode) = if is_bytecode {
            match fs::read(&entry_path) {
                Ok(c) => (c, true),
                Err(e) => {
                    ErrorReporter::report(&NicyError::file_error(
                        entry_path.to_string_lossy().to_string(),
                        "open",
                        format!("Error reading bytecode file: {}", e),
                    ));
                    return;
                }
            }
        } else {
            let code = match fs::read_to_string(&entry_path) {
                Ok(c) => c,
                Err(e) => {
                    ErrorReporter::report(&NicyError::file_error(
                        entry_path.to_string_lossy().to_string(),
                        "open",
                        format!("Error opening file: {}", e),
                    ));
                    return;
                }
            };
            (code.into_bytes(), false)
        };

        let entry_native_requested = if code_is_bytecode {
            false
        } else {
            let code_str = String::from_utf8_lossy(&code_bytes).to_string();
            let (native, effective) = strip_native_directive(&code_str);
            code_bytes = effective.into_bytes();
            native
        };

        unsafe {
            let l = api::luaL_newstate();
            if l.is_null() {
                ErrorReporter::fatal(&NicyError::runtime_error("Failed to create Luau state"));
                return;
            }

            api::luaL_openlibs(l);
            // Extend the standard os library with native functions
            extend_os_library(l);
            // Inject collectgarbage global function (not included by luaL_openlibs in Luau)
            extend_collectgarbage(l);
            let entry_jit_enabled = !code_is_bytecode
                && entry_native_requested
                && require_resolver::ensure_codegen_context(l);
            task_scheduler::init(l);
            push_nicy_table(l, &entry_path);
            require_resolver::install_require(l);
            if let Err(e) = require_resolver::init_runtime(l, &entry_path) {
                ErrorReporter::report(&NicyError::runtime_error(format!(
                    "Failed to init runtime: {}",
                    e
                )));
                api::lua_close(l);
                return;
            }
            if let Err(e) = require_resolver::set_entry_jit(l, entry_jit_enabled) {
                ErrorReporter::report(&NicyError::runtime_error(format!(
                    "Failed to set entry jit: {}",
                    e
                )));
                require_resolver::shutdown_runtime(l);
                api::lua_close(l);
                return;
            }

            let mut chunkname = entry_path.to_string_lossy().as_bytes().to_vec();
            for b in &mut chunkname {
                if *b == 0 {
                    *b = b'?';
                }
            }
            chunkname.push(0);

            let load_status = if code_is_bytecode {
                mlua_sys::luau::luau_load(
                    l,
                    chunkname.as_ptr() as *const c_char,
                    code_bytes.as_ptr() as *const c_char,
                    code_bytes.len(),
                    0,
                )
            } else {
                api::luaL_loadbuffer(
                    l,
                    code_bytes.as_ptr() as *const c_char,
                    code_bytes.len(),
                    chunkname.as_ptr() as *const c_char,
                )
            };
            if load_status != 0 {
                ErrorReporter::report_lua_error(l, "load");
                require_resolver::shutdown_runtime(l);
                api::lua_close(l);
                return;
            }

            if code_is_bytecode {
                // Bytecode is already compiled, just need to apply CodeGen for native execution
                require_resolver::ensure_codegen_context(l);
                require_resolver::compile_loaded_chunk(l);
            } else if entry_jit_enabled {
                require_resolver::compile_loaded_chunk(l);
            }

            if let Err(e) = require_resolver::push_entry_module(l) {
                ErrorReporter::report(&NicyError::runtime_error(format!(
                    "Failed to push entry module: {}",
                    e
                )));
                require_resolver::shutdown_runtime(l);
                api::lua_close(l);
                return;
            }

            // Install error handler for better error reporting
            let _err_handler_ref = install_error_handler(l);

            task_scheduler::schedule_main_thread(l);
            api::lua_settop(l, -2);

            task_scheduler::run_until_idle(l);

            // FIX (LEAK-1): Clean up error handler registry reference BEFORE other shutdown.
            // Must happen before shutdown_all_globals so the Lua state is still valid.
            let err_ref = ERR_HANDLER_REF.swap(0, Ordering::SeqCst);
            if err_ref != 0 {
                lauxlib::luaL_unref(l, lua::LUA_REGISTRYINDEX, err_ref);
            }

            // FIX (LEAK-2): Remove nicy_ext_cache table from registry.
            // This table is created by get_or_create_extension_cache_table and
            // persists across runs if not cleaned up.
            api::lua_pushnil(l);
            api::lua_setfield(
                l,
                api::LUA_REGISTRYINDEX,
                b"nicy_ext_cache\0".as_ptr() as *const c_char,
            );

            require_resolver::shutdown_all_globals(l);
            task_scheduler::shutdown_scheduler(l);
            shutdown_loaded_libs();
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            api::lua_close(l);
        }
    })) {
        log_panic("nicy_start", p);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nicy_eval(code_ptr: *const c_char) {
    if let Err(p) = catch_unwind(AssertUnwindSafe(|| {
        if code_ptr.is_null() {
            ErrorReporter::fatal(&NicyError::panic_error("nicy_eval", "code_ptr is null"));
            return;
        }

        let c_str = unsafe { CStr::from_ptr(code_ptr) };
        let code_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => {
                ErrorReporter::fatal(&NicyError::runtime_error("Invalid code encoding"));
                return;
            }
        };

        unsafe {
            let l = api::luaL_newstate();
            if l.is_null() {
                ErrorReporter::fatal(&NicyError::runtime_error(
                    "Failed to create Luau state for eval",
                ));
                return;
            }

            api::luaL_openlibs(l);
            // Extend the standard os library with native functions
            extend_os_library(l);
            // Inject collectgarbage global function
            extend_collectgarbage(l);

            task_scheduler::init(l);

            let eval_path = PathBuf::from("eval");

            require_resolver::install_require(l);
            let _ = require_resolver::init_runtime(l, &eval_path);

            push_nicy_table(l, &eval_path);

            let chunkname = b"eval\0";
            let load_status = api::luaL_loadbuffer(
                l,
                code_str.as_ptr() as *const c_char,
                code_str.as_bytes().len(),
                chunkname.as_ptr() as *const c_char,
            );

            if load_status != 0 {
                ErrorReporter::report_lua_error(l, "eval compile");
                require_resolver::shutdown_runtime(l);
                api::lua_close(l);
                return;
            }

            let call_status = api::lua_pcall(l, 0, 0, 0);
            if call_status != 0 {
                ErrorReporter::report_lua_error(l, "eval");
            }

            task_scheduler::run_until_idle(l);

            // FIX (LEAK-2): Remove nicy_ext_cache table from registry.
            api::lua_pushnil(l);
            api::lua_setfield(
                l,
                api::LUA_REGISTRYINDEX,
                b"nicy_ext_cache\0".as_ptr() as *const c_char,
            );

            require_resolver::shutdown_all_globals(l);
            task_scheduler::shutdown_scheduler(l);
            shutdown_loaded_libs();
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            api::lua_close(l);
        }
    })) {
        log_panic("nicy_eval", p);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nicy_compile(path_ptr: *const c_char) {
    if let Err(p) = catch_unwind(AssertUnwindSafe(|| {
        if path_ptr.is_null() {
            ErrorReporter::fatal(&NicyError::panic_error("nicy_compile", "path_ptr is null"));
            return;
        }

        let c_str = unsafe { CStr::from_ptr(path_ptr) };
        let path_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => {
                ErrorReporter::fatal(&NicyError::runtime_error("Invalid path encoding"));
                return;
            }
        };

        let entry_path = match fs::canonicalize(path_str) {
            Ok(p) => p,
            Err(_) => PathBuf::from(path_str),
        };

        let source = match fs::read_to_string(&entry_path) {
            Ok(c) => c,
            Err(e) => {
                ErrorReporter::report(&NicyError::file_error(
                    entry_path.to_string_lossy().to_string(),
                    "read",
                    format!("Compile error reading file: {}", e),
                ));
                return;
            }
        };
        let (directives, code) = parse_compiler_directives(&source);

        unsafe {
            let mut options = mlua_sys::luau::lua_CompileOptions::default();
            options.optimizationLevel = directives.optimization_level;
            options.debugLevel = 1;
            options.typeInfoLevel = directives.type_info_level;
            options.coverageLevel = if directives.coverage { 1 } else { 0 };
            let bytecode_result = mlua_sys::luau::luau_compile(code.as_bytes(), options);

            if bytecode_result.is_empty() {
                ErrorReporter::fatal(&NicyError::runtime_error(
                    "Failed to generate bytecode (syntax error?)",
                ));
                return;
            }

            let out_path = entry_path.with_extension("luauc");
            if let Err(e) = fs::write(&out_path, &bytecode_result) {
                ErrorReporter::report(&NicyError::file_error(
                    out_path.to_string_lossy().to_string(),
                    "write",
                    format!("Failed to save bytecode to {}: {}", out_path.display(), e),
                ));
            } else {
                println!(
                    "[NICY] Bytecode successfully compiled to {}",
                    out_path.display()
                );
            }
        }
    })) {
        log_panic("nicy_compile", p);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nicy_version() -> *const c_char {
    RUNTIME_VERSION_LABEL.as_ptr() as *const c_char
}

#[unsafe(no_mangle)]
pub extern "C" fn nicy_luau_version() -> *const c_char {
    static LUAU_RELEASE: OnceLock<CString> = OnceLock::new();
    LUAU_RELEASE
        .get_or_init(|| {
            lua::luau_version()
                .and_then(|s| CString::new(s).ok())
                .unwrap_or_else(|| CString::new("unknown luau").unwrap())
        })
        .as_ptr()
}
