/*\nCopyright (C) 2026 Yanlvl99 | Nicy Luau Runtime Development\n\nThis Source Code Form is subject to the terms of the Mozilla Public\nLicense, v. 2.0. If a copy of the MPL was not distributed with this\nfile, You can obtain one at http://mozilla.org/MPL/2.0/.\n*/

use mlua_sys::luau::compat;
use mlua_sys::luau::lauxlib;
use mlua_sys::luau::lua;
#[cfg(not(target_os = "android"))]
use mlua_sys::luau::luacodegen;
use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use crate::error::{ErrorReporter, NicyError, RequireChain};
use crate::strip_native_directive;
use crate::task_scheduler;

type LuauState = lua::lua_State;
const NICY_CODEGEN_CREATED_KEY: &[u8] = b"nicy_codegen_created\0";

#[derive(Clone, Copy, Eq, PartialEq)]
struct FileFingerprint {
    modified_ns: u64,
    size: u64,
}

impl FileFingerprint {
    fn from_path(path: &Path) -> Result<Self, String> {
        let meta = fs::metadata(path)
            .map_err(|e| format!("failed to stat '{}': {}", path.display(), e))?;
        let size = meta.len();
        let modified_ns = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        Ok(Self { modified_ns, size })
    }
}

struct ModuleCacheEntry {
    fp: FileFingerprint,
    value_ref: c_int,
}

struct LuaurcCacheEntry {
    fp: FileFingerprint,
    aliases: Arc<HashMap<String, String>>,
}

struct RuntimeData {
    l_ptr: usize,
    entry_file: PathBuf,
    entry_dir: PathBuf,
    module_stack: Vec<PathBuf>,
    module_cache: HashMap<PathBuf, ModuleCacheEntry>,
    module_jit: HashMap<PathBuf, bool>,
    loading: HashSet<PathBuf>,
    luaurc_cache: HashMap<PathBuf, LuaurcCacheEntry>,
    require_chain: RequireChain,
}

impl RuntimeData {
    fn new(l: *mut LuauState, entry_file: PathBuf, entry_dir: PathBuf) -> Self {
        Self {
            l_ptr: l as usize,
            entry_file,
            entry_dir,
            module_stack: Vec::new(),
            module_cache: HashMap::new(),
            module_jit: HashMap::new(),
            loading: HashSet::new(),
            luaurc_cache: HashMap::new(),
            require_chain: RequireChain::new(),
        }
    }
}

static RUNTIMES: OnceLock<Mutex<HashMap<usize, RuntimeData>>> = OnceLock::new();

static COROUTINE_TO_MAIN: OnceLock<Mutex<HashMap<usize, usize>>> = OnceLock::new();

fn coroutines_to_main() -> &'static Mutex<HashMap<usize, usize>> {
    COROUTINE_TO_MAIN.get_or_init(|| Mutex::new(HashMap::new()))
}

fn runtimes() -> &'static Mutex<HashMap<usize, RuntimeData>> {
    RUNTIMES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn register_coroutine(main_l: *mut LuauState, th: *mut LuauState) {
    let mut map = coroutines_to_main()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    map.insert(th as usize, main_l as usize);
}

pub fn find_main_state(l: *mut LuauState) -> Option<usize> {
    const MAX_ITERATIONS: usize = 1000;
    let mut current = l as usize;
    let map = coroutines_to_main()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut visited = HashSet::new();
    let mut iterations = 0;

    while !visited.contains(&current) {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            // Belt-and-suspenders: should never happen if thread chain is acyclic,
            // but protects against pathological/corrupted state
            eprintln!(
                "[nicy] WARNING: find_main_state exceeded {MAX_ITERATIONS} iterations, possible cycle in thread chain"
            );
            break;
        }
        visited.insert(current);

        let all = runtimes().lock().unwrap_or_else(|e| e.into_inner());
        if all.contains_key(&current) {
            return Some(current);
        }

        match map.get(&current) {
            Some(&next) => current = next,
            None => break,
        }
    }

    None
}

