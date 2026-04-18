/*
Copyright (C) 2026 Yanlvl99 | Nicy Luau Runtime Development

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use libloading::{Library, Symbol};
use std::env;
use std::ffi::{CStr, CString};
use std::fs;
use std::path::PathBuf;

fn print_help() {
    println!("nicy - The Ultimate Luau Runtime");
    println!("Usage:");
    println!("  nicy run <script.luau>");
    println!("  nicy eval <\"code\">");
    println!("  nicy compile <script.luau>");
    println!("  nicy help");
    println!("  nicy version");
    println!("  nicy runtime-version");
}

fn runtime_library_basename() -> &'static str {
    if cfg!(target_os = "windows") {
        "nicyruntime.dll"
    } else if cfg!(target_os = "macos") {
        "libnicyruntime.dylib"
    } else {
        "libnicyruntime.so"
    }
}

#[cfg(target_os = "android")]
fn termux_prefix() -> PathBuf {
    env::var("PREFIX")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data/data/com.termux/files/usr"))
}

#[cfg(target_os = "android")]
fn android_runtime_candidates(base: &str) -> Vec<PathBuf> {
    let prefix = termux_prefix();
    vec![
        prefix.join("lib").join(base),
        prefix.join("bin").join(base),
        PathBuf::from("/data/data/com.termux/files/usr/lib").join(base),
        PathBuf::from("/data/data/com.termux/files/usr/bin").join(base),
    ]
}

#[cfg(target_os = "android")]
fn preload_android_libcxx(errors: &mut Vec<String>) {
    let prefix = termux_prefix();
    let candidates = vec![
        prefix.join("lib").join("libc++_shared.so"),
        prefix.join("bin").join("libc++_shared.so"),
        PathBuf::from("/data/data/com.termux/files/usr/lib/libc++_shared.so"),
        PathBuf::from("/data/data/com.termux/files/usr/bin/libc++_shared.so"),
    ];

    for candidate in candidates {
        if let Ok(lib) = unsafe { Library::new(&candidate) } {
            std::mem::forget(lib);
            return;
        }
    }
}

fn runtime_library_prefix_and_ext() -> (&'static str, &'static str) {
    if cfg!(target_os = "windows") {
        ("nicyruntime", ".dll")
    } else if cfg!(target_os = "macos") {
        ("libnicyruntime", ".dylib")
    } else {
        ("libnicyruntime", ".so")
    }
}

fn collect_local_library_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let base = runtime_library_basename();
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    if let Some(dir) = exe_dir {
        candidates.push(dir.join(base));

        let (prefix, ext) = runtime_library_prefix_and_ext();
        if let Ok(entries) = fs::read_dir(&dir) {
            let mut extra = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|name| name.starts_with(prefix) && name.ends_with(ext))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
            extra.sort();
            candidates.extend(extra);
        }
    }

    candidates
}

fn load_nicy_lib() -> Result<Library, String> {
    let base = runtime_library_basename();
    let mut errors = Vec::new();

    #[cfg(target_os = "android")]
    preload_android_libcxx(&mut errors);

    #[cfg(target_os = "android")]
    for candidate in android_runtime_candidates(base) {
        if let Ok(lib) = unsafe { Library::new(&candidate) } {
            return Ok(lib);
        }
    }

    for candidate in collect_local_library_candidates() {
        if let Ok(lib) = unsafe { Library::new(&candidate) } {
            return Ok(lib);
        }
    }

    unsafe { Library::new(base) }.map_err(|err| {
        errors.push(format!("PATH {}: {}", base, err));
        errors.join("\n")
    })
}

fn load_symbol<'a, T>(
    lib: &'a Library,
    symbol_name: &'static [u8],
    pretty_name: &'static str,
) -> Result<Symbol<'a, T>, String> {
    unsafe { lib.get(symbol_name) }
        .map_err(|e| format!("failed to load symbol '{}': {}", pretty_name, e))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    let command = args[1].as_str();

    match command {
        "help" | "--help" | "-h" => print_help(),
        "version" | "--version" | "-v" => println!("nicy {}", env!("CARGO_PKG_VERSION")),
        "runtime-version" | "--runtime-version" | "-rv" => {
            let lib = load_nicy_lib().map_err(|details| {
                eprintln!(
                    "[FATAL] Failed to load runtime library '{}'.\n[FATAL] Attempt details:\n{}",
                    runtime_library_basename(),
                    details
                );
                ""
            })?;

            let get_version: Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> =
                load_symbol(&lib, b"nicy_version\0", "nicy_version").map_err(|err| {
                    eprintln!("[FATAL] {}", err);
                    ""
                })?;

            let get_luau_version: Symbol<
                unsafe extern "C" fn() -> *const std::os::raw::c_char,
            > = load_symbol(&lib, b"nicy_luau_version\0", "nicy_luau_version").map_err(|err| {
                eprintln!("[FATAL] {}", err);
                ""
            })?;

            let engine_ptr = unsafe { get_version() };
            let luau_ptr = unsafe { get_luau_version() };

            if engine_ptr.is_null() || luau_ptr.is_null() {
                return Err("[FATAL] runtime returned invalid version pointers".into());
            }

            let engine_ver = unsafe { CStr::from_ptr(engine_ptr) }.to_string_lossy();
            let luau_ver = unsafe { CStr::from_ptr(luau_ptr) }.to_string_lossy();
            println!("Engine: {}", engine_ver);
            println!("Luau: {}", luau_ver);
        }
        "run" => {
            if args.len() < 3 {
                return Err("[ERROR] Missing script file. Example: nicy run script.luau".into());
            }
            execute_file(&args[2])?;
        }
        "eval" => {
            if args.len() < 3 {
                return Err(
                    "[ERROR] Missing code to evaluate. Example: nicy eval \"print('hello')\""
                        .into(),
                );
            }
            execute_eval(&args[2])?;
        }
        "compile" => {
            if args.len() < 3 {
                return Err(
                    "[ERROR] Missing script file to compile. Example: nicy compile script.luau"
                        .into(),
                );
            }
            execute_compile(&args[2])?;
        }
        _ => {
            let path = std::path::Path::new(command);
            if path.exists() {
                execute_file(command)?;
            } else {
                return Err(format!("[ERROR] Unknown command or file not found: '{}'", command).into());
            }
        }
    }

    Ok(())
}

fn execute_file(script_rel_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(script_rel_path);
    if !path.exists() {
        return Err(format!("[ERROR] File '{}' does not exist.", script_rel_path).into());
    }

    let lib = load_nicy_lib()?;

    let script_path = path
        .to_str()
        .ok_or_else(|| format!("[ERROR] Script path has invalid UTF-8: '{}'", script_rel_path))?;

    let c_path = CString::new(script_path)
        .map_err(|_| "[ERROR] Invalid script path: contains NUL byte".to_string())?;

    let start: Symbol<unsafe extern "C" fn(*const std::os::raw::c_char)> =
        load_symbol(&lib, b"nicy_start\0", "nicy_start")?;

    unsafe { start(c_path.as_ptr()) };
    Ok(())
}

fn execute_eval(code: &str) -> Result<(), Box<dyn std::error::Error>> {
    let lib = load_nicy_lib()?;

    let c_code =
        CString::new(code).map_err(|_| "[ERROR] Invalid eval code: contains NUL byte")?;

    let eval: Symbol<unsafe extern "C" fn(*const std::os::raw::c_char)> =
        load_symbol(&lib, b"nicy_eval\0", "nicy_eval")?;

    unsafe { eval(c_code.as_ptr()) };
    Ok(())
}

fn execute_compile(script_rel_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(script_rel_path);
    if !path.exists() {
        return Err(format!("[ERROR] File '{}' does not exist.", script_rel_path).into());
    }

    let lib = load_nicy_lib()?;

    let script_path = path
        .to_str()
        .ok_or_else(|| format!("[ERROR] Script path has invalid UTF-8: '{}'", script_rel_path))?;

    let c_path = CString::new(script_path)
        .map_err(|_| "[ERROR] Invalid script path: contains NUL byte")?;

    let compile: Symbol<unsafe extern "C" fn(*const std::os::raw::c_char)> =
        load_symbol(&lib, b"nicy_compile\0", "nicy_compile")?;

    unsafe { compile(c_path.as_ptr()) };
    Ok(())
}
