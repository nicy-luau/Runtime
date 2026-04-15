<#
.SYNOPSIS
    Update {{FFI_COUNT}} placeholder in docs with actual FFI function count.
.DESCRIPTION
    Scans Runtime/src/**/*.rs for #[unsafe(no_mangle)] functions,
    counts them, and replaces all occurrences of {{FFI_COUNT}} in docs.
.EXAMPLE
    .\update-ffi-count.ps1
    .\update-ffi-count.ps1 -Check
#>

param(
    [switch]$Check
)

$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path $PSScriptRoot -Parent
$ProjectRoot = Split-Path $ProjectRoot -Parent

function Get-FFICount {
    $count = 0

    $ffiExports = Join-Path $ProjectRoot "Runtime\src\ffi_exports.rs"
    $libRs = Join-Path $ProjectRoot "Runtime\src\lib.rs"

    if (Test-Path $ffiExports) {
        $c = (Select-String -Path $ffiExports -Pattern '^\s*#\[unsafe\(no_mangle\)\]' -AllMatches).Matches.Count
        $count += $c
        Write-Host "  ffi_exports.rs: $c functions" -ForegroundColor Gray
    }

    if (Test-Path $libRs) {
        $c = (Select-String -Path $libRs -Pattern '^\s*#\[unsafe\(no_mangle\)\]' -AllMatches).Matches.Count
        $count += $c
        Write-Host "  lib.rs: $c functions" -ForegroundColor Gray
    }

    return $count
}

function Find-FilesWithPlaceholder {
    $files = @()
    $searchPaths = @(
        (Join-Path $ProjectRoot "Docs\src"),
        (Join-Path $ProjectRoot "README.md"),
        (Join-Path $ProjectRoot "Runtime\README.md")
    )

    foreach ($path in $searchPaths) {
        if (Test-Path $path) {
            if ((Test-Path $path) -and (Get-Item $path).PSIsContainer) {
                $files += Get-ChildItem -Path $path -Recurse -File |
                    Where-Object { $_.Extension -match '\.(md|txt)$' } |
                    Select-String -Pattern '\{\{FFI_COUNT\}\}' |
                    ForEach-Object { $_.Path } |
                    Select-Object -Unique
            } else {
                $content = Get-Content $path -Raw
                if ($content -match '\{\{FFI_COUNT\}\}') {
                    $files += $path
                }
            }
        }
    }

    return $files | Select-Object -Unique
}

$actualCount = Get-FFICount

if ($actualCount -eq 0) {
    Write-Host "Error: No FFI functions found in Runtime/src/" -ForegroundColor Red
    exit 1
}

$coreCount = 5
$wrappers = $actualCount - 2
$coreWrappers = $wrappers - $coreCount

Write-Host "`nTotal FFI functions: $actualCount" -ForegroundColor Cyan
Write-Host "  Core functions: $coreCount" -ForegroundColor Gray
Write-Host "  Lua C API wrappers: $coreWrappers" -ForegroundColor Gray
Write-Host "  Error utilities: 2" -ForegroundColor Gray

$docsFiles = Find-FilesWithPlaceholder

if ($Check) {
    # Check mode
    if ($docsFiles.Count -gt 0) {
        Write-Host "`n✓ Found {{FFI_COUNT}} placeholder in $($docsFiles.Count) file(s)." -ForegroundColor Green

        $mismatch = $false
        foreach ($file in $docsFiles) {
            if ((Get-Content $file -Raw) -match '\{\{FFI_COUNT\}\}') {
                Write-Host "  ✓ $file (uses placeholder)" -ForegroundColor Gray
            }
        }

        if ($mismatch) {
            Write-Host "`nError: Run '.\update-ffi-count.ps1' to fix." -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "`n::warning::No {{FFI_COUNT}} placeholder found in docs." -ForegroundColor Yellow
        Write-Host "Consider adding it to keep docs in sync automatically." -ForegroundColor Yellow
    }
} else {
    # Update mode
    if ($docsFiles.Count -eq 0) {
        Write-Host "`nNo {{FFI_COUNT}} placeholder found in docs." -ForegroundColor Yellow
        Write-Host "To use this feature, replace FFI counts in docs with {{FFI_COUNT}}." -ForegroundColor Yellow
        exit 0
    }

    Write-Host "`nUpdating $($docsFiles.Count) file(s) with counts: total=$actualCount, wrappers=$coreWrappers" -ForegroundColor Cyan

    foreach ($file in $docsFiles) {
        Write-Host "  Updating: $file" -ForegroundColor Gray
        $content = Get-Content $file -Raw
        $content = $content -replace '\{\{FFI_COUNT\}\}', $actualCount
        $content = $content -replace '\{\{FFI_COUNT_MINUS_CORE\}\}', $coreWrappers
        [System.IO.File]::WriteAllText($file, $content, [System.Text.Encoding]::UTF8)
    }

    Write-Host "`n✓ Updated $($docsFiles.Count) file(s)." -ForegroundColor Green
}