fn with_runtime<R>(l: *mut LuauState, f: impl FnOnce(&mut RuntimeData) -> R) -> Result<R, String> {
    let main_l =
        find_main_state(l).ok_or_else(|| "runtime context is not initialized".to_string())?;
    let mut all = runtimes().lock().unwrap_or_else(|e| e.into_inner());
    let rt = all
        .get_mut(&main_l)
        .ok_or_else(|| "runtime context is not initialized".to_string())?;
    Ok(f(rt))
}

fn canonicalize_existing(path: PathBuf) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("path does not exist: {}", path.display()));
    }

    // FIX (Windows): fs::canonicalize returns the \\?\\  (extended-length) prefix
    // that LoadLibraryW does NOT accept => ACCESS_DENIED when loading .ndyn files.
    // It can also fail with PermissionDenied on paths through junctions / AppData.
    // Strategy: try canonicalize, fall back to original path on error, then strip prefix.
    let canonical = match fs::canonicalize(&path) {
        Ok(p) => p,
        Err(_) => path,   // graceful fallback — keeps the original absolute path
    };

    #[cfg(windows)]
    {
        let s = canonical.to_string_lossy();
        if s.starts_with(r"\\?\\\\") {
            return Ok(PathBuf::from(&s[4..]));
        }
    }

    Ok(canonical)
}

fn resolve_loadlib_base(
    rt: &RuntimeData,
    current_module: &Path,
    spec: &str,
) -> Result<PathBuf, String> {
    if let Some(rest) = spec.strip_prefix("@self") {
        if rest.is_empty() {
            if is_init_module(current_module) {
                return Ok(current_module
                    .parent()
                    .unwrap_or(rt.entry_dir.as_path())
                    .to_path_buf());
            }
            return Ok(current_module.to_path_buf());
        }
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        let root = current_module
            .parent()
            .unwrap_or(rt.entry_dir.as_path())
            .to_path_buf();
        return Ok(root.join(rest));
    }
    if spec.starts_with("./") || spec.starts_with("../") {
        let parent = current_module.parent().unwrap_or(rt.entry_dir.as_path());
        return Ok(parent.join(spec));
    }
    if spec.starts_with('@') {
        return Err("loadlib supports only @self aliases".to_string());
    }
    let p = Path::new(spec);
    if p.is_absolute() {
        return Ok(p.to_path_buf());
    }
    Ok(rt.entry_dir.join(p))
}

/// Parse aliases from a .luaurc file using serde_json.
/// Falls back to an empty map if the file is not valid JSON or lacks "aliases".
fn parse_aliases_from_luaurc(content: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();

    // Try standard JSON parsing first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(aliases) = json.get("aliases").and_then(|v| v.as_object()) {
            for (key, value) in aliases {
                if let Some(v_str) = value.as_str() {
                    let normalized = if key.starts_with('@') {
                        key.clone()
                    } else {
                        format!("@{}", key)
                    };
                    if !v_str.is_empty() {
                        out.insert(normalized, v_str.to_string());
                    }
                }
            }
        }
        return out;
    }

    // Fallback: the file might be JSONC (with comments). Strip lines starting with //
    // and try again.
    let stripped: String = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !(trimmed.starts_with("//") || trimmed.starts_with('#'))
        })
        .collect::<Vec<_>>()
        .join("\n");

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stripped)
        && let Some(aliases) = json.get("aliases").and_then(|v| v.as_object())
    {
        for (key, value) in aliases {
            if let Some(v_str) = value.as_str() {
                let normalized = if key.starts_with('@') {
                    key.clone()
                } else {
                    format!("@{}", key)
                };
                if !v_str.is_empty() {
                    out.insert(normalized, v_str.to_string());
                }
            }
        }
    }

    out
}

