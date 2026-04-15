# Security Policy

## Supported Versions

Only the latest release is actively supported. Security fixes are always applied to `main` and shipped in the next release.

| Version       | Supported          |
|---------------|--------------------|
| 1.1.0 (latest)| :white_check_mark: |
| < 1.1.0       | :x:                |

## Scope

Nicy Runtime is a **system-level Luau runtime** designed to execute untrusted scripts. By its nature, it provides access to OS-level facilities — including filesystem operations, native library loading (`runtime.loadlib`), environment variable access, dynamic module resolution, and arbitrary code execution. It is not a sandbox.

**What this means:** scripts executed by this runtime have the same privileges as the process hosting the runtime. If you need to restrict what a script can do, you must do so at the OS level (e.g., containers, sandboxing, permissions, chroot, seccomp, etc.).

## What Is In Scope for Security Reports

We consider the following to be valid security concerns:

- **Undefined Behavior (UB)** in FFI boundaries — null pointer dereferences, use-after-free, double-frees, or memory corruption in `extern "C"` functions.
- **Stack corruption** — leaked Lua state references, unbalanced stacks after errors, or registry entries not properly unref'd that could lead to crashes across `nicy_start` / `nicy_eval` calls.
- **Panic safety** — panics in Rust code that escape `catch_unwind` and crash the host process without proper cleanup.
- **Native module loading vulnerabilities** — issues in `runtime.loadlib` or the require resolver that allow loading unintended libraries (e.g., path traversal, `\\?\` prefix bypasses, symlink following).
- **Bytecode validation** — malformed bytecode that causes the VM to crash, misbehave, or execute unintended code.
- **Task scheduler concurrency bugs** — data races between `task.spawn`, `task.cancel`, and `require` that could lead to memory corruption or deadlocks.
- **FFI export vulnerabilities** — exported functions that can be called with malicious pointer values to achieve arbitrary read/write.
- **Denial of service via crafted input** — scripts or bytecode files that cause infinite loops, stack exhaustion, or memory exhaustion in a way that cannot be interrupted.

## What Is Out of Scope

These are **expected behaviors** of a general-purpose runtime and will not be treated as vulnerabilities:

- A script using `runtime.loadlib` to execute native code — this is the intended purpose.
- A script reading/writing files on the filesystem — the runtime grants this access by design.
- A script consuming CPU or memory — scripts can do `while true do end`; the host process must manage resource limits.
- A script using `os.getenv` to read environment variables, `os.tmpname`/`os.remove`/`os.rename` for filesystem operations — these are standard Luau extensions provided by design.
- Exploits that require modifying the runtime library itself (DLL patching, memory editing of the running process).
- Issues that only manifest in `debug` builds (assertions, debug symbols, verbose error output).

## Reporting a Vulnerability

### Responsible Disclosure

If you discover a security issue, please report it **privately** so we can address it before it is publicly disclosed.

1. **Do NOT open a public issue.** Open a [GitHub Security Advisory](https://github.com/nicy-luau/Runtime/security/advisories/new) or email maintainers directly.
2. **Include a proof of concept** — a minimal script or set of steps that reproduces the issue.
3. **Describe the impact** — what can an attacker do? What is the worst-case scenario?
4. We will acknowledge receipt within **48 hours** and provide a timeline for a fix.
5. Once the fix is released, we will coordinate public disclosure and credit you (if you wish).

### Timeline

| Phase | Expected Time |
|-------|---------------|
| Acknowledgment | 48 hours |
| Initial assessment | 1 week |
| Fix development | 2–4 weeks |
| Release + public disclosure | Upon fix availability |

## Embargo and Disclosure

- Reporters are asked to keep any reported issues confidential until the fix is released.
- We will publish a security advisory alongside the release with a CVE (if applicable) and credit the reporter.
- We will NOT publish the reporter's personal information without explicit consent.

## Security Hardening Measures

The project implements several defensive measures:

- **`null_guard!` macro** on all 50+ FFI exports to prevent null pointer dereference.
- **`unsafe` annotations** on all FFI entry points with documented safety requirements.
- **`catch_unwind` guards** on all scheduler operations to prevent Rust panics from crashing the host.
- **State validity tracking** (`CURRENT_L_VALID`) to prevent UB from FFI calls on closed Lua states.
- **Integer range validation** in `task.cancel` to prevent precision-loss bugs with f64 IDs above 2^53.
- **Path canonicalization guards** to prevent loading of unintended native modules on Windows.
- **Explicit registry cleanup** before `lua_close` to prevent memory leaks across repeated runtime invocations.

These measures reduce risk but do not eliminate it. The fundamental security model is: **the runtime executes code with the privileges of the host process.**

## Responsible Use

This runtime is a tool. Like any tool, it can be used for constructive purposes or misused. The maintainers:

- Do not endorse the use of this runtime for malicious purposes.
- Do not provide support for exploiting vulnerabilities in third-party software.
- Will cooperate with responsible disclosure and will not assist in weaponizing security findings.
- Expect contributors and users to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Acknowledgments

We appreciate security researchers and users who report vulnerabilities responsibly. Thank you for helping keep this project safe.
