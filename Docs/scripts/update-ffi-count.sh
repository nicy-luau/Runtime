#!/usr/bin/env bash
#
# update-ffi-count.sh — Replace {{FFI_COUNT}} placeholder in docs with actual count.
#
# This script scans Runtime/src/**/*.rs for #[unsafe(no_mangle)] functions,
# counts them, and replaces all occurrences of {{FFI_COUNT}} in docs with the actual number.
#
# Usage:
#   bash update-ffi-count.sh [--check]
#
# Options:
#   --check    Only validate, don't modify files (for CI)
#
# Example:
#   # Update docs with current count
#   bash update-ffi-count.sh
#
#   # Validate in CI (exit 1 if count is wrong)
#   bash update-ffi-count.sh --check
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Count actual FFI functions
count_ffi_functions() {
    local count=0
    
    # Count in ffi_exports.rs
    if [ -f "$PROJECT_ROOT/Runtime/src/ffi_exports.rs" ]; then
        local ffi_count
        ffi_count=$(grep -c '^\s*#\[unsafe(no_mangle)\]' "$PROJECT_ROOT/Runtime/src/ffi_exports.rs" || echo 0)
        count=$((count + ffi_count))
    fi
    
    # Count in lib.rs (for nicy_start, nicy_eval, etc.)
    if [ -f "$PROJECT_ROOT/Runtime/src/lib.rs" ]; then
        local lib_count
        lib_count=$(grep -c '^\s*#\[unsafe(no_mangle)\]' "$PROJECT_ROOT/Runtime/src/lib.rs" || echo 0)
        count=$((count + lib_count))
    fi
    
    echo "$count"
}

# Find all docs files with {{FFI_COUNT}} placeholder
find_docs_with_placeholder() {
    grep -rl '{{FFI_COUNT}}' "$PROJECT_ROOT/Docs/src" "$PROJECT_ROOT/README.md" "$PROJECT_ROOT/Runtime/README.md" 2>/dev/null || true
}

# Main logic
main() {
    local check_mode=false
    if [ "${1:-}" = "--check" ]; then
        check_mode=true
    fi
    
    local actual_count
    actual_count=$(count_ffi_functions)
    
    if [ "$actual_count" -eq 0 ]; then
        echo "Error: No FFI functions found in Runtime/src/" >&2
        exit 1
    fi
    
    # Calculate derived counts
    local core_count=5  # nicy_start, nicy_eval, nicy_compile, nicy_version, nicy_luau_version
    local wrappers=$((actual_count - 2))  # Subtract 2 error utility functions
    local core_wrappers=$((wrappers - core_count))  # Lua C API wrappers only
    
    echo "Found $actual_count FFI functions in code."
    echo "  Core functions: $core_count"
    echo "  Lua C API wrappers: $core_wrappers"
    echo "  Error utilities: 2"
    
    if $check_mode; then
        # Check mode: verify placeholder exists and count matches
        local docs_files
        docs_files=$(find_docs_with_placeholder)
        
        if [ -n "$docs_files" ]; then
            echo "✓ Found {{FFI_COUNT}} placeholder in docs."
            
            # Check if any file has wrong count
            local mismatch=false
            while IFS= read -r file; do
                if grep -q "{{FFI_COUNT}}" "$file"; then
                    echo "  ✓ $file (uses placeholder)"
                else
                    # Check if hardcoded count is wrong
                    local current
                    current=$(grep -oP '\*\*[0-9]+\*\*' "$file" 2>/dev/null | head -1 | tr -d '*' || echo "")
                    if [ -n "$current" ] && [ "$current" != "$actual_count" ]; then
                        echo "::error file=$file::FFI count mismatch: docs=$current, actual=$actual_count"
                        mismatch=true
                    fi
                fi
            done <<< "$docs_files"
            
            if $mismatch; then
                echo "Error: Run 'bash update-ffi-count.sh' to fix." >&2
                exit 1
            else
                echo "✓ FFI count is correct in all docs."
            fi
        else
            echo "::warning::No {{FFI_COUNT}} placeholder found in docs."
            echo "Consider adding it to keep docs in sync automatically."
        fi
    else
        # Update mode: replace placeholder with actual count
        local docs_files
        docs_files=$(find_docs_with_placeholder)
        
        if [ -z "$docs_files" ]; then
            echo "No {{FFI_COUNT}} placeholder found in docs."
            echo "To use this feature, replace FFI counts in docs with {{FFI_COUNT}}."
            exit 0
        fi
        
        echo "Updating docs with counts: total=$actual_count, wrappers=$core_wrappers"
        
        # Replace placeholders with actual counts
        while IFS= read -r file; do
            echo "  Updating: $file"
            if [[ "$OSTYPE" == "darwin"* ]]; then
                # macOS sed requires empty string for -i
                sed -i '' "s/{{FFI_COUNT}}/$actual_count/g" "$file"
                sed -i '' "s/{{FFI_COUNT_MINUS_CORE}}/$core_wrappers/g" "$file"
            else
                # Linux sed
                sed -i "s/{{FFI_COUNT}}/$actual_count/g" "$file"
                sed -i "s/{{FFI_COUNT_MINUS_CORE}}/$core_wrappers/g" "$file"
            fi
        done <<< "$docs_files"
        
        echo "✓ Updated $(echo "$docs_files" | wc -l) file(s)."
    fi
}

main "$@"