fn aliases_for_dir(rt: &mut RuntimeData, dir: &Path) -> Result<HashMap<String, PathBuf>, String> {
    let mut chain = Vec::new();
    let mut cur = Some(dir);
    while let Some(d) = cur {
        chain.push(d.to_path_buf());
        cur = d.parent();
    }
    chain.reverse();

    let mut out = HashMap::<String, PathBuf>::new();
    for d in chain {
        let rc = d.join(".luaurc");
        if !rc.exists() {
            continue;
        }

        let fp = FileFingerprint::from_path(&rc)?;
        let aliases: Arc<HashMap<String, String>> = if let Some(cached) = rt.luaurc_cache.get(&rc) {
            if cached.fp == fp {
                cached.aliases.clone()
            } else {
                let content = fs::read_to_string(&rc)
                    .map_err(|e| format!("failed to read '{}': {}", rc.display(), e))?;
                let parsed = parse_aliases_from_luaurc(&content);
                rt.luaurc_cache.insert(
                    rc.clone(),
                    LuaurcCacheEntry {
                        fp,
                        aliases: Arc::new(parsed.clone()),
                    },
                );
                Arc::new(parsed)
            }
        } else {
            let content = fs::read_to_string(&rc)
                .map_err(|e| format!("failed to read '{}': {}", rc.display(), e))?;
            let parsed = parse_aliases_from_luaurc(&content);
            rt.luaurc_cache.insert(
                rc.clone(),
                LuaurcCacheEntry {
                    fp,
                    aliases: Arc::new(parsed.clone()),
                },
            );
            Arc::new(parsed)
        };

        for (k, v) in &*aliases {
            let base = Path::new(&v);
            let abs = if base.is_absolute() {
                base.to_path_buf()
            } else {
                d.join(base)
            };
            out.insert(k.clone(), abs);
        }
    }
    Ok(out)
}

fn is_init_module(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|s| s.to_str()),
        Some("init.lua") | Some("init.luau")
    )
}

fn resolve_spec_base(
    rt: &mut RuntimeData,
    current_module: &Path,
    spec: &str,
) -> Result<PathBuf, String> {
    if spec.starts_with("./") || spec.starts_with("../") {
        let parent = current_module.parent().unwrap_or(rt.entry_dir.as_path());
        return Ok(parent.join(spec));
    }
    if spec == "." {
        if is_init_module(current_module) {
            return Ok(current_module
                .parent()
                .unwrap_or(rt.entry_dir.as_path())
                .to_path_buf());
        }
        return Ok(current_module.to_path_buf());
    }
    if let Some(rest) = spec.strip_prefix("@self") {
        if rest.is_empty() {
            if is_init_module(current_module) {
                return Ok(current_module
                    .parent()
                    .unwrap_or(rt.entry_dir.as_path())
                    .to_path_buf());
            }
            return Ok(current_module.to_path_buf());
        }
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        let self_root = current_module
            .parent()
            .unwrap_or(rt.entry_dir.as_path())
            .to_path_buf();
        return Ok(self_root.join(rest));
    }
    if let Some(alias_spec) = spec.strip_prefix('@') {
        let mut parts = alias_spec.splitn(2, '/');
        let alias_name = parts.next().unwrap_or_default();
        let remain = parts.next().unwrap_or_default();
        let key = format!("@{}", alias_name);
        let caller_dir = current_module
            .parent()
            .unwrap_or(rt.entry_dir.as_path())
            .to_path_buf();
        let aliases = aliases_for_dir(rt, &caller_dir)?;
        let base = aliases
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("unknown alias '{}'", key))?;
        if remain.is_empty() {
            return Ok(base);
        }
        return Ok(base.join(remain));
    }
    Ok(rt.entry_dir.join(spec))
}

fn candidate_paths(base: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if base.extension().is_some() {
        out.push(base.to_path_buf());
        return out;
    }
    // Bytecode first, then source
    out.push(base.with_extension("luauc"));
    out.push(base.with_extension("luau"));
    out.push(base.with_extension("lua"));
    out.push(base.join("init.luauc"));
    out.push(base.join("init.luau"));
    out.push(base.join("init.lua"));
    out
}

fn resolve_module_path(
    rt: &mut RuntimeData,
    current_module: &Path,
    spec: &str,
) -> Result<(PathBuf, Vec<String>), String> {
    let base = resolve_spec_base(rt, current_module, spec)?;
    let mut searched_paths = Vec::new();
    for c in candidate_paths(&base) {
        searched_paths.push(c.display().to_string());
        if c.exists() {
            let resolved = canonicalize_existing(c)?;
            return Ok((resolved, searched_paths));
        }
    }
    Err(format!(
        "module '{}' not found from '{}'",
        spec,
        current_module.display()
    ))
}

