# Contributing to Nicy Runtime

Thank you for your interest in contributing to Nicy Runtime! This document covers everything you need to know to get started, from setting up the development environment to submitting your first pull request.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Features](#suggesting-features)
  - [Pull Requests](#pull-requests)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Cloning the Repository](#cloning-the-repository)
  - [Building](#building)
  - [Running Tests](#running-tests)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
  - [Branching](#branching)
  - [Commit Messages](#commit-messages)
  - [Opening a Pull Request](#opening-a-pull-request)
- [Coding Conventions](#coding-conventions)
  - [Rust Code](#rust-code)
  - [Luau Test Code](#luau-test-code)
  - [C FFI Code](#c-ffi-code)
- [Cross-Platform Builds](#cross-platform-builds)
- [FFI Header Regeneration](#ffi-header-regeneration)
- [Documentation](#documentation)

---

## Code of Conduct

- Be respectful and constructive in all interactions.
- Assume good faith — contributors are here to help.
- Keep discussions focused on the task at hand.

---

## How Can I Contribute?

### Reporting Bugs

Before opening a new issue, search the [existing issues](https://github.com/nicy-luau/Runtime/issues) to check if the bug has already been reported.

When reporting a bug, include:

- **Runtime version** (`nicy version` or `nicy runtime-version`)
- **Platform and architecture** (e.g., Windows x64, Linux ARM64, Android/Termux)
- **Steps to reproduce** — a minimal script that triggers the issue is ideal
- **Expected behavior** vs **actual behavior**
- **Error output or crash log**, if applicable

### Suggesting Features

Feature suggestions are welcome! When proposing a feature:

- Explain the **use case** — what problem does it solve?
- Provide a **concrete example** of how the API would look (e.g., a Luau snippet)
- Note whether it aligns with Roblox's Luau API or extends it
- If possible, outline the scope of implementation effort

### Pull Requests

- Work on a **fork** of this repository, then submit a PR from your fork.
- Keep PRs **focused** — one feature or fix per PR.
- Make sure tests pass before submitting.
- If your PR changes the FFI API, update the header file (`Runtime/NicyRuntime.h`) if needed.

---

## Getting Started

### Prerequisites

- **Rust** (latest stable) — install via [rustup](https://rustup.rs/)
- **Git** — for cloning and version control

Optional:
- **Rust Analyzer** — IDE support for Rust development
- **Luau LSP** — editor support for `.luau` files (e.g., [Luau LSP](https://github.com/JohnnyMorganz/luau-lsp))

### Cloning the Repository

```bash
git clone https://github.com/nicy-luau/Runtime.git
cd Runtime
```

### Building

#### Option A: Using the build script

```powershell
./build.ps1
```

This builds the project in release mode and places the resulting `nicyruntime.dll` (or `.so`/`.dylib`) in the appropriate output directory.

#### Option B: Using Cargo directly

```bash
# Build everything in release mode
cargo build --release

# Build only the runtime library
cargo build --release -p nicyruntime

# Build only the CLI
cargo build --release -p nicy
```

The project uses `lto = true`, `codegen-units = 1`, and `opt-level = "z"` in release for minimal binary size. Debug builds are faster to compile but produce larger, slower binaries.

### Running Tests

```bash
# Run the full Luau test suite (requires a built binary)
./target/release/nicy.exe Runtime/tests/run_all.luau

# Run Rust unit tests
cargo test --release -p nicyruntime
```

All tests should pass before submitting a PR. The test suite covers core API, require resolution, task scheduling (including stress tests with 10k+ operations), error handling, GC behavior, and edge cases.

---

## Project Structure

```
C:\NicyRuntime\
├── Cargo.toml              # Workspace root (resolver = "2")
├── Nicy/                   # CLI host application
│   ├── Cargo.toml
│   └── src/main.rs         # CLI entry point (run, eval, compile, help)
├── Runtime/                # Runtime dynamic library (cdylib)
│   ├── Cargo.toml
│   ├── NicyRuntime.h       # C header for FFI consumers
│   ├── src/
│   │   ├── lib.rs           # Main library: nicy_start, nicy_eval, nicy_compile, FFI entry points
│   │   ├── error.rs         # Error types (NicyError), error reporting, stack trace formatting
│   │   ├── ffi_exports.rs   # C-ABI exports: Lua C API wrappers + null_guard! macro
│   │   ├── task_scheduler.rs# Async task system: spawn, defer, delay, wait, cancel
│   │   └── require_resolver.rs  # Module resolution: .luaurc aliases, native modules, bytecode
│   └── tests/              # Luau integration tests
│       ├── run_all.luau    # Test runner (all suites)
│       ├── helpers/        # Test helpers (expect library)
│       └── Task/           # Task scheduler integration tests
├── Docs/                   # mdBook documentation site
├── build.ps1               # Build script (Windows, multi-target)
├── README.md
├── LICENSE                 # Mozilla Public License 2.0
└── CONTRIBUTING.md         # This file
```

The workspace has two crates:
- **`nicy`** (binary) — CLI frontend that loads the runtime library dynamically.
- **`nicyruntime`** (cdylib) — The actual Luau engine. Contains all FFI exports, scheduler, error handling, and module resolution.

---

## Development Workflow

### Branching

- The `main` branch is the stable development branch.
- Create feature branches from `main` in your fork.
- Name branches descriptively (e.g., `fix/cancel-ub`, `feat/native-module-caching`).

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]
```

Types:
- `feat` — new functionality
- `fix` — bug fix
- `refactor` — code change that is neither a feature nor a fix
- `docs` — documentation changes
- `test` — test additions or modifications
- `chore` — maintenance, build, CI, version bumps
- `perf` — performance improvements

Scope should be one of: `Runtime`, `Nicy`, `Docs`, `CI`, or omitted if unclear.

Examples:
```
fix(Runtime): fix UB in task_cancel for IDs above 2^53
feat(Runtime): add JSONC fallback for .luaurc parsing
refactor(Nicy): replace process::exit with Result propagation
docs(readme): add FFI header reference table
chore: bump version to 1.1.0
```

### Opening a Pull Request

1. Push your branch to your fork.
2. Open a PR targeting `main` on the upstream repository.
3. Include:
   - A clear title following Conventional Commits format.
   - A description of **what changed** and **why**.
   - Any relevant test output or screenshots.
4. The PR will be reviewed for correctness, performance, and safety before merging.

---

## Coding Conventions

### Rust Code

- Edition: **Rust 2024** (workspace `resolver = "2"`)
- Use `rustfmt` for formatting — run `cargo fmt` before committing.
- Use `clippy` for linting — run `cargo clippy` and fix warnings.
- **Safety**: All `extern "C"` FFI functions should be `unsafe extern "C"`. Document pointer validity requirements with `/// # Safety` comments.
- **Null guards**: Use the `null_guard!` macro in `ffi_exports.rs` at the top of every exported FFI function to prevent UB from null pointers.
- **C-string literals**: Use `c"string"` syntax instead of `b"string\0".as_ptr() as *const c_char`.
- **Let-chains**: Use `let`-chain syntax to collapse nested `if let` blocks.
- **No `unsafe` blocks** unless directly interacting with FFI. Keep `unsafe` scopes as narrow as possible.
- **Error handling**: Propagate errors via `Result` where possible. Use `NicyError` enum for typed error codes.
- **Logging**: Use `ErrorReporter::report_with_state()` or `ErrorReporter::warn()` instead of `println!`.
- **Avoid** adding `println!` or debug output to production code. Use `#[cfg(test)]` for test-specific code.

### Luau Test Code

- Tests live in `Runtime/tests/` and are written in `.luau` files.
- Use the `expect` helper from `tests/helpers/expect.luau`:
  - `expect.eq(a, b, "label")` — assert equality
  - `expect.neq(a, b, "label")` — assert inequality
  - `expect.truthy(value, "label")` — assert truthy
  - `expect.falsy(value, "label")` — assert falsy
  - `expect.type("string", value, "label")` — assert type
  - `expect.ok(fn, "label")` — assert function succeeds (wraps in `pcall`)
  - `expect.fails(fn, "label")` — assert function errors
- Note: `expect.ok` expects a **function**, not a boolean. Use `expect.truthy` for boolean values.
- Test files return a table with a `name` field and an array of test objects, each with `name` and `run` fields.
- Use `require()` for test fixtures and helpers.

### C FFI Code

- All exported functions in `ffi_exports.rs` must:
  - Be declared `#[unsafe(no_mangle)] pub unsafe extern "C-unwind"`
  - Start with a `null_guard!(l)` or `null_guard!(l, default)` macro call
  - Return `c_int` (or appropriate C type) matching the header declaration in `NicyRuntime.h`
- The header file (`NicyRuntime.h`) is the contract for C/C++ consumers — keep it in sync with `ffi_exports.rs`.
- Use `c_int` instead of `i32` in FFI function signatures for consistency.
- Use `c_char` for string pointers, never raw `u8` or `i8`.

---

## Cross-Platform Builds

The project targets four platforms with different `mlua-sys` feature flags:

| Target | Features | Notes |
|--------|----------|-------|
| Windows x64 | `luau`, `luau-codegen`, `luau-vector4`, `vendored` | Full features |
| Linux x64 | `luau`, `luau-codegen`, `luau-vector4`, `vendored` | Full features |
| Linux x86 (32-bit) | `luau`, `luau-codegen`, `vendored` | No `vector4` (TValue size mismatch) |
| macOS (aarch64/x64) | `luau`, `luau-codegen`, `luau-vector4`, `vendored` | Full features |
| Android (aarch64) | `luau`, `vendored` | No `codegen`, no `vector4` (stability) |

To build for a specific target:

```bash
# Example: Linux x64
cargo build --release -p nicyruntime --target x86_64-unknown-linux-gnu

# Example: Android ARM64 (requires NDK)
cargo build --release -p nicyruntime --target aarch64-linux-android
```

The `build.ps1` script handles target setup automatically on Windows.

---

## FFI Header Regeneration

The C header file at `Runtime/NicyRuntime.h` must stay in sync with the FFI exports in `Runtime/src/ffi_exports.rs`. When adding or modifying FFI functions:

1. Update the function signature in `ffi_exports.rs`.
2. Add or update the corresponding declaration in `NicyRuntime.h`.
3. Use consistent naming: `nicy_lua_*` for Lua API, `nicy_luaL_*` for auxiliary API, `nicy_*` for custom functions.

---

## Documentation

Documentation lives in the `Docs/` directory as an mdBook site. To build locally:

```bash
cd Docs
mdbook build
```

When adding or modifying API references, update the corresponding pages under `Docs/src/` and regenerate the header file. Documentation PRs should reflect changes to the live docs site structure.

---

## License

By contributing to this project, you agree that your contributions will be licensed under the [Mozilla Public License 2.0](LICENSE).
