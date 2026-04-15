# FFI C-ABI Reference

NicyRuntime exports **88 functions** with a stable `extern "C-unwind"` ABI. This includes **5 core runtime functions** (`nicy_start`, `nicy_eval`, `nicy_compile`, `nicy_version`, `nicy_luau_version`) and **83 Lua C API wrappers** for complete Luau state management.

## Header File

Include in your C/C++ project:

```c
#include "NicyRuntime.h"
```

## Calling Convention

All functions use `extern "C-unwind"`, allowing exceptions to propagate across FFI boundaries.

## Stack Operations

| Function | Signature |
|----------|-----------|
| `nicy_lua_gettop` | `c_int(*l)` |
| `nicy_lua_settop` | `void(*l, idx)` |
| `nicy_lua_pushvalue` | `void(*l, idx)` |
| `nicy_lua_remove` | `void(*l, idx)` |
| `nicy_lua_insert` | `void(*l, idx)` |
| `nicy_lua_absindex` | `c_int(*l, idx)` |
| `nicy_lua_checkstack` | `c_int(*l, extra)` |
| `nicy_lua_pop` | `void(*l, n)` (macro: `settop(l, -n-1)`) |

## Push Operations

| Function | Signature |
|----------|-----------|
| `nicy_lua_pushnil` | `void(*l)` |
| `nicy_lua_pushboolean` | `void(*l, b: c_int)` |
| `nicy_lua_pushnumber` | `void(*l, n: lua_Number)` |
| `nicy_lua_pushinteger` | `void(*l, n: lua_Integer)` |
| `nicy_lua_pushstring` | `void(*l, s: *const c_char)` |
| `nicy_lua_pushlstring` | `void(*l, s: *const c_char, len: usize)` |
| `nicy_lua_pushcfunction` | `void(*l, f: lua_CFunction)` |
| `nicy_lua_pushcclosure` | `void(*l, f: lua_CFunction, n: c_int)` |
| `nicy_lua_pushlightuserdata` | `void(*l, p: *mut c_void)` |
| `nicy_lua_newuserdata` | `*mut c_void(*l, sz: usize)` |
| `nicy_lua_newthread` | `*mut LuauState(*l)` |

## Type Checking

| Function | Signature |
|----------|-----------|
| `nicy_lua_type` | `c_int(*l, idx)` |
| `nicy_lua_typename` | `*const c_char(*l, tp: c_int)` |
| `nicy_lua_isnil` | `c_int(*l, idx)` |
| `nicy_lua_isboolean` | `c_int(*l, idx)` |
| `nicy_lua_isnumber` | `c_int(*l, idx)` |
| `nicy_lua_isstring` | `c_int(*l, idx)` |
| `nicy_lua_istable` | `c_int(*l, idx)` |
| `nicy_lua_isfunction` | `c_int(*l, idx)` |
| `nicy_lua_isuserdata` | `c_int(*l, idx)` |
| `nicy_lua_isthread` | `c_int(*l, idx)` |
| `nicy_lua_iscfunction` | `c_int(*l, idx)` |
| `nicy_lua_isinteger` | `c_int(*l, idx)` |

## Get & Conversion

| Function | Signature |
|----------|-----------|
| `nicy_lua_tostring` | `*const c_char(*l, idx)` |
| `nicy_lua_tolstring` | `*const c_char(*l, idx, len: *mut usize)` |
| `nicy_lua_toboolean` | `c_int(*l, idx)` |
| `nicy_lua_tonumber` | `lua_Number(*l, idx)` |
| `nicy_lua_tointeger` | `lua_Integer(*l, idx)` |
| `nicy_lua_touserdata` | `*mut c_void(*l, idx)` |

## Table Access