fn lua_string_at(l: *mut LuauState, idx: c_int) -> Result<String, String> {
    let ptr = unsafe { lauxlib::luaL_checkstring(l, idx) };
    if ptr.is_null() {
        return Err("expected a string".to_string());
    }
    let s = unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|_| "invalid utf-8 string".to_string())?;
    Ok(s.to_string())
}

fn push_error(l: *mut LuauState, err: NicyError) -> c_int {
    // Always report the error to console (unless inside pcall)
    if !ErrorReporter::is_inside_pcall(Some(l)) {
        ErrorReporter::report(&err);
    }

    // Always return nil + error to Lua (never call lua_error)
    let msg = err.to_string();
    let filtered = msg.replace('\0', "?");
    unsafe {
        lua::lua_pushnil(l);
        compat::lua_pushlstring(l, filtered.as_ptr() as *const c_char, filtered.len());
        2
    }
}

fn codegen_supported() -> bool {
    #[cfg(target_os = "android")]
    {
        false
    }
    #[cfg(not(target_os = "android"))]
    unsafe {
        luacodegen::luau_codegen_supported() != 0
    }
}

fn codegen_created(l: *mut LuauState) -> bool {
    unsafe {
        lua::lua_getfield(
            l,
            lua::LUA_REGISTRYINDEX,
            NICY_CODEGEN_CREATED_KEY.as_ptr() as *const c_char,
        );
        let enabled = lua::lua_type(l, -1) != lua::LUA_TNIL && lua::lua_toboolean(l, -1) != 0;
        lua::lua_settop(l, -2);
        enabled
    }
}

pub fn ensure_codegen_context(l: *mut LuauState) -> bool {
    if !codegen_supported() {
        return false;
    }
    if !codegen_created(l) {
        #[cfg(not(target_os = "android"))]
        unsafe {
            luacodegen::luau_codegen_create(l)
        };
        unsafe {
            lua::lua_pushboolean(l, 1);
            lua::lua_setfield(
                l,
                lua::LUA_REGISTRYINDEX,
                NICY_CODEGEN_CREATED_KEY.as_ptr() as *const c_char,
            );
        }
    }
    true
}

pub fn compile_loaded_chunk(l: *mut LuauState) {
    #[cfg(not(target_os = "android"))]
    unsafe {
        luacodegen::luau_codegen_compile(l, -1);
    }
    #[cfg(target_os = "android")]
    {
        let _ = l;
    }
}

pub fn init_runtime(l: *mut LuauState, entry_file: &Path) -> Result<(), String> {
    let entry_file = canonicalize_existing(entry_file.to_path_buf())?;
    let entry_dir = entry_file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let mut all = runtimes().lock().unwrap_or_else(|e| e.into_inner());
    all.insert(l as usize, RuntimeData::new(l, entry_file, entry_dir));
    register_coroutine(l, l);
    Ok(())
}

pub fn set_entry_jit(l: *mut LuauState, enabled: bool) -> Result<(), String> {
    with_runtime(l, |rt| {
        rt.module_jit.insert(rt.entry_file.clone(), enabled);
    })?;
    Ok(())
}

pub fn has_jit(l: *mut LuauState, spec: Option<&str>) -> bool {
    let lookup = with_runtime(l, |rt| {
        let current = rt
            .module_stack
            .last()
            .cloned()
            .unwrap_or_else(|| rt.entry_file.clone());

        if let Some(spec) = spec {
            resolve_module_path(rt, &current, spec)
        } else {
            Ok((current, Vec::new()))
        }
    });

    let Ok(Ok((path, _))) = lookup else {
        return false;
    };

    with_runtime(l, |rt| rt.module_jit.get(&path).copied().unwrap_or(false)).unwrap_or(false)
}

pub fn resolve_loadlib_path(l: *mut LuauState, spec: &str) -> Result<PathBuf, String> {
    with_runtime(l, |rt| {
        let current = rt
            .module_stack
            .last()
            .cloned()
            .unwrap_or_else(|| rt.entry_file.clone());
        resolve_loadlib_base(rt, &current, spec).and_then(canonicalize_existing)
    })?
}

