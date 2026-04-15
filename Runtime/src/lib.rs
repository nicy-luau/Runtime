/*
Copyright (C) 2026 Yanlvl99 | Nicy Luau Runtime Development

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use libloading::{Library, Symbol};
use mlua_sys::luau::lauxlib;
use mlua_sys::luau::lualib;
use mlua_sys::luau::{compat, lua};
use std::ffi::CStr;
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

mod api;
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

type LuauState = lua::lua_State;

unsafe fn push_loadlib_error(l: *mut LuauState, msg: &str) -> c_int {
    let filtered = msg.replace('\0', "?");

    // Also emit a warn so the error is visible even if the script doesn't check the return value
    ErrorReporter::warn(&format!("runtime.loadlib failed: {}", filtered));

    unsafe { lua::lua_pushnil(l) };
    unsafe {
        compat::lua_pushlstring(
            l,
            filtered.as_ptr() as *const c_char,
            filtered.len(),
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

pub(crate) fn panic_payload_to_string(p: Box<dyn std::any::Any + Send>) -> String {
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
            if let Ok(n) = rest.trim().parse::<i32>() {
                directives.optimization_level = n.clamp(0, 2);
                directive_end_line = i + 1;
            }
        } else if trimmed == "--!coverage" || trimmed.starts_with("--!coverage ") {
            directives.coverage = true;
            directive_end_line = i + 1;
        } else if trimmed == "--!profile" || trimmed.starts_with("--!profile ") {
            directives.profile = true;
            directive_end_line = i + 1;
        } else if let Some(rest) = trimmed.strip_prefix("--!typeinfo") {
            if let Ok(n) = rest.trim().parse::<i32>() {
                directives.type_info_level = n.clamp(0, 1);
                directive_end_line = i + 1;
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
    let p = unsafe { lua::lua_tostring(l, idx) };
    if p.is_null() {
        return "nil".to_string();
    }
    unsafe { CStr::from_ptr(p) }.to_string_lossy().to_string()
}

unsafe extern "C-unwind" fn nicy_runtime_warn(l: *mut LuauState) -> c_int {
    let top = unsafe { lua::lua_gettop(l) };
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
        lua::lua_getfield(
            l,
            lua::LUA_REGISTRYINDEX,
            c"nicy_ext_cache".as_ptr() as *const c_char,
        )
    };
    if unsafe { lua::lua_type(l, -1) } != lua::LUA_TTABLE {
        // Table doesn't exist: remove nil and create new
        unsafe { lua::lua_settop(l, -2) };

        unsafe { lua::lua_createtable(l, 0, 0) };
        unsafe { lua::lua_createtable(l, 0, 1) };
        unsafe { compat::lua_pushstring(l, c"v".as_ptr() as *const c_char) };
        unsafe { lua::lua_setfield(l, -2, c"__mode".as_ptr() as *const c_char) };
        unsafe { lua::lua_setmetatable(l, -2) };

        unsafe { lua::lua_pushvalue(l, -1) };
        unsafe {
            lua::lua_setfield(
                l,
                lua::LUA_REGISTRYINDEX,
                c"nicy_ext_cache".as_ptr() as *const c_char,
            )
        };
    } else {
        // Table already exists: remove from stack to balance
        unsafe { lua::lua_pop(l, 1) };
    }
}

unsafe extern "C-unwind" fn nicy_runtime_loadlib(l: *mut LuauState) -> c_int {
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        let path_ptr = lauxlib::luaL_checkstring(l, 1);
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
        let cache_idx = lua::lua_absindex(l, -1);
        compat::lua_pushlstring(
            l,
            resolved_key.as_ptr() as *const c_char,
            resolved_key.len(),
        );
        lua::lua_gettable(l, cache_idx);
        if lua::lua_type(l, -1) != lua::LUA_TNIL {
            lua::lua_remove(l, cache_idx);
            return Ok(1);
        }
        lua::lua_settop(l, -2);

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
        let cache_idx = lua::lua_absindex(l, -1);
        compat::lua_pushlstring(
            l,
            resolved_key.as_ptr() as *const c_char,
            resolved_key.len(),
        );
        lua::lua_pushvalue(l, -3);
        lua::lua_settable(l, cache_idx);
        lua::lua_remove(l, cache_idx);

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
    let top = unsafe { lua::lua_gettop(l) };
    let spec = if top >= 1 {
        let ptr = unsafe { lauxlib::luaL_checkstring(l, 1) };
        if ptr.is_null() {
            None
        } else {
            unsafe { CStr::from_ptr(ptr) }.to_str().ok()
        }
    } else {
        None
    };

    let enabled = require_resolver::has_jit(l, spec);
    unsafe { lua::lua_pushboolean(l, enabled as c_int) };
    1
}

/// # Safety
/// Caller must ensure `l` is a valid, non-null pointer to an open `lua_State`.
pub unsafe fn safe_pcall(
    l: *mut LuauState,
    nargs: c_int,
    nresults: c_int,
    errfunc: c_int,
) -> c_int {
    ErrorReporter::enter_pcall(l);
    let result = unsafe { lua::lua_pcall(l, nargs, nresults, errfunc) };
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
            c"error_handler".as_ptr() as *const _,
        )
    };

    if status != 0 {
        // Load failed: error message is on stack, pop it
        unsafe { lua::lua_pop(l, 1) };
        return 0;
    }

    // Load succeeded: function is on stack, call it
    let pcall_status = unsafe { lua::lua_pcall(l, 0, 1, 0) };
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

unsafe fn push_nicy_table(l: *mut LuauState, entry_path: &Path) {
    let luau_ver_ptr = nicy_luau_version();

    unsafe {
        lua::lua_getglobal(l, c"_G".as_ptr() as *const c_char);
        if lua::lua_type(l, -1) == lua::LUA_TTABLE {
            compat::lua_pushstring(l, luau_ver_ptr);
            lua::lua_setfield(l, -2, c"_VERSION".as_ptr() as *const c_char);
        }
        lua::lua_pop(l, 1);
    }

    unsafe { lua::lua_createtable(l, 0, 5) };

    unsafe { compat::lua_pushstring(l, RUNTIME_VERSION.as_ptr() as *const c_char) };
    unsafe { lua::lua_setfield(l, -2, c"version".as_ptr() as *const c_char) };

    unsafe { lua::lua_pushcfunction(l, nicy_runtime_loadlib) };
    unsafe { lua::lua_setfield(l, -2, c"loadlib".as_ptr() as *const c_char) };
    unsafe { lua::lua_pushcfunction(l, nicy_runtime_has_jit) };
    unsafe { lua::lua_setfield(l, -2, c"hasJIT".as_ptr() as *const c_char) };

    let entry_file = entry_path.to_string_lossy().to_string();
    unsafe {
        compat::lua_pushlstring(
            l,
            entry_file.as_ptr() as *const c_char,
            entry_file.len(),
        )
    };
    unsafe { lua::lua_setfield(l, -2, c"entry_file".as_ptr() as *const c_char) };

    let entry_dir = entry_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    unsafe {
        compat::lua_pushlstring(
            l,
            entry_dir.as_ptr() as *const c_char,
            entry_dir.len(),
        )
    };
    unsafe { lua::lua_setfield(l, -2, c"entry_dir".as_ptr() as *const c_char) };

    unsafe { lua::lua_setglobal(l, c"runtime".as_ptr() as *const c_char) };

    unsafe { lua::lua_getglobal(l, c"warn".as_ptr() as *const c_char) };
    if unsafe { lua::lua_type(l, -1) } == lua::LUA_TNIL {
        unsafe { lua::lua_settop(l, -2) };
        unsafe { lua::lua_pushcfunction(l, nicy_runtime_warn) };
        unsafe { lua::lua_setglobal(l, c"warn".as_ptr() as *const c_char) };
    } else {
        unsafe { lua::lua_settop(l, -2) };
    }
}

// ── OS library extensions ───────────────────────────────────────

unsafe extern "C-unwind" fn os_exit(l: *mut LuauState) -> c_int {
    let code = if unsafe { lua::lua_gettop(l) } >= 1 {
        unsafe { lua::lua_tonumber(l, 1) as i32 }
    } else {
        0
    };
    std::process::exit(code);
}

unsafe extern "C-unwind" fn os_getenv(l: *mut LuauState) -> c_int {
    let name_ptr = unsafe { lauxlib::luaL_checkstring(l, 1) };
    if name_ptr.is_null() {
        unsafe { lua::lua_pushnil(l) };
        return 1;
    }
    let name = unsafe { CStr::from_ptr(name_ptr) }.to_string_lossy();
    match std::env::var_os(&*name) {
        Some(val) => {
            let val_str = val.to_string_lossy();
            unsafe { compat::lua_pushlstring(l, val_str.as_ptr() as *const c_char, val_str.len()) };
        }
        None => unsafe { lua::lua_pushnil(l) },
    }
    1
}

unsafe extern "C-unwind" fn os_remove(l: *mut LuauState) -> c_int {
    let path_ptr = unsafe { lauxlib::luaL_checkstring(l, 1) };
    if path_ptr.is_null() {
        unsafe { lua::lua_pushboolean(l, 0) };
        return 1;
    }
    let path = unsafe { CStr::from_ptr(path_ptr) }.to_string_lossy();
    let success = fs::remove_file(&*path).is_ok() || fs::remove_dir(&*path).is_ok();
    unsafe { lua::lua_pushboolean(l, if success { 1 } else { 0 }) };
    1
}

unsafe extern "C-unwind" fn os_rename(l: *mut LuauState) -> c_int {
    let old_ptr = unsafe { lauxlib::luaL_checkstring(l, 1) };
    let new_ptr = unsafe { lauxlib::luaL_checkstring(l, 2) };
    if old_ptr.is_null() || new_ptr.is_null() {
        unsafe { lua::lua_pushboolean(l, 0) };
        return 1;
    }
    let old_path = unsafe { CStr::from_ptr(old_ptr) }
        .to_string_lossy()
        .to_string();
    let new_path = unsafe { CStr::from_ptr(new_ptr) }
        .to_string_lossy()
        .to_string();
    let success = fs::rename(&old_path, &new_path).is_ok();
    unsafe { lua::lua_pushboolean(l, if success { 1 } else { 0 }) };
    1
}

unsafe extern "C-unwind" fn os_sleep(l: *mut LuauState) -> c_int {
    let ms = if unsafe { lua::lua_gettop(l) } >= 1 {
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
    let path_str = full_path.to_string_lossy();
    unsafe {
        compat::lua_pushlstring(
            l,
            path_str.as_ptr() as *const c_char,
            path_str.len(),
        )
    };
    1
}

/// Extend the standard `os` library with additional functions.
/// Must be called AFTER luaL_openlibs so that the `os` table already exists.
unsafe fn extend_os_library(l: *mut LuauState) {
    unsafe { lua::lua_getglobal(l, c"os".as_ptr() as *const c_char) };

    unsafe { lua::lua_pushcfunction(l, os_exit) };
    unsafe { lua::lua_setfield(l, -2, c"exit".as_ptr() as *const c_char) };

    unsafe { lua::lua_pushcfunction(l, os_getenv) };
    unsafe { lua::lua_setfield(l, -2, c"getenv".as_ptr() as *const c_char) };

    unsafe { lua::lua_pushcfunction(l, os_remove) };
    unsafe { lua::lua_setfield(l, -2, c"remove".as_ptr() as *const c_char) };

    unsafe { lua::lua_pushcfunction(l, os_rename) };
    unsafe { lua::lua_setfield(l, -2, c"rename".as_ptr() as *const c_char) };

    unsafe { lua::lua_pushcfunction(l, os_sleep) };
    unsafe { lua::lua_setfield(l, -2, c"sleep".as_ptr() as *const c_char) };

    unsafe { lua::lua_pushcfunction(l, os_tmpname) };
    unsafe { lua::lua_setfield(l, -2, c"tmpname".as_ptr() as *const c_char) };

    unsafe { lua::lua_pop(l, 1) };
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
        let opt_ptr = lauxlib::luaL_checkstring(l, 1);
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
                lua::lua_pushboolean(l, if running != 0 { 1 } else { 0 });
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
                let size = if lua::lua_gettop(l) >= 2 {
                    (lauxlib::luaL_checkinteger_(l, 2)) as c_int
                } else {
                    100 // default step size
                };
                let result = lua::lua_gc(l, LUA_GCSTEP, size);
                lua::lua_pushboolean(l, if result != 0 { 1 } else { 0 });
                Ok(1)
            }
            _ => Err(format!("invalid option '{}'", opt)),
        }
    }));

    match result {
        Ok(Ok(n)) => n,
        Ok(Err(msg)) => unsafe {
            ErrorReporter::warn(&format!("collectgarbage: {}", msg));
            lua::lua_pushnil(l);
            compat::lua_pushlstring(l, msg.as_ptr() as *const c_char, msg.len());
            2
        },
        Err(p) => unsafe {
            let msg = format!("panic in collectgarbage: {}", panic_payload_to_string(p));
            ErrorReporter::report(&NicyError::PanicError {
                context: "collectgarbage",
                payload: msg.clone(),
            });
            lua::lua_pushnil(l);
            compat::lua_pushlstring(l, msg.as_ptr() as *const c_char, msg.len());
            2
        },
    }
}

/// Inject `collectgarbage` as a global function.
/// Luau's luaL_openlibs does NOT include collectgarbage, so we add it manually.
unsafe fn extend_collectgarbage(l: *mut LuauState) {
    unsafe { lua::lua_pushcfunction(l, nicy_collectgarbage) };
    unsafe { lua::lua_setglobal(l, c"collectgarbage".as_ptr() as *const c_char) };
}

/// # Safety
/// Caller must ensure `path_ptr` is a valid, non-null C string pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nicy_start(path_ptr: *const c_char) {
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
            let l = lauxlib::luaL_newstate();
            if l.is_null() {
                ErrorReporter::fatal(&NicyError::runtime_error("Failed to create Luau state"));
                return;
            }

            lualib::luaL_openlibs(l);
            // Extend the standard os library with native functions
            extend_os_library(l);
            // Inject collectgarbage global function (not included by luaL_openlibs in Luau)
            extend_collectgarbage(l);
            let entry_jit_enabled = !code_is_bytecode
                && entry_native_requested
                && require_resolver::ensure_codegen_context(l);
            task_scheduler::mark_current_state_valid(l);
            task_scheduler::init(l);
            push_nicy_table(l, &entry_path);
            require_resolver::install_require(l);
            if let Err(e) = require_resolver::init_runtime(l, &entry_path) {
                ErrorReporter::report(&NicyError::runtime_error(format!(
                    "Failed to init runtime: {}",
                    e
                )));
                lua::lua_close(l);
                return;
            }
            if let Err(e) = require_resolver::set_entry_jit(l, entry_jit_enabled) {
                ErrorReporter::report(&NicyError::runtime_error(format!(
                    "Failed to set entry jit: {}",
                    e
                )));
                require_resolver::shutdown_runtime(l);
                lua::lua_close(l);
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
                compat::luaL_loadbuffer(
                    l,
                    code_bytes.as_ptr() as *const c_char,
                    code_bytes.len(),
                    chunkname.as_ptr() as *const c_char,
                )
            };
            if load_status != 0 {
                ErrorReporter::report_lua_error(l, "load");
                require_resolver::shutdown_runtime(l);
                lua::lua_close(l);
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
                lua::lua_close(l);
                return;
            }

            // Install error handler for better error reporting
            let _err_handler_ref = install_error_handler(l);

            task_scheduler::schedule_main_thread(l);
            lua::lua_settop(l, -2);

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
            lua::lua_pushnil(l);
            lua::lua_setfield(
                l,
                lua::LUA_REGISTRYINDEX,
                c"nicy_ext_cache".as_ptr() as *const c_char,
            );

            require_resolver::shutdown_all_globals(l);
            task_scheduler::shutdown_scheduler(l);
            task_scheduler::mark_current_state_invalid();
            shutdown_loaded_libs();
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            lua::lua_close(l);
        }
    })) {
        log_panic("nicy_start", p);
    }
}

/// # Safety
/// Caller must ensure `code_ptr` is a valid, non-null C string pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nicy_eval(code_ptr: *const c_char) {
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
            let l = lauxlib::luaL_newstate();
            if l.is_null() {
                ErrorReporter::fatal(&NicyError::runtime_error(
                    "Failed to create Luau state for eval",
                ));
                return;
            }

            lualib::luaL_openlibs(l);
            // Extend the standard os library with native functions
            extend_os_library(l);
            // Inject collectgarbage global function
            extend_collectgarbage(l);

            task_scheduler::mark_current_state_valid(l);
            task_scheduler::init(l);

            let eval_path = PathBuf::from("eval");

            require_resolver::install_require(l);
            let _ = require_resolver::init_runtime(l, &eval_path);

            push_nicy_table(l, &eval_path);

            let chunkname = b"eval\0";
            let load_status = compat::luaL_loadbuffer(
                l,
                code_str.as_ptr() as *const c_char,
                code_str.len(),
                chunkname.as_ptr() as *const c_char,
            );

            if load_status != 0 {
                ErrorReporter::report_lua_error(l, "eval compile");
                require_resolver::shutdown_runtime(l);
                lua::lua_close(l);
                return;
            }

            let call_status = lua::lua_pcall(l, 0, 0, 0);
            if call_status != 0 {
                ErrorReporter::report_lua_error(l, "eval");
            }

            task_scheduler::run_until_idle(l);

            // FIX (LEAK-2): Remove nicy_ext_cache table from registry.
            lua::lua_pushnil(l);
            lua::lua_setfield(
                l,
                lua::LUA_REGISTRYINDEX,
                c"nicy_ext_cache".as_ptr() as *const c_char,
            );

            require_resolver::shutdown_all_globals(l);
            task_scheduler::shutdown_scheduler(l);
            task_scheduler::mark_current_state_invalid();
            shutdown_loaded_libs();
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            lua::lua_close(l);
        }
    })) {
        log_panic("nicy_eval", p);
    }
}

/// # Safety
/// Caller must ensure `path_ptr` is a valid, non-null C string pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nicy_compile(path_ptr: *const c_char) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_native_directive() {
        let (native, code) = strip_native_directive("--!native\nprint('hello')");
        assert!(native);
        assert_eq!(code, "print('hello')");
    }

    #[test]
    fn test_no_native_directive() {
        let (native, code) = strip_native_directive("print('hello')");
        assert!(!native);
        assert_eq!(code, "print('hello')");
    }

    #[test]
    fn test_empty_source() {
        let (native, code) = strip_native_directive("");
        assert!(!native);
        assert_eq!(code, "");
    }

    #[test]
    fn test_parse_compiler_directives_native() {
        let (d, code) = parse_compiler_directives("--!native\nreturn 1");
        assert!(d.native);
        assert_eq!(code, "return 1");
    }

    #[test]
    fn test_parse_compiler_directives_optimize() {
        let (d, code) = parse_compiler_directives("--!optimize 2\nreturn 1");
        assert_eq!(d.optimization_level, 2);
        assert!(!d.native);
        assert_eq!(code, "return 1");
    }

    #[test]
    fn test_parse_compiler_directives_multiple() {
        let (d, code) = parse_compiler_directives(
            "--!native\n--!optimize 2\n--!coverage\n--!profile\n--!typeinfo 1\nreturn 1",
        );
        assert!(d.native);
        assert_eq!(d.optimization_level, 2);
        assert!(d.coverage);
        assert!(d.profile);
        assert_eq!(d.type_info_level, 1);
        assert_eq!(code, "return 1");
    }

    #[test]
    fn test_parse_compiler_directives_skips_unknown() {
        let (d, code) = parse_compiler_directives("--!strict\n--!native\nreturn 1");
        // --!strict is unknown but still consumed as directive block
        assert!(d.native);
        assert_eq!(code, "return 1");
    }

    #[test]
    fn test_parse_compiler_directives_no_directives() {
        let (d, code) = parse_compiler_directives("return 1");
        assert!(!d.native);
        assert_eq!(d.optimization_level, 1);
        assert_eq!(code, "return 1");
    }

    #[test]
    fn test_parse_compiler_directives_optimize_clamp() {
        let (d, _) = parse_compiler_directives("--!optimize 9\nreturn 1");
        assert_eq!(d.optimization_level, 2); // clamped to max 2
    }

    #[test]
    fn test_panic_payload_string() {
        assert_eq!(panic_payload_to_string(Box::new("test message")), "test message");
        assert_eq!(
            panic_payload_to_string(Box::new(String::from("owned string"))),
            "owned string"
        );
        assert_eq!(
            panic_payload_to_string(Box::new(42i32)),
            "non-string panic payload"
        );
    }
}
