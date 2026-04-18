/*
 * NicyRuntime.h — C header for the Nicy Runtime Luau engine.
 *
 * Provides:
 *   1. Core runtime functions (start, eval, compile, version)
 *   2. Full Lua C API wrappers prefixed with nicy_lua_*
 *   3. Auxlib wrappers prefixed with nicy_luaL_*
 *   4. Type definitions, constants, and calling-convention macros
 *
 * Link against nicyruntime.dll / libnicyruntime.so / libnicyruntime.dylib
 *
 * Copyright (C) 2026 Yanlvl99 | Nicy Luau Runtime Development
 * Licensed under the Mozilla Public License 2.0.
 */

#ifndef NICY_RUNTIME_H
#define NICY_RUNTIME_H

#include <stddef.h>
#include <stdint.h>

/* ================================================================
 * Calling convention
 * ================================================================ */

#if defined(_WIN32) || defined(__WIN32__)
  #define NICY_CAPI __cdecl
#else
  #define NICY_CAPI
#endif

#if defined(__cplusplus)
  #define NICY_EXTERN_C extern "C"
#else
  #define NICY_EXTERN_C extern
#endif

/* ================================================================
 * Opaque types
 * ================================================================ */

typedef struct lua_State nicy_State;

/* ================================================================
 * Integer and Number types (match Luau internals)
 * ================================================================ */

typedef intptr_t nicy_Integer;   /* lua_Integer  — signed integer  */
typedef double   nicy_Number;    /* lua_Number   — floating point  */

/* C function pointer type */
typedef int (NICY_CAPI *nicy_CFunction)(nicy_State *L);

/* ================================================================
 * Lua type codes  (lua_type() return values)
 * ================================================================ */

#define NICY_LUA_TNONE        (-1)
#define NICY_LUA_TNIL          0
#define NICY_LUA_TBOOLEAN      1
#define NICY_LUA_TLIGHTUSERDATA 2
#define NICY_LUA_TNUMBER       3
#define NICY_LUA_TINTEGER      4
#define NICY_LUA_TVECTOR       5
#define NICY_LUA_TSTRING       6
#define NICY_LUA_TTABLE        7
#define NICY_LUA_TFUNCTION     8
#define NICY_LUA_TUSERDATA     9
#define NICY_LUA_TTHREAD       10
#define NICY_LUA_TBUFFER       11

/* ================================================================
 * Lua status / error codes
 * ================================================================ */

#define NICY_LUA_OK            0
#define NICY_LUA_YIELD         1
#define NICY_LUA_ERRRUN        2
#define NICY_LUA_ERRSYNTAX     3
#define NICY_LUA_ERRMEM        4
#define NICY_LUA_ERRERR        5
#define NICY_LUA_ERRFILE       6

/* ================================================================
 * GC options  (for nicy_lua_gc)
 * ================================================================ */

#define NICY_LUA_GCSTOP        0
#define NICY_LUA_GCRESTART     1
#define NICY_LUA_GCCOLLECT     2
#define NICY_LUA_GCCOUNT       3
#define NICY_LUA_GCCOUNTB      4
#define NICY_LUA_GCSTEP        5
#define NICY_LUA_GCSETPAUSE     6
#define NICY_LUA_GCSETSTEPMUL   7
#define NICY_LUA_GCISRUNNING    9

/* ================================================================
 * Registry pseudo-index
 * ================================================================ */

#define NICY_LUA_REGISTRYINDEX  (-10000)

/* Number of multiple results (used as nresults in pcall/call) */
#define NICY_LUA_MULTRET        (-1)

/* ================================================================
 * 1. Core Runtime API
 * ================================================================ */

/**
 * nicy_start — Load and execute a Luau script file.
 *
 * @param filepath  Absolute or relative path to a .luau, .lua, or .luauc file.
 *
 * Creates a new Lua state, loads standard libraries, registers the
 * `runtime` and `task` globals, resolves require(), and runs the
 * script to completion (including all scheduled tasks).
 *
 * Prints errors to stderr and exits on fatal failures.
 * Thread-safe — each call creates an independent Lua state.
 */
NICY_EXTERN_C void NICY_CAPI nicy_start(const char *filepath);

/**
 * nicy_eval — Evaluate a string of Luau source code.
 *
 * @param code  NUL-terminated Luau source code.
 *
 * Creates an isolated Lua state with standard libraries, compiles
 * and runs the code, then tears everything down.  Errors are
 * printed to stderr.
 *
 * Useful for quick one-liners or embedding small scripts.
 */
NICY_EXTERN_C void NICY_CAPI nicy_eval(const char *code);