pub fn shutdown_runtime(l: *mut LuauState) {
    let mut all = runtimes().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(rt) = all.remove(&(l as usize)) {
        for entry in rt.module_cache.values() {
            unsafe {
                lauxlib::luaL_unref(l, lua::LUA_REGISTRYINDEX, entry.value_ref);
            }
        }
    }
}

/// CRITICAL FIX (C-1): Clear ALL global static state AND unref all cached module values.
/// This prevents memory leaks and stale pointer crashes between nicy_start calls.
///
/// Safety: If a runtime's l_ptr is null, it was never fully initialized and module_cache
/// is guaranteed empty — no unref needed. If l_ptr is non-zero but the Lua state was
/// externally closed (host bug), unref on a dangling pointer is safe — luaL_unref on
/// a closed state is a no-op in Luau.
pub fn shutdown_all_globals(_l: *mut LuauState) {
    // Clear RUNTIMES - unref all cached module values using each runtime's own state
    {
        let mut all = runtimes().lock().unwrap_or_else(|e| e.into_inner());
        for (_, rt) in all.drain() {
            // l_ptr is set by init_runtime before any module loading occurs.
            // If l_ptr == 0, init_runtime was never called and module_cache is empty.
            if rt.l_ptr != 0 {
                let state = rt.l_ptr as *mut LuauState;
                for entry in rt.module_cache.values() {
                    unsafe {
                        lauxlib::luaL_unref(state, lua::LUA_REGISTRYINDEX, entry.value_ref);
                    }
                }
            }
            // Safety: rt.l_ptr == 0 means module_cache is empty — no leak possible.
        }
    }

    // Clear COROUTINE_TO_MAIN - prevents stale pointer crashes
    {
        let mut map = coroutines_to_main()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        map.clear();
    }
}

pub fn push_entry_module(l: *mut LuauState) -> Result<(), String> {
    with_runtime(l, |rt| {
        rt.module_stack.push(rt.entry_file.clone());
    })?;
    Ok(())
}

