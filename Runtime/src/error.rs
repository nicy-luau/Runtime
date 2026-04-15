/*
Copyright (C) 2026 Yanlvl99 | Nicy Luau Runtime Development

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! # Nicy Runtime Error System
//!
//! Concise error reporting by default with optional verbose PowerShell-style details.
//! - Default: Short, readable error messages
//! - Verbose: Full hierarchical output via NICY_VERBOSE_ERRORS=1
//! - Path cleanup: Removes \\?\ prefix automatically
//! - pcall isolation: Errors inside pcall are NOT printed
//! - Fatal require: require() without pcall terminates execution

use mlua_sys::luau::lua;
use std::cell::Cell;
use std::env;
use std::fmt;

// ============================================================================
// ANSI Colors for Windows Console
// ============================================================================

/// Check if ANSI colors are enabled (default true, can be disabled via NICY_NO_COLOR=1)
fn use_colors() -> bool {
    std::env::var("NICY_NO_COLOR")
        .map(|v| v != "1" && v != "true")
        .unwrap_or(true)
}

struct Colors;

impl Colors {
    fn red() -> &'static str {
        if use_colors() { "\x1b[31m" } else { "" }
    }
    fn yellow() -> &'static str {
        if use_colors() { "\x1b[33m" } else { "" }
    }
    fn cyan() -> &'static str {
        if use_colors() { "\x1b[36m" } else { "" }
    }
    #[allow(dead_code)]
    fn green() -> &'static str {
        if use_colors() { "\x1b[32m" } else { "" }
    }
    #[allow(dead_code)]
    fn bold() -> &'static str {
        if use_colors() { "\x1b[1m" } else { "" }
    }
    fn reset() -> &'static str {
        if use_colors() { "\x1b[0m" } else { "" }
    }
    fn dim() -> &'static str {
        if use_colors() { "\x1b[2m" } else { "" }
    }
}

// ============================================================================
// Nicy Runtime Error Codes
// ============================================================================

/// Nicy Runtime Error Codes
///
/// These extend Luau's standard error codes (LUA_OK, LUA_ERRRUN, etc.)
/// with runtime-specific codes for better error categorization.
///
/// Standard Luau codes:
/// - 0: LUA_OK (success)
/// - 1: LUA_YIELD (coroutine yielded)
/// - 2: LUA_ERRRUN (runtime error)
/// - 3: LUA_ERRSYNTAX (syntax error)
/// - 4: LUA_ERRMEM (memory error)
/// - 5: LUA_ERRERR (error handler error)
/// - 6: LUA_ERRFILE (file error)
///
/// Nicy-specific codes (100+ range):
/// - 100: NICY_ERR_MODULE_NOT_FOUND (require failed to resolve module)
/// - 101: NICY_ERR_MODULE_LOAD_FAILED (module found but failed to load/compile)
/// - 102: NICY_ERR_MODULE_INIT_FAILED (module loaded but init function failed)
/// - 103: NICY_ERR_CYCLIC_REQUIRE (cyclic dependency detected)
/// - 104: NICY_ERR_TASK_CRASH (task/coroutine crashed)
/// - 105: NICY_ERR_NATIVE_CRASH (native DLL crashed)
/// - 106: NICY_ERR_TIMEOUT (operation timed out)
/// - 107: NICY_ERR_PERMISSION_DENIED (access denied)
pub mod error_codes {
    // Standard Luau error codes
    pub const LUA_OK: i32 = 0;
    pub const LUA_YIELD: i32 = 1;
    pub const LUA_ERRRUN: i32 = 2;
    pub const LUA_ERRSYNTAX: i32 = 3;
    pub const LUA_ERRMEM: i32 = 4;
    pub const LUA_ERRERR: i32 = 5;
    pub const LUA_ERRFILE: i32 = 6;

    // Nicy-specific error codes
    pub const NICY_ERR_MODULE_NOT_FOUND: i32 = 100;
    pub const NICY_ERR_MODULE_LOAD_FAILED: i32 = 101;
    pub const NICY_ERR_MODULE_INIT_FAILED: i32 = 102;
    pub const NICY_ERR_CYCLIC_REQUIRE: i32 = 103;
    pub const NICY_ERR_TASK_CRASH: i32 = 104;
    pub const NICY_ERR_NATIVE_CRASH: i32 = 105;
    pub const NICY_ERR_TIMEOUT: i32 = 106;
    pub const NICY_ERR_PERMISSION_DENIED: i32 = 107;

    /// Convert error code to human-readable name
    pub fn code_to_name(code: i32) -> &'static str {
        match code {
            LUA_OK => "LUA_OK",
            LUA_YIELD => "LUA_YIELD",
            LUA_ERRRUN => "LUA_ERRRUN",
            LUA_ERRSYNTAX => "LUA_ERRSYNTAX",
            LUA_ERRMEM => "LUA_ERRMEM",
            LUA_ERRERR => "LUA_ERRERR",
            LUA_ERRFILE => "LUA_ERRFILE",
            NICY_ERR_MODULE_NOT_FOUND => "NICY_ERR_MODULE_NOT_FOUND",
            NICY_ERR_MODULE_LOAD_FAILED => "NICY_ERR_MODULE_LOAD_FAILED",
            NICY_ERR_MODULE_INIT_FAILED => "NICY_ERR_MODULE_INIT_FAILED",
            NICY_ERR_CYCLIC_REQUIRE => "NICY_ERR_CYCLIC_REQUIRE",
            NICY_ERR_TASK_CRASH => "NICY_ERR_TASK_CRASH",
            NICY_ERR_NATIVE_CRASH => "NICY_ERR_NATIVE_CRASH",
            NICY_ERR_TIMEOUT => "NICY_ERR_TIMEOUT",
            NICY_ERR_PERMISSION_DENIED => "NICY_ERR_PERMISSION_DENIED",
            _ => "UNKNOWN_ERROR_CODE",
        }
    }
}

// ============================================================================
// Error Levels
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorLevel {
    Error,
    Warning,
    Info,
}

impl ErrorLevel {
    pub fn label(&self) -> &'static str {
        match self {
            ErrorLevel::Error => "error",
            ErrorLevel::Warning => "warning",
            ErrorLevel::Info => "info",
        }
    }
}

// ============================================================================
// Require Chain Tracking
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct RequireChainFrame {
    pub file: String,
    pub line: Option<u32>,
    pub spec: String,
    pub searched_paths: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RequireChain {
    frames: Vec<RequireChainFrame>,
}

impl RequireChain {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn push(
        &mut self,
        file: String,
        line: Option<u32>,
        spec: String,
        searched_paths: Vec<String>,
    ) {
        self.frames.push(RequireChainFrame {
            file,
            line,
            spec,
            searched_paths,
        });
    }

    pub fn pop(&mut self) {
        self.frames.pop();
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn format_chain(&self) -> String {
        if self.frames.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        for frame in self.frames.iter().rev() {
            let line_info = frame.line.map(|l| format!(":{}", l)).unwrap_or_default();
            result.push_str(&format!(
                "    at require('{}'), {}{}\n",
                frame.spec, frame.file, line_info
            ));

            if !frame.searched_paths.is_empty() {
                result.push_str("    SearchedPaths         :\n");
                for path in &frame.searched_paths {
                    result.push_str(&format!("        {}\n", path));
                }
            }
        }

        result
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone)]
pub enum NicyError {
    LoadError {
        path: String,
        line: Option<u32>,
        column: Option<u32>,
        message: String,
    },
    RequireError {
        spec: String,
        resolved_path: Option<String>,
        chain: RequireChain,
        message: String,
    },
    RuntimeError {
        message: String,
        stack_trace: Option<String>,
        file: Option<String>,
        line: Option<u32>,
    },
    TaskError {
        task_type: &'static str,
        message: String,
    },
    PanicError {
        context: &'static str,
        payload: String,
    },
    FileError {
        path: String,
        operation: &'static str,
        message: String,
    },
    RuntimeErrorGeneric {
        context: &'static str,
        message: String,
    },
}

impl fmt::Display for NicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NicyError::LoadError {
                path,
                line,
                column,
                message,
            } => {
                let loc = match (line, column) {
                    (Some(l), Some(c)) => format!("{}:{}:{}", path, l, c),
                    (Some(l), None) => format!("{}:{}", path, l),
                    _ => path.clone(),
                };
                write!(f, "Failed to load '{}': {}", loc, message)
            }
            NicyError::RequireError { spec, message, .. } => {
                write!(f, "Could not resolve module '{}': {}", spec, message)
            }
            NicyError::RuntimeError {
                message,
                file,
                line,
                ..
            } => {
                if let Some(file) = file {
                    if let Some(line) = line {
                        write!(f, "{}:{}: {}", file, line, message)
                    } else {
                        write!(f, "{}: {}", file, message)
                    }
                } else {
                    write!(f, "{}", message)
                }
            }
            NicyError::TaskError { task_type, message } => {
                write!(f, "task.{} error: {}", task_type, message)
            }
            NicyError::PanicError { context, payload } => {
                write!(f, "Panic in {}: {}", context, payload)
            }
            NicyError::FileError {
                path,
                operation,
                message,
            } => {
                write!(f, "Failed to {} '{}': {}", operation, path, message)
            }
            NicyError::RuntimeErrorGeneric { context, message } => {
                write!(f, "{}: {}", context, message)
            }
        }
    }
}

impl NicyError {
    pub fn level(&self) -> ErrorLevel {
        ErrorLevel::Error
    }

    pub fn title(&self) -> &'static str {
        match self {
            NicyError::LoadError { .. } => "LoadError",
            NicyError::RequireError { .. } => "RequireError",
            NicyError::RuntimeError { .. } => "RuntimeError",
            NicyError::TaskError { .. } => "TaskError",
            NicyError::PanicError { .. } => "PanicError",
            NicyError::FileError { .. } => "FileError",
            NicyError::RuntimeErrorGeneric { .. } => "RuntimeError",
        }
    }

    /// Get the appropriate error code for this error type
    /// Uses standard Luau codes where applicable, nicy-specific codes otherwise
    pub fn code(&self) -> i32 {
        match self {
            NicyError::LoadError { .. } => error_codes::LUA_ERRSYNTAX,
            NicyError::RequireError { .. } => error_codes::NICY_ERR_MODULE_NOT_FOUND,
            NicyError::RuntimeError { .. } => error_codes::LUA_ERRRUN,
            NicyError::TaskError { .. } => error_codes::NICY_ERR_TASK_CRASH,
            NicyError::PanicError { .. } => error_codes::NICY_ERR_NATIVE_CRASH,
            NicyError::FileError { .. } => error_codes::LUA_ERRFILE,
            NicyError::RuntimeErrorGeneric { .. } => error_codes::LUA_ERRRUN,
        }
    }
}

// ============================================================================
// Path Cleanup
// ============================================================================

/// Clean up Windows extended path prefix \\?\
pub fn clean_path(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        path.to_string()
    }
}

/// Clean path in a string, handling common patterns
pub fn clean_path_in_string(s: &str) -> String {
    s.replace(r"\\?\", "")
}

// ============================================================================
// Verbose Mode Check
// ============================================================================

fn is_verbose_mode() -> bool {
    env::var("NICY_VERBOSE_ERRORS")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false)
}

// ============================================================================
// Error Formatter
// ============================================================================

pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Format error - concise by default, verbose if NICY_VERBOSE_ERRORS=1
    pub fn format_error(err: &NicyError) -> String {
        if is_verbose_mode() {
            Self::format_verbose(err)
        } else {
            Self::format_concise(err)
        }
    }

    /// Concise error output (default)
    fn format_concise(err: &NicyError) -> String {
        let mut output = String::new();

        match err {
            NicyError::RequireError {
                spec,
                chain,
                message,
                ..
            } => {
                let clean_msg = clean_path_in_string(message);
                output.push_str(&format!(
                    "{}require error:{} {}\n",
                    Colors::red(),
                    Colors::reset(),
                    spec
                ));
                output.push_str(&format!(
                    "  {}{}{}\n",
                    Colors::dim(),
                    clean_msg,
                    Colors::reset()
                ));

                if !chain.is_empty()
                    && let Some(frame) = chain.frames.last()
                {
                    let line_info = frame.line.map(|l| format!(":{}", l)).unwrap_or_default();
                    output.push_str(&format!("  at {}{}\n", frame.file, line_info));
                }
            }

            NicyError::RuntimeError {
                message,
                file,
                line,
                ..
            } => {
                let clean_msg = clean_path_in_string(message);
                if let Some(file) = file {
                    let clean_file = clean_path(file);
                    if let Some(line) = line {
                        output.push_str(&format!(
                            "{}runtime error:{} {}:{}\n",
                            Colors::red(),
                            Colors::reset(),
                            clean_file,
                            line
                        ));
                    } else {
                        output.push_str(&format!(
                            "{}runtime error:{} {}\n",
                            Colors::red(),
                            Colors::reset(),
                            clean_file
                        ));
                    }
                } else {
                    output.push_str(&format!(
                        "{}runtime error:{}\n",
                        Colors::red(),
                        Colors::reset()
                    ));
                }
                output.push_str(&format!(
                    "  {}{}{}\n",
                    Colors::dim(),
                    clean_msg,
                    Colors::reset()
                ));
            }

            NicyError::TaskError { task_type, message } => {
                let clean_msg = clean_path_in_string(message);
                output.push_str(&format!(
                    "{}task.{} error:{}\n",
                    Colors::yellow(),
                    task_type,
                    Colors::reset()
                ));
                output.push_str(&format!(
                    "  {}{}{}\n",
                    Colors::dim(),
                    clean_msg,
                    Colors::reset()
                ));
            }

            NicyError::LoadError {
                path,
                line,
                message,
                ..
            } => {
                let clean_path = clean_path(path);
                let loc = match line {
                    Some(l) => format!("{}:{}", clean_path, l),
                    None => clean_path,
                };
                output.push_str(&format!(
                    "{}load error:{} {}\n",
                    Colors::red(),
                    Colors::reset(),
                    loc
                ));
                output.push_str(&format!(
                    "  {}{}{}\n",
                    Colors::dim(),
                    message,
                    Colors::reset()
                ));
            }

            NicyError::PanicError { context, payload } => {
                output.push_str(&format!("panic in {}\n", context));
                output.push_str(&format!("  {}\n", payload));
            }

            NicyError::FileError {
                path,
                operation,
                message,
            } => {
                let clean_path = clean_path(path);
                output.push_str(&format!("file error: {} '{}'\n", operation, clean_path));
                output.push_str(&format!("  {}\n", message));
            }

            NicyError::RuntimeErrorGeneric { context, message } => {
                output.push_str(&format!("{} error\n", context));
                output.push_str(&format!("  {}\n", message));
            }
        }

        output
    }

    /// Verbose PowerShell-style hierarchical output
    fn format_verbose(err: &NicyError) -> String {
        let mut output = String::new();
        let c = Colors::cyan();
        let res = Colors::reset();

        match err {
            NicyError::RequireError {
                spec,
                resolved_path,
                chain,
                message,
            } => {
                let clean_msg = clean_path_in_string(message);
                output.push_str(&format!(
                    "{}require:{} Could not resolve module '{}'\n",
                    Colors::red(),
                    res,
                    spec
                ));
                output.push('\n');

                output.push_str(&format!(
                    "{}Exception{}             :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}ErrorRecord{}          :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "        {}Exception{}             :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "            {}Message{} : {}\n",
                    Colors::bold(),
                    res,
                    clean_msg
                ));
                output.push_str(&format!(
                    "            {}Code{}    : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));

                if let Some(path) = resolved_path {
                    output.push_str(&format!(
                        "        {}TargetObject{}          : {}\n",
                        Colors::bold(),
                        res,
                        clean_path(path)
                    ));
                } else {
                    output.push_str(&format!(
                        "        {}TargetObject{}          : {}\n",
                        Colors::bold(),
                        res,
                        spec
                    ));
                }

                output.push_str(&format!(
                    "        {}CategoryInfo{}          : {}ModuleNotFound{}\n",
                    Colors::bold(),
                    res,
                    c,
                    res
                ));
                output.push_str(&format!(
                    "        {}FullyQualifiedErrorId{} : {}RequireFailed,{}{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    spec,
                    res
                ));

                if !chain.is_empty() {
                    output.push_str(&format!(
                        "    {}RequireChain{}           :\n",
                        Colors::dim(),
                        res
                    ));
                    let chain_str = chain.format_chain();
                    for line in chain_str.lines() {
                        output.push_str(&format!("        {}{}\n", Colors::dim(), line));
                    }
                }

                output.push_str(&format!(
                    "    {}Message{}              : {}\n",
                    Colors::bold(),
                    res,
                    clean_msg
                ));
                output.push_str(&format!(
                    "    {}Source{}               : {}nicyruntime.require{}\n",
                    Colors::bold(),
                    res,
                    c,
                    res
                ));
                output.push_str(&format!(
                    "    {}Code{}                 : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));

                if !chain.is_empty() {
                    output.push_str(&format!(
                        "{}InvocationInfo{}        :\n",
                        Colors::bold(),
                        res
                    ));
                    if let Some(frame) = chain.frames.last() {
                        let clean_file = clean_path(&frame.file);
                        output.push_str(&format!(
                            "    {}ScriptFile{}       : {}\n",
                            Colors::dim(),
                            res,
                            clean_file
                        ));
                        if let Some(l) = frame.line {
                            output.push_str(&format!(
                                "    {}ScriptLineNumber{} : {}\n",
                                Colors::dim(),
                                res,
                                l
                            ));
                        }
                        output.push_str(&format!(
                            "    {}Line{}             : local mod = require(\"{}\")\n",
                            Colors::dim(),
                            res,
                            spec
                        ));
                        output.push_str(&format!(
                            "    {}PositionMessage{}  : At {}:{}{}\n",
                            Colors::dim(),
                            res,
                            clean_file,
                            frame.line.unwrap_or(0),
                            res
                        ));
                    }
                    output.push_str(&format!(
                        "    {}CommandOrigin{}    : Internal\n",
                        Colors::dim(),
                        res
                    ));
                }

                output.push_str(&format!(
                    "{}ScriptStackTrace{}      :\n",
                    Colors::bold(),
                    res
                ));
                if !chain.is_empty() {
                    for frame in chain.frames.iter().rev() {
                        let clean_file = clean_path(&frame.file);
                        if let Some(l) = frame.line {
                            output.push_str(&format!(
                                "    {}at require('{}'), {}:{}{}\n",
                                Colors::dim(),
                                spec,
                                clean_file,
                                l,
                                res
                            ));
                        } else {
                            output.push_str(&format!(
                                "    {}at require('{}'), {}{}\n",
                                Colors::dim(),
                                spec,
                                clean_file,
                                res
                            ));
                        }
                    }
                }
                output.push_str(&format!(
                    "    {}at <ScriptBlock>, <entry>{}\n",
                    Colors::dim(),
                    res
                ));
            }

            NicyError::RuntimeError {
                message,
                stack_trace,
                file,
                line,
            } => {
                let clean_msg = clean_path_in_string(message);
                if let Some(f) = file {
                    let clean_file = clean_path(f);
                    if let Some(l) = line {
                        output.push_str(&format!(
                            "{}runtime:{} {}:{}: {}\n",
                            Colors::red(),
                            res,
                            clean_file,
                            l,
                            clean_msg
                        ));
                    } else {
                        output.push_str(&format!(
                            "{}runtime:{} {}: {}\n",
                            Colors::red(),
                            res,
                            clean_file,
                            clean_msg
                        ));
                    }
                } else {
                    output.push_str(&format!("{}runtime:{} {}\n", Colors::red(), res, clean_msg));
                }
                output.push('\n');

                output.push_str(&format!(
                    "{}Exception{}             :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}ErrorRecord{}          :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "        {}Exception{}             :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "            {}Message{} : {}\n",
                    Colors::bold(),
                    res,
                    clean_msg
                ));
                output.push_str(&format!(
                    "            {}Code{}    : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));

                if let Some(f) = file {
                    output.push_str(&format!(
                        "        {}TargetObject{}          : {}\n",
                        Colors::bold(),
                        res,
                        clean_path(f)
                    ));
                }

                output.push_str(&format!(
                    "        {}CategoryInfo{}          : {}RuntimeError{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    res
                ));
                output.push_str(&format!(
                    "        {}FullyQualifiedErrorId{} : {}LuaRuntimeError{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    res
                ));

                output.push_str(&format!(
                    "    {}Message{}              : {}\n",
                    Colors::bold(),
                    res,
                    clean_msg
                ));
                output.push_str(&format!(
                    "    {}Source{}               : {}nicyruntime.runtime{}\n",
                    Colors::bold(),
                    res,
                    c,
                    res
                ));
                output.push_str(&format!(
                    "    {}Code{}                 : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));

                if let Some(f) = file {
                    let clean_file = clean_path(f);
                    output.push_str(&format!(
                        "{}InvocationInfo{}        :\n",
                        Colors::bold(),
                        res
                    ));
                    output.push_str(&format!(
                        "    {}ScriptFile{}       : {}\n",
                        Colors::dim(),
                        res,
                        clean_file
                    ));
                    if let Some(l) = line {
                        output.push_str(&format!(
                            "    {}ScriptLineNumber{} : {}\n",
                            Colors::dim(),
                            res,
                            l
                        ));
                    }
                    output.push_str(&format!(
                        "    {}CommandOrigin{}    : Internal\n",
                        Colors::dim(),
                        res
                    ));
                }

                output.push_str(&format!(
                    "{}ScriptStackTrace{}      :\n",
                    Colors::bold(),
                    res
                ));
                if let Some(stack) = stack_trace
                    && !stack.is_empty()
                {
                    for stack_line in stack.lines() {
                            output.push_str(&format!(
                                "    {}{}{}\n",
                                Colors::dim(),
                                clean_path_in_string(stack_line.trim()),
                                res
                            ));
                        }
                    }
                if let (Some(f), Some(l)) = (file, line) {
                    output.push_str(&format!(
                        "    {}at <main>, {}:{}{}\n",
                        Colors::dim(),
                        clean_path(f),
                        l,
                        res
                    ));
                }
            }

            NicyError::TaskError { task_type, message } => {
                let clean_msg = clean_path_in_string(message);
                output.push_str(&format!(
                    "{}task.{}:{} {}\n",
                    Colors::yellow(),
                    task_type,
                    res,
                    clean_msg
                ));
                output.push('\n');

                output.push_str(&format!(
                    "{}Exception{}             :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}ErrorRecord{}          :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "        {}Exception{}             :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "            {}Message{} : {}\n",
                    Colors::bold(),
                    res,
                    clean_msg
                ));
                output.push_str(&format!(
                    "            {}Code{}    : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));
                output.push_str(&format!(
                    "        {}CategoryInfo{}          : {}TaskError{}\n",
                    Colors::bold(),
                    res,
                    Colors::yellow(),
                    res
                ));
                output.push_str(&format!(
                    "        {}FullyQualifiedErrorId{} : {}TaskFailed,task.{}{}\n",
                    Colors::bold(),
                    res,
                    Colors::yellow(),
                    task_type,
                    res
                ));

                output.push_str(&format!(
                    "    {}Message{}              : {}\n",
                    Colors::bold(),
                    res,
                    clean_msg
                ));
                output.push_str(&format!(
                    "    {}Source{}               : {}nicyruntime.task_scheduler{}\n",
                    Colors::bold(),
                    res,
                    c,
                    res
                ));
                output.push_str(&format!(
                    "    {}Code{}                 : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));

                output.push_str(&format!(
                    "{}InvocationInfo{}        :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}TaskType{}         : {}\n",
                    Colors::dim(),
                    res,
                    task_type
                ));
                output.push_str(&format!(
                    "    {}CommandOrigin{}    : Internal\n",
                    Colors::dim(),
                    res
                ));

                output.push_str(&format!(
                    "{}ScriptStackTrace{}      :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}at task.{}{} callback\n",
                    Colors::dim(),
                    task_type,
                    res
                ));
                output.push_str(&format!(
                    "    {}at <task_scheduler>{}\n",
                    Colors::dim(),
                    res
                ));
            }

            NicyError::LoadError {
                path,
                line,
                column,
                message,
            } => {
                let clean_path = clean_path(path);
                let loc = match (line, column) {
                    (Some(l), Some(c_val)) => format!("{}:{}:{}", clean_path, l, c_val),
                    (Some(l), None) => format!("{}:{}", clean_path, l),
                    _ => clean_path.clone(),
                };

                output.push_str(&format!(
                    "{}load:{} Failed to load '{}': {}\n",
                    Colors::red(),
                    res,
                    loc,
                    message
                ));
                output.push('\n');

                output.push_str(&format!(
                    "{}Exception{}             :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}ErrorRecord{}          :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "        {}Exception{}             :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "            {}Message{} : {}\n",
                    Colors::bold(),
                    res,
                    message
                ));
                output.push_str(&format!(
                    "            {}Code{}    : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));
                output.push_str(&format!(
                    "        {}TargetObject{}          : {}\n",
                    Colors::bold(),
                    res,
                    clean_path
                ));
                output.push_str(&format!(
                    "        {}CategoryInfo{}          : {}LoadError{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    res
                ));
                output.push_str(&format!(
                    "        {}FullyQualifiedErrorId{} : {}LoadFailed{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    res
                ));

                output.push_str(&format!(
                    "    {}Message{}              : {}\n",
                    Colors::bold(),
                    res,
                    message
                ));
                output.push_str(&format!(
                    "    {}Source{}               : {}nicyruntime.loader{}\n",
                    Colors::bold(),
                    res,
                    c,
                    res
                ));
                output.push_str(&format!(
                    "    {}Code{}                 : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));

                output.push_str(&format!(
                    "{}InvocationInfo{}        :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}ScriptFile{}       : {}\n",
                    Colors::dim(),
                    res,
                    clean_path
                ));
                if let Some(l) = line {
                    output.push_str(&format!(
                        "    {}ScriptLineNumber{} : {}\n",
                        Colors::dim(),
                        res,
                        l
                    ));
                }
                output.push_str(&format!(
                    "    {}CommandOrigin{}    : Internal\n",
                    Colors::dim(),
                    res
                ));

                output.push_str(&format!(
                    "{}ScriptStackTrace{}      :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}at load('{}'), {}{}\n",
                    Colors::dim(),
                    clean_path,
                    loc,
                    res
                ));
            }

            NicyError::PanicError { context, payload } => {
                output.push_str(&format!("{}panic:{} {}\n", Colors::red(), res, payload));
                output.push('\n');

                output.push_str(&format!(
                    "{}Exception{}             :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}ErrorRecord{}          :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "        {}Exception{}             :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "            {}Message{} : {}\n",
                    Colors::bold(),
                    res,
                    payload
                ));
                output.push_str(&format!(
                    "            {}Code{}    : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));
                output.push_str(&format!(
                    "        {}CategoryInfo{}          : {}PanicError{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    res
                ));
                output.push_str(&format!(
                    "        {}FullyQualifiedErrorId{} : {}RustPanic,{}{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    context,
                    res
                ));

                output.push_str(&format!(
                    "    {}Message{}              : {}\n",
                    Colors::bold(),
                    res,
                    payload
                ));
                output.push_str(&format!(
                    "    {}Source{}               : {}nicyruntime.panic_handler{}\n",
                    Colors::bold(),
                    res,
                    c,
                    res
                ));
                output.push_str(&format!(
                    "    {}Code{}                 : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));

                output.push_str(&format!(
                    "{}InvocationInfo{}        :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}Context{}          : {}\n",
                    Colors::dim(),
                    res,
                    context
                ));
                output.push_str(&format!(
                    "    {}CommandOrigin{}    : Internal\n",
                    Colors::dim(),
                    res
                ));

                output.push_str(&format!(
                    "{}ScriptStackTrace{}      :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!("    {}at panic in '{}'\n", Colors::dim(), context));
            }

            NicyError::FileError {
                path,
                operation,
                message,
            } => {
                let clean_path = clean_path(path);
                output.push_str(&format!(
                    "{}file:{} Failed to {} '{}': {}\n",
                    Colors::red(),
                    res,
                    operation,
                    clean_path,
                    message
                ));
                output.push('\n');

                output.push_str(&format!(
                    "{}Exception{}             :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}ErrorRecord{}          :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "        {}Exception{}             :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "            {}Message{} : {}\n",
                    Colors::bold(),
                    res,
                    message
                ));
                output.push_str(&format!(
                    "            {}Code{}    : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));
                output.push_str(&format!(
                    "        {}TargetObject{}          : {}\n",
                    Colors::bold(),
                    res,
                    clean_path
                ));
                output.push_str(&format!(
                    "        {}CategoryInfo{}          : {}FileError{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    res
                ));
                output.push_str(&format!(
                    "        {}FullyQualifiedErrorId{} : {}{}Failed{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    operation,
                    res
                ));

                output.push_str(&format!(
                    "    {}Message{}              : {}\n",
                    Colors::bold(),
                    res,
                    message
                ));
                output.push_str(&format!(
                    "    {}Source{}               : {}nicyruntime.filesystem{}\n",
                    Colors::bold(),
                    res,
                    c,
                    res
                ));
                output.push_str(&format!(
                    "    {}Code{}                 : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));

                output.push_str(&format!(
                    "{}InvocationInfo{}        :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}FilePath{}         : {}\n",
                    Colors::dim(),
                    res,
                    clean_path
                ));
                output.push_str(&format!(
                    "    {}Operation{}        : {}\n",
                    Colors::dim(),
                    res,
                    operation
                ));
                output.push_str(&format!(
                    "    {}CommandOrigin{}    : Internal\n",
                    Colors::dim(),
                    res
                ));

                output.push_str(&format!(
                    "{}ScriptStackTrace{}      :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}at {}('{}')\n",
                    Colors::dim(),
                    operation,
                    clean_path
                ));
            }

            NicyError::RuntimeErrorGeneric { context, message } => {
                output.push_str(&format!(
                    "{}runtime:{} {}: {}\n",
                    Colors::red(),
                    res,
                    context,
                    message
                ));
                output.push('\n');

                output.push_str(&format!(
                    "{}Exception{}             :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}ErrorRecord{}          :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "        {}Exception{}             :\n",
                    Colors::dim(),
                    res
                ));
                output.push_str(&format!(
                    "            {}Message{} : {}\n",
                    Colors::bold(),
                    res,
                    message
                ));
                output.push_str(&format!(
                    "            {}Code{}    : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));
                output.push_str(&format!(
                    "        {}CategoryInfo{}          : {}RuntimeError{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    res
                ));
                output.push_str(&format!(
                    "        {}FullyQualifiedErrorId{} : {}GenericError,{}{}\n",
                    Colors::bold(),
                    res,
                    Colors::red(),
                    context,
                    res
                ));

                output.push_str(&format!(
                    "    {}Message{}              : {}\n",
                    Colors::bold(),
                    res,
                    message
                ));
                output.push_str(&format!(
                    "    {}Source{}               : {}nicyruntime.runtime{}\n",
                    Colors::bold(),
                    res,
                    c,
                    res
                ));
                output.push_str(&format!(
                    "    {}Code{}                 : {}{} ({}){}\n",
                    Colors::bold(),
                    res,
                    c,
                    err.code(),
                    error_codes::code_to_name(err.code()),
                    res
                ));

                output.push_str(&format!(
                    "{}InvocationInfo{}        :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!(
                    "    {}Context{}          : {}\n",
                    Colors::dim(),
                    res,
                    context
                ));
                output.push_str(&format!(
                    "    {}CommandOrigin{}    : Internal\n",
                    Colors::dim(),
                    res
                ));

                output.push_str(&format!(
                    "{}ScriptStackTrace{}      :\n",
                    Colors::bold(),
                    res
                ));
                output.push_str(&format!("    {}at '{}'\n", Colors::dim(), context));
            }
        }

        output
    }

    /// Format warning - concise by default
    pub fn format_warning(message: &str) -> String {
        if is_verbose_mode() {
            let mut output = String::new();
            output.push_str(&format!(
                "{}warn:{} {}\n",
                Colors::yellow(),
                Colors::reset(),
                message
            ));
            output.push('\n');
            output.push_str(&format!(
                "{}Exception{}             :\n",
                Colors::bold(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "    {}ErrorRecord{}          :\n",
                Colors::dim(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "        {}Exception{}             :\n",
                Colors::dim(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "            {}Message{} : {}\n",
                Colors::bold(),
                Colors::reset(),
                message
            ));
            output.push_str(&format!(
                "            {}Code{}    : {}{} ({}){}\n",
                Colors::bold(),
                Colors::reset(),
                Colors::cyan(),
                error_codes::LUA_OK,
                error_codes::code_to_name(error_codes::LUA_OK),
                Colors::reset()
            ));
            output.push_str(&format!(
                "        {}CategoryInfo{}          : {}Warning{}\n",
                Colors::bold(),
                Colors::reset(),
                Colors::yellow(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "        {}FullyQualifiedErrorId{} : {}RuntimeWarning{}\n",
                Colors::bold(),
                Colors::reset(),
                Colors::yellow(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "    {}Message{}              : {}\n",
                Colors::bold(),
                Colors::reset(),
                message
            ));
            output.push_str(&format!(
                "    {}Source{}               : {}nicyruntime.warn{}\n",
                Colors::bold(),
                Colors::reset(),
                Colors::cyan(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "    {}Code{}                 : {}{} ({}){}\n",
                Colors::bold(),
                Colors::reset(),
                Colors::cyan(),
                error_codes::LUA_OK,
                error_codes::code_to_name(error_codes::LUA_OK),
                Colors::reset()
            ));
            output.push_str(&format!(
                "{}InvocationInfo{}        :\n",
                Colors::bold(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "    {}CommandOrigin{}    : Internal\n",
                Colors::dim(),
                Colors::reset()
            ));
            output
        } else {
            format!(
                "{}warning:{} {}\n",
                Colors::yellow(),
                Colors::reset(),
                message
            )
        }
    }

    /// Format info - concise by default
    pub fn format_info(message: &str) -> String {
        if is_verbose_mode() {
            let mut output = String::new();
            output.push_str(&format!(
                "{}info:{} {}\n",
                Colors::cyan(),
                Colors::reset(),
                message
            ));
            output.push('\n');
            output.push_str(&format!(
                "{}Exception{}             :\n",
                Colors::bold(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "    {}ErrorRecord{}          :\n",
                Colors::dim(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "        {}Exception{}             :\n",
                Colors::dim(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "            {}Message{} : {}\n",
                Colors::bold(),
                Colors::reset(),
                message
            ));
            output.push_str(&format!(
                "            {}Code{}    : {}{} ({}){}\n",
                Colors::bold(),
                Colors::reset(),
                Colors::cyan(),
                error_codes::LUA_OK,
                error_codes::code_to_name(error_codes::LUA_OK),
                Colors::reset()
            ));
            output.push_str(&format!(
                "        {}CategoryInfo{}          : {}Information{}\n",
                Colors::bold(),
                Colors::reset(),
                Colors::cyan(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "        {}FullyQualifiedErrorId{} : {}RuntimeInfo{}\n",
                Colors::bold(),
                Colors::reset(),
                Colors::cyan(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "    {}Message{}              : {}\n",
                Colors::bold(),
                Colors::reset(),
                message
            ));
            output.push_str(&format!(
                "    {}Source{}               : {}nicyruntime.info{}\n",
                Colors::bold(),
                Colors::reset(),
                Colors::cyan(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "    {}Code{}                 : {}{} ({}){}\n",
                Colors::bold(),
                Colors::reset(),
                Colors::cyan(),
                error_codes::LUA_OK,
                error_codes::code_to_name(error_codes::LUA_OK),
                Colors::reset()
            ));
            output.push_str(&format!(
                "{}InvocationInfo{}        :\n",
                Colors::bold(),
                Colors::reset()
            ));
            output.push_str(&format!(
                "    {}CommandOrigin{}    : Internal\n",
                Colors::dim(),
                Colors::reset()
            ));
            output
        } else {
            format!("{}info:{} {}\n", Colors::cyan(), Colors::reset(), message)
        }
    }
}

/// Capture stack trace from Lua state using debug.traceback
#[allow(dead_code)]
pub unsafe fn capture_lua_stack_trace(l: *mut crate::LuauState) -> Option<String> {
    use std::ffi::CStr;

    unsafe {
        // Push debug.traceback onto stack
        lua::lua_getglobal(l, c"debug".as_ptr() as *const _);
        if lua::lua_type(l, -1) != lua::LUA_TTABLE {
            lua::lua_pop(l, 1);
            return None;
        }

        lua::lua_getfield(l, -1, c"traceback".as_ptr() as *const _);
        lua::lua_remove(l, -2); // Remove debug table

        if lua::lua_type(l, -1) != lua::LUA_TFUNCTION {
            lua::lua_pop(l, 1);
            return None;
        }

        // Call debug.traceback()
        if lua::lua_pcall(l, 0, 1, 0) != 0 {
            lua::lua_pop(l, 1);
            return None;
        }

        let traceback_ptr = lua::lua_tostring(l, -1);
        let traceback = if !traceback_ptr.is_null() {
            let s = CStr::from_ptr(traceback_ptr).to_string_lossy().to_string();
            Some(clean_path_in_string(&s))
        } else {
            None
        };

        lua::lua_pop(l, 1);
        traceback
    }
}

// ============================================================================
// Error Reporter
// ============================================================================

/// Per-coroutine pcall tracking using a Rust-side HashMap.
/// Keyed by the coroutine pointer address, which is unique per thread.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static PCALL_STATE: OnceLock<Mutex<HashMap<usize, bool>>> = OnceLock::new();

fn pcall_state() -> &'static Mutex<HashMap<usize, bool>> {
    PCALL_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

// Fallback for contexts where no Lua state is available.
thread_local! {
    static INSIDE_PCALL_FALLBACK: Cell<bool> = const { Cell::new(false) };
}

pub struct ErrorReporter;

impl ErrorReporter {
    /// Report an error - will print UNLESS inside pcall.
    /// Uses thread-local fallback since no Lua state is available.
    pub fn report(err: &NicyError) {
        Self::report_with_state(None, err)
    }

    /// Report an error with Lua state for accurate per-coroutine pcall detection.
    pub fn report_with_state(l: Option<*mut crate::LuauState>, err: &NicyError) {
        if Self::is_inside_pcall(l) {
            return;
        }

        let formatted = ErrorFormatter::format_error(err);
        eprint!("{}", formatted);
        log_to_file(&formatted);
    }

    /// Report a fatal error and terminate
    pub fn report_and_exit(err: &NicyError) {
        let formatted = ErrorFormatter::format_error(err);
        eprint!("{}", formatted);
        log_to_file(&formatted);
        std::process::exit(1);
    }

    /// Report a warning - always prints
    pub fn warn(message: &str) {
        let formatted = ErrorFormatter::format_warning(message);
        eprint!("{}", formatted);
        log_to_file(&formatted);
    }

    /// Report info - always prints
    pub fn info(message: &str) {
        let formatted = ErrorFormatter::format_info(message);
        eprint!("{}", formatted);
        log_to_file(&formatted);
    }

    /// Report a fatal error - always prints, even inside pcall
    pub fn fatal(err: &NicyError) {
        let formatted = ErrorFormatter::format_error(err);
        eprint!("{}", formatted);
        log_to_file(&formatted);
    }

    /// Mark that we're entering a pcall on the given Lua state/coroutine.
    pub fn enter_pcall(l: *mut crate::LuauState) {
        let key = l as usize;
        let mut map = pcall_state().lock().unwrap();
        map.insert(key, true);
    }

    /// Mark that we're exiting a pcall on the given Lua state/coroutine.
    pub fn exit_pcall(l: *mut crate::LuauState) {
        let key = l as usize;
        let mut map = pcall_state().lock().unwrap();
        map.remove(&key);
    }

    /// Check if we're inside a pcall for the given Lua state/coroutine.
    /// If `l` is provided, checks the per-coroutine state.
    /// If `l` is None, falls back to the thread-local (for contexts without Lua state).
    pub fn is_inside_pcall(l: Option<*mut crate::LuauState>) -> bool {
        match l {
            Some(l) => {
                let key = l as usize;
                let map = pcall_state().lock().unwrap();
                map.get(&key).copied().unwrap_or(false)
            }
            None => INSIDE_PCALL_FALLBACK.with(|c| c.get()),
        }
    }

    /// # Safety
    /// Caller must ensure `l` is a valid, non-null pointer to an open `lua_State`.
    /// The error message must be on top of the Lua stack.
    pub unsafe fn report_lua_error(l: *mut crate::LuauState, _context: &str) {
        use std::ffi::CStr;

        let err_ptr = unsafe { lua::lua_tostring(l, -1) };
        let message = if !err_ptr.is_null() {
            clean_path_in_string(&unsafe { CStr::from_ptr(err_ptr) }.to_string_lossy())
        } else {
            "unknown error".to_string()
        };

        let (file, line) = Self::extract_location(&message);

        let err = NicyError::RuntimeError {
            message,
            stack_trace: None,
            file,
            line,
        };

        Self::report(&err);
    }

    /// Extract file:line from error message like "file.lua:42: error message"
    fn extract_location(message: &str) -> (Option<String>, Option<u32>) {
        if let Some(colon_pos) = message.find(':')
            && let rest = &message[colon_pos + 1..]
            && let Some(end_line) = rest.find(|c: char| !c.is_ascii_digit())
            && end_line > 0
            && let Ok(line) = rest[..end_line].parse::<u32>()
        {
            return (Some(message[..colon_pos].to_string()), Some(line));
        }
        (None, None)
    }
}

// ============================================================================
// Helper macros
// ============================================================================

#[macro_export]
macro_rules! nicy_error {
    ($err:expr) => {
        $crate::error::ErrorReporter::report(&$err);
    };
}

#[macro_export]
macro_rules! nicy_warn {
    ($msg:expr) => {
        $crate::error::ErrorReporter::warn($msg);
    };
}

#[macro_export]
macro_rules! nicy_info {
    ($msg:expr) => {
        $crate::error::ErrorReporter::info($msg);
    };
}

#[macro_export]
macro_rules! nicy_fatal {
    ($err:expr) => {
        $crate::error::ErrorReporter::fatal(&$err);
    };
}

// ============================================================================
// Convenience constructors
// ============================================================================

impl NicyError {
    pub fn load_error(path: impl Into<String>, message: impl Into<String>) -> Self {
        NicyError::LoadError {
            path: path.into(),
            line: None,
            column: None,
            message: message.into(),
        }
    }

    pub fn require_error(
        spec: impl Into<String>,
        message: impl Into<String>,
        chain: RequireChain,
    ) -> Self {
        NicyError::RequireError {
            spec: spec.into(),
            resolved_path: None,
            chain,
            message: message.into(),
        }
    }

    pub fn runtime_error(message: impl Into<String>) -> Self {
        NicyError::RuntimeError {
            message: message.into(),
            stack_trace: None,
            file: None,
            line: None,
        }
    }

    pub fn task_error(task_type: &'static str, message: impl Into<String>) -> Self {
        NicyError::TaskError {
            task_type,
            message: message.into(),
        }
    }

    pub fn panic_error(context: &'static str, payload: impl Into<String>) -> Self {
        NicyError::PanicError {
            context,
            payload: payload.into(),
        }
    }

    pub fn file_error(
        path: impl Into<String>,
        operation: &'static str,
        message: impl Into<String>,
    ) -> Self {
        NicyError::FileError {
            path: path.into(),
            operation,
            message: message.into(),
        }
    }
}

// ============================================================================
// File Logger
// ============================================================================

use std::fs::OpenOptions;
use std::io::Write;

thread_local! {
    static LOG_FILE: Mutex<Option<std::fs::File>> = const { Mutex::new(None) };
}

/// Initialize file logging
pub fn init_file_logging(path: &str) -> Result<(), String> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    LOG_FILE.with(|f| {
        *f.lock().unwrap() = Some(file);
    });

    Ok(())
}

/// Write a message to the log file
fn log_to_file(message: &str) {
    LOG_FILE.with(|f| {
        if let Some(file) = f.lock().unwrap().as_mut() {
            let _ = writeln!(file, "{}", message);
        }
    });
}

/// Check if file logging is enabled via NICY_LOG_FILE env var
fn get_log_file_path() -> Option<String> {
    std::env::var("NICY_LOG_FILE").ok()
}

/// Auto-initialize file logging if NICY_LOG_FILE is set
pub fn auto_init_logging() {
    if let Some(path) = get_log_file_path()
        && let Err(e) = init_file_logging(&path)
    {
        eprintln!("Failed to init logging: {}", e);
    }
}