/**
 * nicy_compile — Compile a Luau script to bytecode (.luauc).
 *
 * @param filepath  Path to a .luau or .lua source file.
 *
 * Reads the source, applies compiler directives (--!native, --!optimize,
 * etc.), and writes the bytecode to <basename>.luauc in the same
 * directory.
 */
NICY_EXTERN_C void NICY_CAPI nicy_compile(const char *filepath);

/**
 * nicy_version — Return a human-readable version string.
 *
 * @return  Static string such as "Nicy Runtime 1.0.0-alpha".
 *          Do not free.
 */
NICY_EXTERN_C const char * NICY_CAPI nicy_version(void);

/**
 * nicy_luau_version — Return the embedded Luau version string.
 *
 * @return  Static string such as "0.709".  Do not free.
 */
NICY_EXTERN_C const char * NICY_CAPI nicy_luau_version(void);

/* ================================================================
 * 2. Stack manipulation
 * ================================================================ */

NICY_EXTERN_C int  NICY_CAPI nicy_lua_gettop(nicy_State *L);
NICY_EXTERN_C void NICY_CAPI nicy_lua_settop(nicy_State *L, int idx);
NICY_EXTERN_C void NICY_CAPI nicy_lua_pushvalue(nicy_State *L, int idx);
NICY_EXTERN_C void NICY_CAPI nicy_lua_remove(nicy_State *L, int idx);
NICY_EXTERN_C void NICY_CAPI nicy_lua_insert(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_checkstack(nicy_State *L, int extra);

/* ================================================================
 * 3. Push operations
 * ================================================================ */

NICY_EXTERN_C void NICY_CAPI nicy_lua_pushnil(nicy_State *L);
NICY_EXTERN_C void NICY_CAPI nicy_lua_pushboolean(nicy_State *L, int b);
NICY_EXTERN_C void NICY_CAPI nicy_lua_pushnumber(nicy_State *L, nicy_Number n);
NICY_EXTERN_C void NICY_CAPI nicy_lua_pushinteger(nicy_State *L, nicy_Integer n);
NICY_EXTERN_C void NICY_CAPI nicy_lua_pushstring(nicy_State *L, const char *s);
NICY_EXTERN_C void NICY_CAPI nicy_lua_pushlstring(nicy_State *L, const char *s, size_t len);
NICY_EXTERN_C void NICY_CAPI nicy_lua_pushcfunction(nicy_State *L, nicy_CFunction f);
NICY_EXTERN_C void NICY_CAPI nicy_lua_pushcclosure(nicy_State *L, nicy_CFunction f, int n);
NICY_EXTERN_C void NICY_CAPI nicy_lua_pushlightuserdata(nicy_State *L, void *p);
NICY_EXTERN_C void NICY_CAPI nicy_lua_pushvector(nicy_State *L, float x, float y, float z, float w);
NICY_EXTERN_C void *NICY_CAPI nicy_lua_newuserdata(nicy_State *L, size_t sz);
NICY_EXTERN_C void *NICY_CAPI nicy_lua_newbuffer(nicy_State *L, size_t sz);
NICY_EXTERN_C nicy_State *NICY_CAPI nicy_lua_newthread(nicy_State *L);

/* ================================================================
 * 4. Query / check type
 * ================================================================ */

NICY_EXTERN_C int  NICY_CAPI nicy_lua_type(nicy_State *L, int idx);
NICY_EXTERN_C const char *NICY_CAPI nicy_lua_typename(nicy_State *L, int tp);

NICY_EXTERN_C int  NICY_CAPI nicy_lua_isnil(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_isboolean(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_isnumber(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_isstring(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_istable(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_isfunction(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_isuserdata(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_isthread(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_iscfunction(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_isinteger(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_isbuffer(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_isvector(nicy_State *L, int idx);

/* ================================================================
 * 5. Get / conversion operations
 * ================================================================ */

NICY_EXTERN_C const char *NICY_CAPI nicy_lua_tostring(nicy_State *L, int idx);
NICY_EXTERN_C const char *NICY_CAPI nicy_lua_tolstring(nicy_State *L, int idx, size_t *len);
NICY_EXTERN_C int         NICY_CAPI nicy_lua_toboolean(nicy_State *L, int idx);
NICY_EXTERN_C nicy_Number NICY_CAPI nicy_lua_tonumber(nicy_State *L, int idx);
NICY_EXTERN_C nicy_Integer NICY_CAPI nicy_lua_tointeger(nicy_State *L, int idx);
NICY_EXTERN_C void *      NICY_CAPI nicy_lua_touserdata(nicy_State *L, int idx);
NICY_EXTERN_C void *      NICY_CAPI nicy_lua_tobuffer(nicy_State *L, int idx, size_t *len);
NICY_EXTERN_C const float *NICY_CAPI nicy_lua_tovector(nicy_State *L, int idx);

/* ================================================================
 * 6. Table and global access — get
 * ================================================================ */

NICY_EXTERN_C void NICY_CAPI nicy_lua_getfield(nicy_State *L, int idx, const char *k);
NICY_EXTERN_C void NICY_CAPI nicy_lua_getglobal(nicy_State *L, const char *k);
NICY_EXTERN_C void NICY_CAPI nicy_lua_gettable(nicy_State *L, int idx);
NICY_EXTERN_C void NICY_CAPI nicy_lua_rawget(nicy_State *L, int idx);
NICY_EXTERN_C void NICY_CAPI nicy_lua_rawgeti(nicy_State *L, int idx, nicy_Integer n);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_getmetatable(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_next(nicy_State *L, int idx);

/* ================================================================
 * 7. Table and global access — set
 * ================================================================ */

NICY_EXTERN_C void NICY_CAPI nicy_lua_setfield(nicy_State *L, int idx, const char *k);
NICY_EXTERN_C void NICY_CAPI nicy_lua_setglobal(nicy_State *L, const char *k);
NICY_EXTERN_C void NICY_CAPI nicy_lua_settable(nicy_State *L, int idx);
NICY_EXTERN_C void NICY_CAPI nicy_lua_rawset(nicy_State *L, int idx);
NICY_EXTERN_C void NICY_CAPI nicy_lua_rawseti(nicy_State *L, int idx, nicy_Integer n);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_setmetatable(nicy_State *L, int idx);

/* ================================================================
 * 8. Table / array creation
 * ================================================================ */

NICY_EXTERN_C void NICY_CAPI nicy_lua_createtable(nicy_State *L, int narr, int nrec);

/* ================================================================
 * 9. Call and execution
 * ================================================================ */

NICY_EXTERN_C void NICY_CAPI nicy_lua_call(nicy_State *L, int nargs, int nresults);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_pcall(nicy_State *L, int nargs, int nresults, int errfunc);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_error(nicy_State *L);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_resume(nicy_State *L, nicy_State *from, int nargs, int *nres);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_yield(nicy_State *L, int nresults);

/* ================================================================
 * 10. Comparison
 * ================================================================ */

NICY_EXTERN_C int NICY_CAPI nicy_lua_equal(nicy_State *L, int idx1, int idx2);
NICY_EXTERN_C int NICY_CAPI nicy_lua_lessthan(nicy_State *L, int idx1, int idx2);
NICY_EXTERN_C int NICY_CAPI nicy_lua_rawequal(nicy_State *L, int idx1, int idx2);

/* ================================================================
 * 11. Other operations
 * ================================================================ */

NICY_EXTERN_C void  NICY_CAPI nicy_lua_concat(nicy_State *L, int n);
NICY_EXTERN_C int   NICY_CAPI nicy_lua_gc(nicy_State *L, int what, int data);
NICY_EXTERN_C size_t NICY_CAPI nicy_lua_rawlen(nicy_State *L, int idx);
NICY_EXTERN_C int   NICY_CAPI nicy_lua_absindex(nicy_State *L, int idx);

/* ================================================================
 * 12. Environment (Lua 5.1 compat)
 * ================================================================ */

NICY_EXTERN_C void NICY_CAPI nicy_lua_getfenv(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_lua_setfenv(nicy_State *L, int idx);

/* ================================================================
 * 13. lauxlib — check / opt helpers
 * ================================================================ */

NICY_EXTERN_C const char *NICY_CAPI nicy_luaL_checkstring(nicy_State *L, int narg);
NICY_EXTERN_C const char *NICY_CAPI nicy_luaL_checklstring(nicy_State *L, int narg, size_t *len);
NICY_EXTERN_C void *       NICY_CAPI nicy_luaL_checkbuffer(nicy_State *L, int narg, size_t *len);
NICY_EXTERN_C nicy_Number  NICY_CAPI nicy_luaL_checknumber(nicy_State *L, int narg);
NICY_EXTERN_C int          NICY_CAPI nicy_luaL_checkboolean(nicy_State *L, int narg);
NICY_EXTERN_C nicy_Integer NICY_CAPI nicy_luaL_checkinteger(nicy_State *L, int narg);
NICY_EXTERN_C void         NICY_CAPI nicy_luaL_checktype(nicy_State *L, int narg, int t);
NICY_EXTERN_C void         NICY_CAPI nicy_luaL_checkany(nicy_State *L, int narg);

NICY_EXTERN_C const char *NICY_CAPI nicy_luaL_optstring(nicy_State *L, int narg, const char *d);
NICY_EXTERN_C nicy_Integer NICY_CAPI nicy_luaL_optinteger(nicy_State *L, int narg, nicy_Integer d);
NICY_EXTERN_C nicy_Number  NICY_CAPI nicy_luaL_optnumber(nicy_State *L, int narg, nicy_Number d);

/* ================================================================
 * 14. lauxlib — errors, traceback, refs
 * ================================================================ */

NICY_EXTERN_C int  NICY_CAPI nicy_luaL_error(nicy_State *L, const char *msg);
NICY_EXTERN_C int  NICY_CAPI nicy_luaL_argerror(nicy_State *L, int narg, const char *extramsg);
NICY_EXTERN_C void NICY_CAPI nicy_luaL_where(nicy_State *L, int lvl);
NICY_EXTERN_C void NICY_CAPI nicy_luaL_traceback(nicy_State *L, nicy_State *L1, const char *msg, int level);

NICY_EXTERN_C int  NICY_CAPI nicy_luaL_ref(nicy_State *L, int t);
NICY_EXTERN_C void NICY_CAPI nicy_luaL_unref(nicy_State *L, int t, int ref);

/* ================================================================
 * 15. lauxlib — metatables and length
 * ================================================================ */

NICY_EXTERN_C nicy_Integer NICY_CAPI nicy_luaL_len(nicy_State *L, int idx);
NICY_EXTERN_C int  NICY_CAPI nicy_luaL_newmetatable(nicy_State *L, const char *tname);
NICY_EXTERN_C int  NICY_CAPI nicy_luaL_getmetatable(nicy_State *L, const char *tname);

/* ================================================================
 * 16. Convenience macros
 * ================================================================ */

/**
 * nicy_lua_isnoneornil — True if index is invalid or value is nil.
 */
#define nicy_lua_isnoneornil(L, n) \
    (nicy_lua_type(L, (n)) <= 0)

/**
 * nicy_lua_pop — Pop n values from the stack.
 */
#define nicy_lua_pop(L, n)  nicy_lua_settop((L), -(n) - 1)

/* ================================================================
 * 17. Error code helpers
 * ================================================================ */

/**
 * nicy_error_name — Convert an error code number to its human-readable name.
 *
 * @param code  Error code (e.g. 103 for NICY_ERR_CYCLIC_REQUIRE).
 * @return  Static string such as "NICY_ERR_CYCLIC_REQUIRE".  Do not free.
 */
NICY_EXTERN_C const char * NICY_CAPI nicy_error_name(int code);

/**
 * nicy_is_nicy_error — Check if an error code is Nicy-specific (100+ range).
 *
 * @return  1 if Nicy-specific, 0 if standard Luau code.
 */
NICY_EXTERN_C int NICY_CAPI nicy_is_nicy_error(int code);

/* ================================================================
 * Notes for embedders
 * ================================================================
 *
 * 1. Thread safety
 *    Each Lua state is NOT thread-safe.  One state must be accessed
 *    from a single thread at a time.  nicy_start(), nicy_eval(), and
 *    nicy_compile() each create and destroy their own state internally
 *    so they are safe to call from any thread.
 *
 * 2. Calling convention
 *    On Windows the functions use __cdecl.  On other platforms this
 *    macro expands to nothing (default ABI).
 *
 * 3. Linking
 *    - Windows: link against nicyruntime.lib (import lib) or
 *               LoadLibrary("nicyruntime.dll") + GetProcAddress.
 *    - macOS:   link against libnicyruntime.dylib  (-lnicyruntime)
 *    - Linux:   link against libnicyruntime.so     (-lnicyruntime)
 *
 * 4. Lifetime of returned strings
 *    nicy_version() and nicy_luau_version() return pointers to
 *    statically allocated, NUL-terminated strings.  Do NOT free them.
 *
 * 5. Compiler directives in source code
 *    When loading source files (not bytecode), the following first-line
 *    directives are recognised:
 *      --!native              Enable JIT/CodeGen for this file
 *      --!optimize 0|1|2      Set optimisation level (default 1)
 *    Directives are stripped before compilation.
 *
 * 6. Bytecode files
 *    Files with the .luauc extension are loaded as pre-compiled
 *    bytecode and executed directly.  CodeGen is applied automatically
 *    when supported by the platform.
 *
 */

#endif /* NICY_RUNTIME_H */