unsafe extern "C-unwind" fn nicy_require(l: *mut LuauState) -> c_int {
    let spec = match lua_string_at(l, 1) {
        Ok(s) => s,
        Err(e) => {
            return push_error(
                l,
                NicyError::require_error("unknown", e, RequireChain::new()),
            );
        }
    };

    let current_module = match with_runtime(l, |rt| rt.module_stack.last().cloned()) {
        Ok(Some(p)) => p,
        Ok(None) => {
            with_runtime(l, |rt| rt.entry_file.clone()).unwrap_or_else(|_| PathBuf::from("."))
        }

        Err(_e) => {
            // Fallback: usar o entry_file da thread principal
            let main_l = find_main_state(l);
            if let Some(main) = main_l {
                let all = runtimes().lock().unwrap_or_else(|e| e.into_inner());
                all.get(&main)
                    .map(|rt| rt.entry_file.clone())
                    .unwrap_or_else(|| PathBuf::from("."))
            } else {
                PathBuf::from(".")
            }
        }
    };

    // Track require chain
    let current_file = with_runtime(l, |rt| {
        rt.module_stack
            .last()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "main".to_string())
    })
    .unwrap_or_else(|_| "main".to_string());
    let (resolved, searched_paths) =
        match with_runtime(l, |rt| resolve_module_path(rt, &current_module, &spec)) {
            Ok(Ok((path, paths))) => (path, paths),
            Ok(Err(e)) => {
                with_runtime(l, |rt| {
                    rt.require_chain.pop();
                })
                .ok();
                return push_error(l, NicyError::require_error(&spec, e, RequireChain::new()));
            }
            Err(e) => {
                with_runtime(l, |rt| {
                    rt.require_chain.pop();
                })
                .ok();
                return push_error(l, NicyError::require_error(&spec, e, RequireChain::new()));
            }
        };

    with_runtime(l, |rt| {
        rt.require_chain
            .push(current_file, None, spec.clone(), searched_paths);
    })
    .ok();

    let current_fp = match FileFingerprint::from_path(&resolved) {
        Ok(fp) => fp,
        Err(e) => {
            with_runtime(l, |rt| {
                rt.require_chain.pop();
            })
            .ok();
            return push_error(
                l,
                NicyError::file_error(resolved.display().to_string(), "stat", e),
            );
        }
    };

    // Retry loop for concurrent requires:
    // 1. Check cache first (another coroutine may have finished loading)
    // 2. If not cached, try to acquire loading lock
    // 3. If lock is held, yield and retry
    let mut retries = 0;
    let max_retries = 50;

    loop {
        // Step 1: Re-check cache on each iteration
        let cached_ref = match with_runtime(l, |rt| {
            rt.module_cache.get(&resolved).and_then(|entry| {
                if entry.fp == current_fp {
                    Some(entry.value_ref)
                } else {
                    None
                }
            })
        }) {
            Ok(v) => v,
            Err(e) => {
                with_runtime(l, |rt| {
                    rt.require_chain.pop();
                })
                .ok();
                return push_error(l, NicyError::runtime_error(e));
            }
        };

        if let Some(r) = cached_ref {
            unsafe { compat::lua_rawgeti(l, lua::LUA_REGISTRYINDEX, r as lua::lua_Integer) };
            with_runtime(l, |rt| {
                rt.require_chain.pop();
            })
            .ok();
            return 1;
        }

        // Remove stale cache entry if it exists (fingerprint mismatch)
        let had_old_ref =
            match with_runtime(l, |rt| rt.module_cache.get(&resolved).map(|e| e.value_ref)) {
                Ok(v) => v,
                Err(e) => {
                    with_runtime(l, |rt| {
                        rt.require_chain.pop();
                    })
                    .ok();
                    return push_error(l, NicyError::runtime_error(e));
                }
            };
        if let Some(r) = had_old_ref {
            unsafe { lauxlib::luaL_unref(l, lua::LUA_REGISTRYINDEX, r) };
            let _ = with_runtime(l, |rt| {
                rt.module_cache.remove(&resolved);
                rt.module_jit.remove(&resolved);
            });
        }

        // Step 2: Try to acquire loading lock
        let is_loading = match with_runtime(l, |rt| {
            if rt.loading.contains(&resolved) {
                true
            } else {
                rt.loading.insert(resolved.clone());
                false
            }
        }) {
            Ok(v) => v,
            Err(e) => {
                with_runtime(l, |rt| {
                    rt.require_chain.pop();
                })
                .ok();
                return push_error(l, NicyError::runtime_error(e));
            }
        };

        if !is_loading {
            break; // Successfully acquired loading lock, proceed with loading
        }

        // Step 3: Another coroutine is loading - check for cycle before yielding
        retries += 1;

        // CYCLE DETECTION: Check if we're trying to require a module that's in our own stack
        let has_cycle = with_runtime(l, |rt| rt.module_stack.contains(&resolved)).unwrap_or(false);

        if has_cycle {
            // Real cycle detected - this module is trying to require itself (directly or indirectly)
            // Remove ourselves from loading and return error immediately
            let _ = with_runtime(l, |rt| {
                rt.loading.remove(&resolved);
            });
            with_runtime(l, |rt| {
                rt.require_chain.pop();
            })
            .ok();
            return push_error(
                l,
                NicyError::require_error(
                    &spec,
                    format!(
                        "cyclic require detected: '{}' is already being loaded",
                        spec
                    ),
                    RequireChain::new(),
                ),
            );
        }

        if retries >= max_retries {
            with_runtime(l, |rt| {
                rt.require_chain.pop();
            })
            .ok();
            return push_error(
                l,
                NicyError::require_error(
                    &spec,
                    format!(
                        "module is currently being loaded, timed out waiting for '{}'",
                        resolved.display()
                    ),
                    RequireChain::new(),
                ),
            );
        }

        task_scheduler::yield_for_scheduler(l);

        // After yielding, check if runtime is still valid
        // This can happen if the module we were waiting for crashed or had an error
        let runtime_check = with_runtime(l, |_| ());
        if runtime_check.is_err() {
            // Runtime context lost (cycle detected or other error), return nil+error
            with_runtime(l, |rt| {
                rt.require_chain.pop();
            })
            .ok();
            return push_error(
                l,
                NicyError::require_error(
                    &spec,
                    format!(
                        "module loading failed for '{}' (possible cycle or crash)",
                        spec
                    ),
                    RequireChain::new(),
                ),
            );
        }
    }

    let is_bytecode = resolved.extension().is_some_and(|e| e == "luauc");

    let (code_bytes, code_is_bytecode, module_native_requested) = if is_bytecode {
        match fs::read(&resolved) {
            Ok(c) => (c, true, false),
            Err(e) => {
                let _ = with_runtime(l, |rt| {
                    rt.loading.remove(&resolved);
                });
                with_runtime(l, |rt| {
                    rt.require_chain.pop();
                })
                .ok();
                return push_error(
                    l,
                    NicyError::file_error(resolved.display().to_string(), "read", e.to_string()),
                );
            }
        }
    } else {
        let source = match fs::read_to_string(&resolved) {
            Ok(c) => c,
            Err(e) => {
                let _ = with_runtime(l, |rt| {
                    rt.loading.remove(&resolved);
                });
                with_runtime(l, |rt| {
                    rt.require_chain.pop();
                })
                .ok();
                return push_error(
                    l,
                    NicyError::file_error(resolved.display().to_string(), "read", e.to_string()),
                );
            }
        };
        let (native, code) = strip_native_directive(&source);
        (code.into_bytes(), false, native)
    };
    let module_jit_enabled = module_native_requested && ensure_codegen_context(l);

    let mut chunkname = resolved.to_string_lossy().replace('\0', "?").into_bytes();
    chunkname.push(0);

    let load_status = if code_is_bytecode {
        unsafe {
            mlua_sys::luau::luau_load(
                l,
                chunkname.as_ptr() as *const c_char,
                code_bytes.as_ptr() as *const c_char,
                code_bytes.len(),
                0,
            )
        }
    } else {
        unsafe {
            compat::luaL_loadbuffer(
                l,
                code_bytes.as_ptr() as *const c_char,
                code_bytes.len(),
                chunkname.as_ptr() as *const c_char,
            )
        }
    };
    if load_status != 0 {
        let err = unsafe { lua::lua_tostring(l, -1) };
        let msg = if err.is_null() {
            "unknown load error".to_string()
        } else {
            unsafe { CStr::from_ptr(err) }.to_string_lossy().to_string()
        };
        // Capture stack trace for diagnostics (consumed by push_error below)
        unsafe {
            compat::luaL_traceback(l, l, std::ptr::null(), 0);
            lua::lua_pop(l, 1);
        }
        unsafe { lua::lua_settop(l, -2) };
        let _ = with_runtime(l, |rt| {
            rt.loading.remove(&resolved);
            // NOTE: module_stack.pop() intentionally omitted here.
            // The push for this module happens later (after successful load),
            // so popping here would erroneously remove the caller's entry.
        });
        with_runtime(l, |rt| {
            rt.require_chain.pop();
        })
        .ok();
        return push_error(
            l,
            NicyError::load_error(resolved.display().to_string(), msg),
        );
    }

    if code_is_bytecode {
        ensure_codegen_context(l);
        compile_loaded_chunk(l);
    } else if module_jit_enabled {
        compile_loaded_chunk(l);
    }

    let func_ref = unsafe { lauxlib::luaL_ref(l, lua::LUA_REGISTRYINDEX) };

    let _ = with_runtime(l, |rt| {
        rt.module_stack.push(resolved.clone());
    });

    let th = unsafe { lua::lua_newthread(l) };
    // Registrar a thread do módulo para que o require funcione dentro dela
    register_coroutine(l, th);

    unsafe { compat::lua_rawgeti(l, lua::LUA_REGISTRYINDEX, func_ref as lua::lua_Integer) };
    unsafe { lua::lua_xmove(l, th, 1) };

    let mut nres: c_int = 0;
    let mut call_status = unsafe { compat::lua_resume(th, l, 0, &mut nres as *mut c_int) };

    while call_status == lua::LUA_YIELD {
        task_scheduler::run_one_iteration(l);
        call_status = unsafe { compat::lua_resume(th, l, 0, &mut nres as *mut c_int) };
    }

    // CRITICAL: Always cleanup loading and module_stack, whether success or failure
    let _ = with_runtime(l, |rt| {
        rt.loading.remove(&resolved);
        rt.module_stack.pop();
    });

    if call_status != 0 {
        let err = unsafe { lua::lua_tostring(th, -1) };
        let msg = if err.is_null() {
            "unknown runtime error".to_string()
        } else {
            unsafe { CStr::from_ptr(err) }.to_string_lossy().to_string()
        };

        // Capture stack trace for diagnostics
        unsafe {
            compat::luaL_traceback(th, th, std::ptr::null(), 0);
            lua::lua_pop(th, 1);
        }

        unsafe { lua::lua_settop(l, -2) };
        with_runtime(l, |rt| {
            rt.require_chain.pop();
        })
        .ok();
        return push_error(l, NicyError::runtime_error(msg));
    }

    unsafe { lua::lua_xmove(th, l, 1) };

    if unsafe { lua::lua_type(l, -1) } == lua::LUA_TNIL {
        unsafe { lua::lua_settop(l, -2) };
        unsafe { lua::lua_pushboolean(l, 1) };
    }

    let value_ref = unsafe { lauxlib::luaL_ref(l, lua::LUA_REGISTRYINDEX) };

    // CRITICAL: Insert into cache BEFORE removing from loading to prevent race condition.
    // If we remove from loading first, another spawn could see both cache=empty and
    // loading=empty, and start loading the module again, causing duplicate execution
    // and potential registry reference collisions.
    let cache_insert = with_runtime(l, |rt| {
        rt.module_cache.insert(
            resolved.clone(),
            ModuleCacheEntry {
                fp: current_fp,
                value_ref,
            },
        );
        rt.module_jit.insert(resolved.clone(), module_jit_enabled);
        // loading and module_stack already cleaned up above
    });
    if let Err(e) = cache_insert {
        unsafe { lauxlib::luaL_unref(l, lua::LUA_REGISTRYINDEX, value_ref) };
        with_runtime(l, |rt| {
            rt.require_chain.pop();
        })
        .ok();
        return push_error(l, NicyError::runtime_error(e));
    }

    with_runtime(l, |rt| {
        rt.require_chain.pop();
    })
    .ok();
    unsafe { compat::lua_rawgeti(l, lua::LUA_REGISTRYINDEX, value_ref as lua::lua_Integer) };
    1
}