| Function | Signature |
|----------|-----------|
| `nicy_lua_getfield` | `void(*l, idx, k: *const c_char)` |
| `nicy_lua_getglobal` | `void(*l, k: *const c_char)` |
| `nicy_lua_setglobal` | `void(*l, k: *const c_char)` |
| `nicy_lua_gettable` | `void(*l, idx)` |
| `nicy_lua_settable` | `void(*l, idx)` |
| `nicy_lua_rawget` | `void(*l, idx)` |
| `nicy_lua_rawgeti` | `void(*l, idx, n: lua_Integer)` |
| `nicy_lua_rawset` | `void(*l, idx)` |
| `nicy_lua_rawseti` | `void(*l, idx, n: lua_Integer)` |
| `nicy_lua_getmetatable` | `c_int(*l, idx)` |
| `nicy_lua_setmetatable` | `c_int(*l, idx)` |
| `nicy_lua_createtable` | `void(*l, narr: c_int, nrec: c_int)` |
| `nicy_lua_next` | `c_int(*l, idx)` |

## Call & Execution

| Function | Signature |
|----------|-----------|
| `nicy_lua_call` | `void(*l, nargs: c_int, nresults: c_int)` |
| `nicy_lua_pcall` | `c_int(*l, nargs, nresults, errfunc: c_int)` |
| `nicy_lua_error` | `c_int(*l)` |
| `nicy_lua_resume` | `c_int(*l, from: *mut LuauState, nargs, nres: *mut c_int)` |
| `nicy_lua_yield` | `c_int(*l, nresults: c_int)` |

## Comparison & Other

| Function | Signature |
|----------|-----------|
| `nicy_lua_equal` | `c_int(*l, idx1, idx2)` |
| `nicy_lua_lessthan` | `c_int(*l, idx1, idx2)` |
| `nicy_lua_rawequal` | `c_int(*l, idx1, idx2)` |
| `nicy_lua_concat` | `void(*l, n: c_int)` |
| `nicy_lua_gc` | `c_int(*l, what: c_int, data: c_int)` |
| `nicy_lua_rawlen` | `usize(*l, idx)` |

## Lua 5.1 Compatibility

| Function | Signature |
|----------|-----------|
| `nicy_lua_getfenv` | `void(*l, idx)` |
| `nicy_lua_setfenv` | `c_int(*l, idx)` |

## Auxiliary Library (lauxlib)

| Function | Signature |
|----------|-----------|
| `nicy_luaL_checkstring` | `*const c_char(*l, narg: c_int)` |
| `nicy_luaL_checklstring` | `*const c_char(*l, narg, len: *mut usize)` |
| `nicy_luaL_checknumber` | `lua_Number(*l, narg)` |
| `nicy_luaL_checkboolean` | `c_int(*l, narg)` |
| `nicy_luaL_checkinteger` | `lua_Integer(*l, narg)` |
| `nicy_luaL_checktype` | `void(*l, narg, t: c_int)` |
| `nicy_luaL_checkany` | `void(*l, narg)` |
| `nicy_luaL_optstring` | `*const c_char(*l, narg, d: *const c_char)` |
| `nicy_luaL_optinteger` | `lua_Integer(*l, narg, d: lua_Integer)` |
| `nicy_luaL_optnumber` | `lua_Number(*l, narg, d: lua_Number)` |
| `nicy_luaL_argerror` | `c_int(*l, narg, extramsg: *const c_char)` |
| `nicy_luaL_where` | `void(*l, lvl: c_int)` |
| `nicy_luaL_traceback` | `void(*l, l1: *mut LuauState, msg: *const c_char, level: c_int)` |
| `nicy_luaL_ref` | `c_int(*l, t: c_int)` |
| `nicy_luaL_unref` | `void(*l, t, r: c_int)` |
| `nicy_luaL_len` | `lua_Integer(*l, idx)` |
| `nicy_luaL_newmetatable` | `c_int(*l, tname: *const c_char)` |
| `nicy_luaL_getmetatable` | `c_int(*l, tname: *const c_char)` |
| `nicy_luaL_error` | `c_int(*l, msg: *const c_char)` |
