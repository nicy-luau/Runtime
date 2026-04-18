# FFI C-ABI Reference

NicyRuntime exports **89 functions** with a stable `extern "C-unwind"` ABI. This includes **87 Lua C API wrappers** for complete Luau state management and **2 error code utilities**.

## Header File

[📥 Download NicyRuntime.h](https://raw.githubusercontent.com/nicy-luau/Runtime/main/Runtime/NicyRuntime.h)

Include in your C/C++ project:

```c
#include "NicyRuntime.h"
```

## Calling Convention

All functions use `extern "C-unwind"`, allowing exceptions to propagate across FFI boundaries. In C, these are mapped according to the platform's ABI (e.g., `__cdecl` on Windows).

## Stack Operations

| Function | Signature |
|----------|-----------|
| `nicy_lua_gettop` | `int nicy_lua_gettop(nicy_State *L);` |
| `nicy_lua_settop` | `void nicy_lua_settop(nicy_State *L, int idx);` |
| `nicy_lua_pushvalue` | `void nicy_lua_pushvalue(nicy_State *L, int idx);` |
| `nicy_lua_remove` | `void nicy_lua_remove(nicy_State *L, int idx);` |
| `nicy_lua_insert` | `void nicy_lua_insert(nicy_State *L, int idx);` |
| `nicy_lua_absindex` | `int nicy_lua_absindex(nicy_State *L, int idx);` |
| `nicy_lua_checkstack` | `int nicy_lua_checkstack(nicy_State *L, int extra);` |
| `nicy_lua_pop` | `void nicy_lua_pop(nicy_State *L, int n);` *(macro: `settop(L, -n-1)`)* |

## Push Operations

| Function | Signature |
|----------|-----------|
| `nicy_lua_pushnil` | `void nicy_lua_pushnil(nicy_State *L);` |
| `nicy_lua_pushboolean` | `void nicy_lua_pushboolean(nicy_State *L, int b);` |
| `nicy_lua_pushnumber` | `void nicy_lua_pushnumber(nicy_State *L, nicy_Number n);` |
| `nicy_lua_pushinteger` | `void nicy_lua_pushinteger(nicy_State *L, nicy_Integer n);` |
| `nicy_lua_pushstring` | `void nicy_lua_pushstring(nicy_State *L, const char *s);` |
| `nicy_lua_pushlstring` | `void nicy_lua_pushlstring(nicy_State *L, const char *s, size_t len);` |
| `nicy_lua_pushcfunction` | `void nicy_lua_pushcfunction(nicy_State *L, nicy_CFunction f);` |
| `nicy_lua_pushcclosure` | `void nicy_lua_pushcclosure(nicy_State *L, nicy_CFunction f, int n);` |
| `nicy_lua_pushlightuserdata` | `void nicy_lua_pushlightuserdata(nicy_State *L, void *p);` |
| `nicy_lua_newuserdata` | `void *nicy_lua_newuserdata(nicy_State *L, size_t sz);` |
| `nicy_lua_newbuffer` | `void *nicy_lua_newbuffer(nicy_State *L, size_t sz);` |
| `nicy_lua_newthread` | `nicy_State *nicy_lua_newthread(nicy_State *L);` |

## Type Checking

| Function | Signature |
|----------|-----------|
| `nicy_lua_type` | `int nicy_lua_type(nicy_State *L, int idx);` |
| `nicy_lua_typename` | `const char *nicy_lua_typename(nicy_State *L, int tp);` |
| `nicy_lua_isnil` | `int nicy_lua_isnil(nicy_State *L, int idx);` |
| `nicy_lua_isboolean` | `int nicy_lua_isboolean(nicy_State *L, int idx);` |
| `nicy_lua_isnumber` | `int nicy_lua_isnumber(nicy_State *L, int idx);` |
| `nicy_lua_isstring` | `int nicy_lua_isstring(nicy_State *L, int idx);` |
| `nicy_lua_istable` | `int nicy_lua_istable(nicy_State *L, int idx);` |
| `nicy_lua_isfunction` | `int nicy_lua_isfunction(nicy_State *L, int idx);` |
| `nicy_lua_isuserdata` | `int nicy_lua_isuserdata(nicy_State *L, int idx);` |
| `nicy_lua_isthread` | `int nicy_lua_isthread(nicy_State *L, int idx);` |
| `nicy_lua_iscfunction` | `int nicy_lua_iscfunction(nicy_State *L, int idx);` |
| `nicy_lua_isinteger` | `int nicy_lua_isinteger(nicy_State *L, int idx);` |
| `nicy_lua_isbuffer` | `int nicy_lua_isbuffer(nicy_State *L, int idx);` |

## Get & Conversion

| Function | Signature |
|----------|-----------|
| `nicy_lua_tostring` | `const char *nicy_lua_tostring(nicy_State *L, int idx);` |
| `nicy_lua_tolstring` | `const char *nicy_lua_tolstring(nicy_State *L, int idx, size_t *len);` |
| `nicy_lua_toboolean` | `int nicy_lua_toboolean(nicy_State *L, int idx);` |
| `nicy_lua_tonumber` | `nicy_Number nicy_lua_tonumber(nicy_State *L, int idx);` |
| `nicy_lua_tointeger` | `nicy_Integer nicy_lua_tointeger(nicy_State *L, int idx);` |
| `nicy_lua_touserdata` | `void *nicy_lua_touserdata(nicy_State *L, int idx);` |
| `nicy_lua_tobuffer` | `void *nicy_lua_tobuffer(nicy_State *L, int idx, size_t *len);` |

## Table Access

| Function | Signature |
|----------|-----------|
| `nicy_lua_getfield` | `void nicy_lua_getfield(nicy_State *L, int idx, const char *k);` |
| `nicy_lua_getglobal` | `void nicy_lua_getglobal(nicy_State *L, const char *k);` |
| `nicy_lua_setglobal` | `void nicy_lua_setglobal(nicy_State *L, const char *k);` |
| `nicy_lua_gettable` | `void nicy_lua_gettable(nicy_State *L, int idx);` |
| `nicy_lua_settable` | `void nicy_lua_settable(nicy_State *L, int idx);` |
| `nicy_lua_rawget` | `void nicy_lua_rawget(nicy_State *L, int idx);` |
| `nicy_lua_rawgeti` | `void nicy_lua_rawgeti(nicy_State *L, int idx, nicy_Integer n);` |
| `nicy_lua_rawset` | `void nicy_lua_rawset(nicy_State *L, int idx);` |
| `nicy_lua_rawseti` | `void nicy_lua_rawseti(nicy_State *L, int idx, nicy_Integer n);` |
| `nicy_lua_getmetatable` | `int nicy_lua_getmetatable(nicy_State *L, int idx);` |
| `nicy_lua_setmetatable` | `int nicy_lua_setmetatable(nicy_State *L, int idx);` |
| `nicy_lua_createtable` | `void nicy_lua_createtable(nicy_State *L, int narr, int nrec);` |
| `nicy_lua_next` | `int nicy_lua_next(nicy_State *L, int idx);` |

## Call & Execution

| Function | Signature |
|----------|-----------|
| `nicy_lua_call` | `void nicy_lua_call(nicy_State *L, int nargs, int nresults);` |
| `nicy_lua_pcall` | `int nicy_lua_pcall(nicy_State *L, int nargs, int nresults, int errfunc);` |
| `nicy_lua_error` | `int nicy_lua_error(nicy_State *L);` |
| `nicy_lua_resume` | `int nicy_lua_resume(nicy_State *L, nicy_State *from, int nargs, int *nres);` |
| `nicy_lua_yield` | `int nicy_lua_yield(nicy_State *L, int nresults);` |

## Comparison & Other

| Function | Signature |
|----------|-----------|
| `nicy_lua_equal` | `int nicy_lua_equal(nicy_State *L, int idx1, int idx2);` |
| `nicy_lua_lessthan` | `int nicy_lua_lessthan(nicy_State *L, int idx1, int idx2);` |
| `nicy_lua_rawequal` | `int nicy_lua_rawequal(nicy_State *L, int idx1, int idx2);` |
| `nicy_lua_concat` | `void nicy_lua_concat(nicy_State *L, int n);` |
| `nicy_lua_gc` | `int nicy_lua_gc(nicy_State *L, int what, int data);` |
| `nicy_lua_rawlen` | `size_t nicy_lua_rawlen(nicy_State *L, int idx);` |

## Lua 5.1 Compatibility

| Function | Signature |
|----------|-----------|
| `nicy_lua_getfenv` | `void nicy_lua_getfenv(nicy_State *L, int idx);` |
| `nicy_lua_setfenv` | `int nicy_lua_setfenv(nicy_State *L, int idx);` |

## Auxiliary Library (lauxlib)

| Function | Signature |
|----------|-----------|
| `nicy_luaL_checkstring` | `const char *nicy_luaL_checkstring(nicy_State *L, int narg);` |
| `nicy_luaL_checklstring` | `const char *nicy_luaL_checklstring(nicy_State *L, int narg, size_t *len);` |
| `nicy_luaL_checknumber` | `nicy_Number nicy_luaL_checknumber(nicy_State *L, int narg);` |
| `nicy_luaL_checkboolean` | `int nicy_luaL_checkboolean(nicy_State *L, int narg);` |
| `nicy_luaL_checkinteger` | `nicy_Integer nicy_luaL_checkinteger(nicy_State *L, int narg);` |
| `nicy_luaL_checktype` | `void nicy_luaL_checktype(nicy_State *L, int narg, int t);` |
| `nicy_luaL_checkany` | `void nicy_luaL_checkany(nicy_State *L, int narg);` |
| `nicy_luaL_checkbuffer` | `void *nicy_luaL_checkbuffer(nicy_State *L, int narg, size_t *len);` |
| `nicy_luaL_optstring` | `const char *nicy_luaL_optstring(nicy_State *L, int narg, const char *d);` |
| `nicy_luaL_optinteger` | `nicy_Integer nicy_luaL_optinteger(nicy_State *L, int narg, nicy_Integer d);` |
| `nicy_luaL_optnumber` | `nicy_Number nicy_luaL_optnumber(nicy_State *L, int narg, nicy_Number d);` |
| `nicy_luaL_argerror` | `int nicy_luaL_argerror(nicy_State *L, int narg, const char *extramsg);` |
| `nicy_luaL_where` | `void nicy_luaL_where(nicy_State *L, int lvl);` |
| `nicy_luaL_traceback` | `void nicy_luaL_traceback(nicy_State *L, nicy_State *L1, const char *msg, int level);` |
| `nicy_luaL_ref` | `int nicy_luaL_ref(nicy_State *L, int t);` |
| `nicy_luaL_unref` | `void nicy_luaL_unref(nicy_State *L, int t, int ref);` |
| `nicy_luaL_len` | `nicy_Integer nicy_luaL_len(nicy_State *L, int idx);` |
| `nicy_luaL_newmetatable` | `int nicy_luaL_newmetatable(nicy_State *L, const char *tname);` |
| `nicy_luaL_getmetatable` | `int nicy_luaL_getmetatable(nicy_State *L, const char *tname);` |
| `nicy_luaL_error` | `int nicy_luaL_error(nicy_State *L, const char *msg);` |

## Error Code Utilities

These functions help FFI integrators work with NicyRuntime's error codes:

| Function | Signature | Description |
|----------|-----------|-------------|
| `nicy_error_name` | `const char *nicy_error_name(int code);` | Convert error code to name string (e.g., 103 → `"NICY_ERR_CYCLIC_REQUIRE"`) |
| `nicy_is_nicy_error` | `int nicy_is_nicy_error(int code);` | Returns 1 if code is Nicy-specific (100+), 0 if standard Luau |

### Example (C)

```c
int err = nicy_start("script.luau");
if (err != 0) {
    const char* name = nicy_error_name(err);
    int is_nicy = nicy_is_nicy_error(err);
    
    if (is_nicy) {
        fprintf(stderr, "Nicy error: %s\n", name);
    } else {
        fprintf(stderr, "Luau error: %s\n", name);
    }
}
```

## Source Code (`NicyRuntime.h`)

Below is the complete, automatically synchronized source code of the `NicyRuntime.h` C header file.

```c
{{#include ../../../Runtime/NicyRuntime.h}}
```
