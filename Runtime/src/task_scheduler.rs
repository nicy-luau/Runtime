/*
Copyright (C) 2026 Yanlvl99 | Nicy Luau Runtime Development

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

#![allow(unreachable_code)]

use crate::error::{ErrorReporter, NicyError};
use crate::panic_payload_to_string;
use mlua_sys::luau::compat;
use mlua_sys::luau::lauxlib;
use mlua_sys::luau::lua;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::os::raw::{c_char, c_int};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering, compiler_fence};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

type LuauState = lua::lua_State;

#[derive(Clone, Copy, Eq, PartialEq)]
enum TaskKey {
    ThreadRef(c_int),
    DelayId(u64),
}

#[derive(Eq, PartialEq)]
struct ScheduledTask {
    due: Instant,
    key: TaskKey,
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.due.cmp(&self.due)
    }
}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

struct WaitInfo {
    start: Instant,
}

struct Scheduler {
    next_delay_id: u64,
    ready: VecDeque<c_int>,
    yielded: VecDeque<c_int>,
    timers: BinaryHeap<ScheduledTask>,
    waits: HashMap<c_int, WaitInfo>,
    delay_threads: HashMap<u64, c_int>,
    thread_refs: HashMap<usize, c_int>,
    canceled: HashMap<u64, ()>,
    thread_init_nargs: HashMap<c_int, c_int>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            next_delay_id: 1,
            ready: VecDeque::new(),
            yielded: VecDeque::new(),
            timers: BinaryHeap::new(),
            waits: HashMap::new(),
            delay_threads: HashMap::new(),
            thread_refs: HashMap::new(),
            canceled: HashMap::new(),
            thread_init_nargs: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    fn has_work(&self) -> bool {
        !self.ready.is_empty() || !self.yielded.is_empty() || !self.timers.is_empty()
    }

    fn pop_due(&mut self, now: Instant) -> Vec<c_int> {
        let mut out = Vec::new();
        while let Some(task) = self.timers.peek() {
            if task.due > now {
                break;
            }
            let task = self.timers.pop().unwrap();
            match task.key {
                TaskKey::ThreadRef(r) => {
                    if self.canceled.remove(&(r as u64)).is_some() {
                        self.waits.remove(&r);
                        let ptr = self
                            .thread_refs
                            .iter()
                            .find(|(_, v)| **v == r)
                            .map(|(k, _)| *k);
                        if let Some(p) = ptr {
                            self.thread_refs.remove(&p);
                        }
                        unsafe { lauxlib_unref_current_state(r) };
                    } else if self.waits.contains_key(&r) {
                        out.push(r);
                    }
                }
                TaskKey::DelayId(id) => {
                    if self.canceled.remove(&id).is_some() {
                        if let Some(r) = self.delay_threads.remove(&id) {
                            self.waits.remove(&r);
                            let ptr = self
                                .thread_refs
                                .iter()
                                .find(|(_, v)| **v == r)
                                .map(|(k, _)| *k);
                            if let Some(p) = ptr {
                                self.thread_refs.remove(&p);
                            }
                            unsafe { lauxlib_unref_current_state(r) };
                        }
                    } else if let Some(r) = self.delay_threads.remove(&id) {
                        out.push(r);
                    }
                }
            }
        }
        out
    }
}

static SCHED: OnceLock<Mutex<Scheduler>> = OnceLock::new();
static CURRENT_L: AtomicUsize = AtomicUsize::new(0);
/// FIX (UB): Track whether the current Lua state is valid for unref operations.
/// Replaces the lua_gettop() guard which was itself UB on closed states.
static CURRENT_L_VALID: AtomicBool = AtomicBool::new(false);

fn scheduler() -> &'static Mutex<Scheduler> {
    SCHED.get_or_init(|| Mutex::new(Scheduler::new()))
}

/// Mark the current Lua state as valid for unref operations.
/// Call this right after creating a new Lua state, before any scheduler work.
pub fn mark_current_state_valid(l: *mut LuauState) {
    CURRENT_L.store(l as usize, Ordering::SeqCst);
    CURRENT_L_VALID.store(true, Ordering::SeqCst);
}

/// Mark the current Lua state as invalid (e.g., before closing it).
pub fn mark_current_state_invalid() {
    CURRENT_L_VALID.store(false, Ordering::SeqCst);
    CURRENT_L.store(0, Ordering::SeqCst);
}

/// CRITICAL FIX (C-1): Clear all scheduler static state AND unref all thread registry refs.
/// Prevents memory leaks and stale pointers between nicy_start calls.
pub fn shutdown_scheduler(l: *mut LuauState) {
    // Clear SCHED - reset to fresh state, unrefing all thread refs first
    if let Some(sched) = SCHED.get() {
        let mut s = sched.lock().unwrap_or_else(|e| e.into_inner());
        // Unref all thread registry refs before clearing
        if !l.is_null() {
            for &thread_ref in s.thread_refs.values() {
                unsafe { lauxlib::luaL_unref(l, lua::LUA_REGISTRYINDEX, thread_ref) };
            }
        }
        s.ready.clear();
        s.yielded.clear();
        s.thread_refs.clear();
        s.waits.clear();
        s.canceled.clear();
        s.timers.clear();
        s.thread_init_nargs.clear();
        s.delay_threads.clear();
        s.next_delay_id = 1;
        drop(s);
    }
    // Reset CURRENT_L to null with SeqCst to prevent data races
    CURRENT_L_VALID.store(false, Ordering::SeqCst);
    CURRENT_L.store(0, Ordering::SeqCst);
}

/// FIX (UB): Unref a Lua state registry reference with safety guards.
/// Uses CURRENT_L_VALID boolean flag instead of lua_gettop() which was itself UB
/// on closed/freed states. The caller must ensure the state is still open.
///
/// # Safety
/// This function is inherently unsafe as it deals with raw Lua state pointers.
unsafe fn lauxlib_unref_current_state(r: c_int) {
    // Check validity flag — avoids any FFI call on a potentially closed state
    if !CURRENT_L_VALID.load(Ordering::SeqCst) {
        return;
    }
    let l = CURRENT_L.load(Ordering::SeqCst) as *mut LuauState;
    if l.is_null() {
        return;
    }
    unsafe { lauxlib::luaL_unref(l, lua::LUA_REGISTRYINDEX, r) };
}

fn duration_from_seconds(secs: f64) -> Duration {
    if !secs.is_finite() || secs <= 0.0 {
        return Duration::ZERO;
    }
    // Arredondar para 1ms para evitar overhead de timers muito curtos
    let ms = (secs * 1000.0).round() as u64;
    let capped_ms = ms.clamp(1, 60_000 * 60 * 24 * 365 * 10);
    Duration::from_millis(capped_ms)
}

unsafe fn raise_panic_as_lua_error(l: *mut LuauState, msg: &str) -> c_int {
    unsafe { compat::lua_pushlstring(l, msg.as_ptr() as *const c_char, msg.len()) };
    unsafe { lua::lua_error(l) }
}

unsafe fn with_lua_panic_guard(l: *mut LuauState, f: impl FnOnce() -> c_int) -> c_int {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => v,
        Err(p) => {
            let msg = format!("task panic: {}", panic_payload_to_string(p));
            let nicy_err = NicyError::PanicError {
                context: "task_scheduler",
                payload: msg.clone(),
            };
            ErrorReporter::fatal(&nicy_err);
            unsafe { raise_panic_as_lua_error(l, &msg) }
        }
    }
}

unsafe extern "C-unwind" fn task_spawn(l: *mut LuauState) -> c_int {
    unsafe {
        with_lua_panic_guard(l, || {
            lauxlib::luaL_checktype(l, 1, lua::LUA_TFUNCTION);
            let nargs = lua::lua_gettop(l);

            let th = lua::lua_newthread(l);
            lua::lua_pushvalue(l, 1);
            lua::lua_xmove(l, th, 1);
            for i in 2..=nargs {
                lua::lua_pushvalue(l, i);
                lua::lua_xmove(l, th, 1);
            }

            lua::lua_pushvalue(l, -1);
            let thread_ref = lauxlib::luaL_ref(l, lua::LUA_REGISTRYINDEX);

            // Registrar a thread no require_resolver para que require funcione dentro dela
            crate::require_resolver::register_coroutine(l, th);

            // CRITICAL FIX: Memory fence to ensure register_coroutine writes are visible
            // before the thread is added to the scheduler's ready queue.
            // Without this, the CPU may reorder the HashMap write to after the scheduler
            // lock acquisition, causing find_main_state to return None when the spawned
            // thread calls require.
            compiler_fence(Ordering::SeqCst);

            // Store the initial argument count for first resume
            // nargs includes the function, so actual arg count = nargs - 1
            let init_nargs = if nargs > 0 { nargs - 1 } else { 0 };

            let mut s = scheduler().lock().unwrap();
            s.ready.push_back(thread_ref);
            s.thread_refs.insert(th as usize, thread_ref);
            s.thread_init_nargs.insert(thread_ref, init_nargs);

            // Memory fence after scheduler state update to ensure visibility
            compiler_fence(Ordering::SeqCst);

            1
        })
    }
}

unsafe extern "C-unwind" fn task_defer(l: *mut LuauState) -> c_int {
    unsafe {
        with_lua_panic_guard(l, || {
            lauxlib::luaL_checktype(l, 1, lua::LUA_TFUNCTION);
            let nargs = lua::lua_gettop(l);

            let th = lua::lua_newthread(l);
            lua::lua_pushvalue(l, 1);
            lua::lua_xmove(l, th, 1);
            for i in 2..=nargs {
                lua::lua_pushvalue(l, i);
                lua::lua_xmove(l, th, 1);
            }

            lua::lua_pushvalue(l, -1);
            let thread_ref = lauxlib::luaL_ref(l, lua::LUA_REGISTRYINDEX);

            // Registrar a thread no require_resolver para que require funcione dentro dela
            crate::require_resolver::register_coroutine(l, th);

            // Store the initial argument count (function + extra args) for first resume
            // The function is at stack index 1 of the new thread, so nargs for resume = total - 1
            let init_nargs = if nargs > 0 { nargs - 1 } else { 0 };

            let mut s = scheduler().lock().unwrap();
            s.ready.push_back(thread_ref);
            s.thread_refs.insert(th as usize, thread_ref);
            s.thread_init_nargs.insert(thread_ref, init_nargs);

            1
        })
    }
}

unsafe extern "C-unwind" fn task_delay(l: *mut LuauState) -> c_int {
    unsafe {
        with_lua_panic_guard(l, || {
            let secs = lauxlib::luaL_checknumber(l, 1);
            lauxlib::luaL_checktype(l, 2, lua::LUA_TFUNCTION);
            let nargs = lua::lua_gettop(l);

            let th = lua::lua_newthread(l);
            lua::lua_pushvalue(l, 2);
            lua::lua_xmove(l, th, 1);
            for i in 3..=nargs {
                lua::lua_pushvalue(l, i);
                lua::lua_xmove(l, th, 1);
            }

            lua::lua_pushvalue(l, -1);
            let thread_ref = lauxlib::luaL_ref(l, lua::LUA_REGISTRYINDEX);

            let now = Instant::now();
            let dur = duration_from_seconds(secs as f64);
            let due = now + dur;

            // Store the initial argument count for first resume
            // nargs includes secs(1) + function(1) + extra args, so actual arg count = nargs - 2
            let init_nargs = if nargs > 2 { nargs - 2 } else { 0 };

            let mut s = scheduler().lock().unwrap();
            let id = s.next_delay_id;
            s.next_delay_id = s.next_delay_id.wrapping_add(1).max(1);
            s.delay_threads.insert(id, thread_ref);
            s.thread_refs.insert(th as usize, thread_ref);
            s.thread_init_nargs.insert(thread_ref, init_nargs);
            s.timers.push(ScheduledTask {
                due,
                key: TaskKey::DelayId(id),
            });

            lua::lua_pushnumber(l, id as f64);
            1
        })
    }
}

unsafe extern "C-unwind" fn task_wait(l: *mut LuauState) -> c_int {
    unsafe {
        with_lua_panic_guard(l, || {
            let secs = if lua::lua_gettop(l) >= 1 {
                lauxlib::luaL_checknumber(l, 1)
            } else {
                0.0
            };

            let is_main = lua::lua_pushthread(l) != 0;
            lua::lua_settop(l, -2);

            if is_main {
                if secs > 0.0 {
                    let dur = duration_from_seconds(secs);
                    let start = Instant::now();
                    let target = start + dur;

                    while Instant::now() < target {
                        run_one_iteration(l);
                        std::thread::yield_now();
                    }

                    lua::lua_pushnumber(l, Instant::now().duration_since(start).as_secs_f64());
                } else {
                    run_one_iteration(l);
                    lua::lua_pushnumber(l, 0.0);
                }
                return 1;
            }

            // Para threads não-principais: garantir que a thread esteja registrada no scheduler
            let tr = {
                let mut s = scheduler().lock().unwrap();
                match s.thread_refs.get(&(l as usize)).copied() {
                    Some(r) => r,
                    None => {
                        // Thread não registrada (ex: criada via coroutine.create sem task.spawn)
                        // Registrar agora para que o scheduler possa resumí-la
                        lua::lua_pushthread(l);
                        let thread_ref = lauxlib::luaL_ref(l, lua::LUA_REGISTRYINDEX);
                        s.thread_refs.insert(l as usize, thread_ref);
                        crate::require_resolver::register_coroutine(l, l);
                        thread_ref
                    }
                }
            };

            if secs <= 0.001 {
                let mut s = scheduler().lock().unwrap();
                s.yielded.push_back(tr);
                lua::lua_yield(l, 0)
            } else {
                let dur = duration_from_seconds(secs);
                let now = Instant::now();
                let due = now + dur;

                let mut s = scheduler().lock().unwrap();
                s.waits.insert(tr, WaitInfo { start: now });
                s.timers.push(ScheduledTask {
                    due,
                    key: TaskKey::ThreadRef(tr),
                });

                lua::lua_yield(l, 0)
            }
        })
    }
}

unsafe extern "C-unwind" fn task_cancel(l: *mut LuauState) -> c_int {
    unsafe {
        with_lua_panic_guard(l, || {
            if lua::lua_gettop(l) < 1 {
                lua::lua_pushboolean(l, 0);
                return 1;
            }

            let t = lua::lua_type(l, 1);
            if t == lua::LUA_TTHREAD {
                let th = lua::lua_tothread(l, 1);
                if th.is_null() {
                    lua::lua_pushboolean(l, 0);
                    return 1;
                }

                let mut s = scheduler().lock().unwrap();
                let thread_ref = s.thread_refs.get(&(th as usize)).copied();
                if let Some(r) = thread_ref {
                    // FIX (sync hang): Remove the thread from all scheduler queues
                    // so run_until_idle doesn't wait for its pending timers.
                    s.canceled.insert(r as u64, ());
                    s.ready.retain(|&t| t != r);
                    s.yielded.retain(|&t| t != r);
                    s.waits.remove(&r);
                    s.thread_refs.remove(&(th as usize));
                    // Also remove any pending timer for this thread
                    s.timers.retain(|t| match t.key {
                        TaskKey::ThreadRef(tr) => tr != r,
                        _ => true,
                    });
                    lua::lua_pushboolean(l, 1);
                } else {
                    lua::lua_pushboolean(l, 0);
                }
                return 1;
            }

            if t == lua::LUA_TNUMBER {
                let id_raw = mlua_sys::luau::lua::lua_tonumber(l, 1);
                if id_raw <= 0.0 {
                    lua::lua_pushboolean(l, 0);
                    return 1;
                }

                // H2 FIX: f64 loses precision above 2^53. Reject IDs that would
                // truncate to prevent incorrect cancellations.
                const MAX_SAFE_INTEGER: f64 = 9007199254740992.0; // 2^53
                if id_raw > MAX_SAFE_INTEGER {
                    let msg = c"task.cancel: delay id exceeds safe integer range (2^53). Use smaller IDs.";
                    lua::lua_pushnil(l);
                    compat::lua_pushlstring(l, msg.as_ptr() as *const c_char, msg.to_bytes().len());
                    return 2;
                }

                let id = id_raw as u64;
                let mut s = scheduler().lock().unwrap();
                // Check if this ID actually exists as a pending timer or delay
                let exists = s.timers.iter().any(|t| match &t.key {
                    TaskKey::DelayId(d) => *d == id,
                    _ => false,
                }) || s.delay_threads.contains_key(&id);

                if exists {
                    s.canceled.insert(id, ());
                    // FIX (sync hang): Remove timer entries so run_until_idle
                    // doesn't wait for cancelled delay timers
                    s.timers.retain(|t| match t.key {
                        TaskKey::DelayId(d) => d != id,
                        _ => true,
                    });
                    s.delay_threads.remove(&id);
                    lua::lua_pushboolean(l, 1);
                } else {
                    lua::lua_pushboolean(l, 0);
                }
                return 1;
            }

            lua::lua_pushboolean(l, 0);
            1
        })
    }
}

pub fn init(l: *mut LuauState) {
    if l.is_null() {
        return;
    }
    CURRENT_L.store(l as usize, Ordering::SeqCst);
    let _ = scheduler();

    unsafe { lua::lua_createtable(l, 0, 5) };

    unsafe { lua::lua_pushcfunction(l, task_spawn) };
    unsafe { lua::lua_setfield(l, -2, c"spawn".as_ptr() as *const _) };

    unsafe { lua::lua_pushcfunction(l, task_defer) };
    unsafe { lua::lua_setfield(l, -2, c"defer".as_ptr() as *const _) };

    unsafe { lua::lua_pushcfunction(l, task_delay) };
    unsafe { lua::lua_setfield(l, -2, c"delay".as_ptr() as *const _) };

    unsafe { lua::lua_pushcfunction(l, task_wait) };
    unsafe { lua::lua_setfield(l, -2, c"wait".as_ptr() as *const _) };

    unsafe { lua::lua_pushcfunction(l, task_cancel) };
    unsafe { lua::lua_setfield(l, -2, c"cancel".as_ptr() as *const _) };

    unsafe { lua::lua_setglobal(l, c"task".as_ptr() as *const _) };
}

unsafe fn resume_thread(l: *mut LuauState, thread_ref: c_int) -> Result<bool, ()> {
    unsafe { compat::lua_rawgeti(l, lua::LUA_REGISTRYINDEX, thread_ref as lua::lua_Integer) };
    let th = unsafe { lua::lua_tothread(l, -1) };
    if th.is_null() {
        unsafe { lua::lua_settop(l, -2) };
        unsafe { lauxlib::luaL_unref(l, lua::LUA_REGISTRYINDEX, thread_ref) };
        return Err(());
    }

    let elapsed = {
        let mut s = scheduler().lock().unwrap();
        s.waits
            .remove(&thread_ref)
            .map(|w| Instant::now().duration_since(w.start).as_secs_f64())
    };

    unsafe { lua::lua_settop(l, -2) };

    let canceled = {
        let mut s = scheduler().lock().unwrap();
        if s.canceled.remove(&(thread_ref as u64)).is_some() {
            s.thread_refs.retain(|_, v| *v != thread_ref);
            s.waits.remove(&thread_ref);
            true
        } else {
            false
        }
    };

    if canceled {
        unsafe { lauxlib::luaL_unref(l, lua::LUA_REGISTRYINDEX, thread_ref) };
        return Ok(true);
    }

    let nargs = if let Some(dt) = elapsed {
        // CRITICAL FIX: Push elapsed time to thread stack before resume
        unsafe { lua::lua_pushnumber(th, dt) };
        1
    } else {
        // First resume: use the stored initial argument count
        let mut s = scheduler().lock().unwrap();
        s.thread_init_nargs.remove(&thread_ref).unwrap_or(0)
    };

    // Memory fence before FFI call to ensure all scheduler state is visible
    compiler_fence(Ordering::SeqCst);

    let mut nres: c_int = 0;
    let st = unsafe { compat::lua_resume(th, l, nargs, &mut nres as *mut c_int) };
    // Memory fence after FFI call to ensure Lua state changes are visible
    compiler_fence(Ordering::SeqCst);

    let completed = st != lua::LUA_YIELD;

    if completed {
        // Memory fence before cleanup to ensure Lua state is stable
        compiler_fence(Ordering::SeqCst);
        let mut s = scheduler().lock().unwrap();
        s.thread_refs.retain(|_, v| *v != thread_ref);
        s.waits.remove(&thread_ref);
        s.thread_init_nargs.remove(&thread_ref);
        unsafe { lauxlib::luaL_unref(l, lua::LUA_REGISTRYINDEX, thread_ref) };
    }

    if st != 0 && st != lua::LUA_YIELD {
        let err = unsafe { lua::lua_tostring(th, -1) };
        if !err.is_null() {
            let msg = unsafe { std::ffi::CStr::from_ptr(err) }
                .to_string_lossy()
                .to_string();
            let nicy_err = NicyError::TaskError {
                task_type: "resume",
                message: msg,
            };
            ErrorReporter::report_with_state(Some(th), &nicy_err);
            // Clean up the error from stack
            unsafe { lua::lua_settop(th, 0) };
        }
    }

    Ok(completed)
}

pub fn run_until_idle(l: *mut LuauState) {
    loop {
        let mut ready_now = Vec::new();
        let mut next_due: Option<Instant> = None;

        {
            let mut s = scheduler().lock().unwrap();
            // Roblox API: spawn (ready) runs before defer (yielded)
            while let Some(r) = s.ready.pop_front() {
                ready_now.push(r);
            }
            while let Some(r) = s.yielded.pop_front() {
                ready_now.push(r);
            }

            let now = Instant::now();
            ready_now.extend(s.pop_due(now));

            // Check if there's any more work pending
            if ready_now.is_empty() {
                // If nothing is ready, check if there are pending timers
                if let Some(task) = s.timers.peek() {
                    next_due = Some(task.due);
                } else {
                    // No timers either — truly idle
                    break;
                }
            }
        }

        if next_due.is_some() && ready_now.is_empty() {
            // Something is pending in the future — sleep until then
            // This prevents busy-spinning while still being synchronous
            if let Some(when) = next_due {
                let now = Instant::now();
                if when > now {
                    std::thread::sleep(when - now);
                }
            }
            // Loop again — the timer should now be due
            continue;
        }

        for r in ready_now {
            let _result = unsafe { resume_thread(l, r) };
        }
    }
}

/// Runs one iteration of the scheduler, processing all ready tasks.
///
/// # Cancellation Safety
/// Canceled entries are **NOT** cleared in a blanket sweep here. Instead, they are
/// removed individually inside `pop_due` only when the corresponding timer/task is
/// actually due. This ensures that `task.cancel()` remains reliable — a cancel flag
/// will not be wiped before the scheduler has a chance to process it.
pub fn run_one_iteration(l: *mut LuauState) {
    let mut ready_now = Vec::new();

    {
        let mut s = scheduler().lock().unwrap();

        while let Some(r) = s.ready.pop_front() {
            ready_now.push(r);
        }
        while let Some(r) = s.yielded.pop_front() {
            ready_now.push(r);
        }

        let now = Instant::now();
        ready_now.extend(s.pop_due(now));

        // NOTE: Canceled entries are deliberately NOT cleared here. They are removed
        // individually in pop_due() when the corresponding timer is processed, ensuring
        // task.cancel() remains reliable and isn't wiped prematurely.
    }

    for r in ready_now {
        let result = unsafe { resume_thread(l, r) };
        if let Err(()) = result {
            crate::error::ErrorReporter::warn(&format!(
                "task scheduler: thread_ref {} resumed with error",
                r
            ));
        }
    }
}

/// Yields the current coroutine and adds it to the scheduler's yielded queue.
/// When resumed, the coroutine will continue from where it left off.
/// This is used for proper yielding in retry loops (e.g., concurrent require).
pub fn yield_for_scheduler(l: *mut LuauState) {
    unsafe {
        // Get or register the current thread reference
        let tr = {
            let mut s = scheduler().lock().unwrap();
            match s.thread_refs.get(&(l as usize)).copied() {
                Some(r) => r,
                None => {
                    // Thread not registered yet, register it now
                    lua::lua_pushthread(l);
                    let thread_ref = lauxlib::luaL_ref(l, lua::LUA_REGISTRYINDEX);
                    s.thread_refs.insert(l as usize, thread_ref);
                    // CRITICAL FIX: Find the actual main state, don't create circular mapping
                    // The main state is the one that has the RuntimeData
                    let main_l = crate::require_resolver::find_main_state(l).unwrap_or(l as usize);
                    let main_ptr = main_l as *mut crate::LuauState;
                    crate::require_resolver::register_coroutine(main_ptr, l);
                    thread_ref
                }
            }
        };

        // Add to yielded queue so scheduler will resume it later
        {
            let mut s = scheduler().lock().unwrap();
            s.yielded.push_back(tr);
        }

        // Yield the coroutine
        lua::lua_yield(l, 0);
    }
}

pub fn schedule_main_thread(l: *mut LuauState) {
    unsafe {
        let th = lua::lua_newthread(l);
        lua::lua_pushvalue(l, -2);
        lua::lua_xmove(l, th, 1);
        lua::lua_pushvalue(l, -1);
        let thread_ref = lauxlib::luaL_ref(l, lua::LUA_REGISTRYINDEX);

        // Registrar a thread principal no require_resolver
        crate::require_resolver::register_coroutine(l, th);

        let mut s = scheduler().lock().unwrap();
        s.ready.push_back(thread_ref);
        s.thread_refs.insert(th as usize, thread_ref);
        // Main thread has 0 extra args (function is already at stack index 1)
        s.thread_init_nargs.insert(thread_ref, 0);
    }
}

/// Get the registry reference for the current Lua state/thread.
/// Returns the thread_ref if registered, or None.
#[allow(dead_code)]
pub fn get_thread_ref_for_state(l: *mut LuauState) -> Option<c_int> {
    let s = scheduler().lock().unwrap();
    s.thread_refs.get(&(l as usize)).copied()
}

/// Add a thread to the yielded queue so it will be resumed on the next scheduler iteration.
#[allow(dead_code)]
pub fn resume_thread_by_ref(thread_ref: c_int) {
    let mut s = scheduler().lock().unwrap();
    s.yielded.push_back(thread_ref);
}

/// Register a thread reference in the scheduler's thread_refs map.
/// Used when a thread needs to be tracked for waiting/notification.
#[allow(dead_code)]
pub fn register_thread(l: *mut LuauState, thread_ref: c_int) {
    let mut s = scheduler().lock().unwrap();
    s.thread_refs.insert(l as usize, thread_ref);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_from_seconds_positive() {
        let d = duration_from_seconds(1.5);
        assert_eq!(d, Duration::from_millis(1500));
    }

    #[test]
    fn test_duration_from_seconds_zero() {
        let d = duration_from_seconds(0.0);
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn test_duration_from_seconds_negative() {
        let d = duration_from_seconds(-1.0);
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn test_duration_from_seconds_nan() {
        let d = duration_from_seconds(f64::NAN);
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn test_duration_from_seconds_infinity() {
        let d = duration_from_seconds(f64::INFINITY);
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn test_duration_from_seconds_minimum_1ms() {
        let d = duration_from_seconds(0.0001);
        assert!(d.as_millis() >= 1);
    }

    #[test]
    fn test_panic_payload_to_string() {
        assert_eq!(panic_payload_to_string(Box::new("task error")), "task error");
        assert_eq!(
            panic_payload_to_string(Box::new(String::from("owned"))),
            "owned"
        );
        assert_eq!(
            panic_payload_to_string(Box::new(123i32)),
            "non-string panic payload"
        );
    }
}