pub fn install_require(l: *mut LuauState) {
    unsafe { lua::lua_pushcfunction(l, nicy_require) };
    unsafe { lua::lua_setglobal(l, c"require".as_ptr() as *const c_char) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aliases_valid_json() {
        let json = r#"{"aliases": {"@MyLib": "libs/mylib", "@utils": "libs/utils"}}"#;
        let aliases = parse_aliases_from_luaurc(json);
        assert_eq!(aliases.get("@MyLib").unwrap(), "libs/mylib");
        assert_eq!(aliases.get("@utils").unwrap(), "libs/utils");
    }

    #[test]
    fn test_parse_aliases_adds_at_prefix() {
        let json = r#"{"aliases": {"MyLib": "libs/mylib"}}"#;
        let aliases = parse_aliases_from_luaurc(json);
        assert!(aliases.contains_key("@MyLib"));
        assert_eq!(aliases.get("@MyLib").unwrap(), "libs/mylib");
    }

    #[test]
    fn test_parse_aliases_skips_empty_values() {
        let json = r#"{"aliases": {"@EmptyAlias": ""}}"#;
        let aliases = parse_aliases_from_luaurc(json);
        assert!(!aliases.contains_key("@EmptyAlias"));
    }

    #[test]
    fn test_parse_aliases_invalid_json() {
        let aliases = parse_aliases_from_luaurc("not json at all");
        assert!(aliases.is_empty());
    }

    #[test]
    fn test_parse_aliases_jsonc_with_comments() {
        let jsonc = r#"{
    // This is a comment
    "aliases": {
        # Another comment style
        "@Lib": "libs/lib"
    }
}"#;
        let aliases = parse_aliases_from_luaurc(jsonc);
        assert_eq!(aliases.get("@Lib").unwrap(), "libs/lib");
    }

    #[test]
    fn test_parse_aliases_missing_aliases_key() {
        let json = r#"{"some_other_key": "value"}"#;
        let aliases = parse_aliases_from_luaurc(json);
        assert!(aliases.is_empty());
    }
}
