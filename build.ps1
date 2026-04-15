param(
    [string]$target = "user",
    [switch]$force
)

$ErrorActionPreference = "Stop"

$TargetMap = @{
    "android-arm" = "aarch64-linux-android"
    "android-v7"  = "armv7-linux-androideabi"
    "linux-arm"   = "aarch64-unknown-linux-gnu.2.17"
    "linux-x64"   = "x86_64-unknown-linux-gnu.2.17"
    "linux-x86"   = "i686-unknown-linux-gnu.2.17"
    "mac-arm"     = "aarch64-apple-darwin"
    "mac-x64"     = "x86_64-apple-darwin"
    "win-arm"     = "aarch64-pc-windows-msvc"
    "win-x64"     = "x86_64-pc-windows-msvc"
    "win-x86"     = "i686-pc-windows-msvc"
}

function Assert-Command([string]$name) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        throw "Comando obrigatorio nao encontrado: $name"
    }
}

function Get-UserTarget {
    if ($IsWindows) {
        $arch = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString().ToLowerInvariant()
        switch ($arch) {
            "x64" { return "x86_64-pc-windows-msvc" }
            "arm64" { return "aarch64-pc-windows-msvc" }
            "x86" { return "i686-pc-windows-msvc" }
            default { throw "Arquitetura Windows nao suportada: $arch" }
        }
    }

    Assert-Command "rustc"
    $hostLine = rustc -vV | Select-String "^host:\s+"
    if ($null -eq $hostLine) {
        throw "Nao foi possivel detectar host target via rustc -vV"
    }
    return ($hostLine.ToString() -replace "^host:\s+", "").Trim()
}

function Get-BinaryName([string]$name, [string]$kind) {
    if ($kind -eq "cli") {
        if ($name -like "win-*") { return "nicy.exe" } else { return "nicy" }
    }
    # kind = "runtime" (cdylib)
    if ($name -like "win-*") {
        return "nicyruntime.dll"
    }
    if ($name -like "mac-*") {
        return "libnicyruntime.dylib"
    }
    return "libnicyruntime.so"
}

function Get-CliManifest() {
    return "Nicy/Cargo.toml"
}

function Get-RuntimeManifest() {
    return "Runtime/Cargo.toml"
}

function Invoke-Build([string]$name, [string]$rustTarget, [string]$kind, [switch]$forceBuild, [switch]$useDefaultTarget) {
    $cleanTarget = $rustTarget -replace "\.2\.17", ""
    $pureTarget = ($rustTarget -split '\.')[0]

    $fileName = Get-BinaryName $name $kind
    $manifest = if ($kind -eq "cli") { Get-CliManifest } else { Get-RuntimeManifest }
    $binPath = if ($useDefaultTarget) { "target/release/$fileName" } else { "target/$cleanTarget/release/$fileName" }

    if ((Test-Path $binPath) -and -not $forceBuild) {
        Write-Host "`nSkip: $name ($kind) ja existe" -ForegroundColor Green
        return $true
    }

    if ((Test-Path $binPath) -and $forceBuild) {
        Write-Host "`nForce: recompilando $name ($kind)" -ForegroundColor Yellow
    }

    $label = if ($kind -eq "cli") { "CLI" } else { "Runtime" }
    Write-Host "`nCompilando ${label}: $name ($rustTarget)" -ForegroundColor Cyan

    if (-not $useDefaultTarget) {
        rustup target add $pureTarget | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Erro ao instalar target Rust: $pureTarget" -ForegroundColor Red
            return $false
        }
    }

    Assert-Command "cargo"
    if ($useDefaultTarget) {
        cargo build --release --manifest-path $manifest --target-dir target
    } elseif ($name -like "win-*") {
        cargo build --release --target $rustTarget --manifest-path $manifest --target-dir target
    } else {
        cargo zigbuild --release --target $rustTarget --manifest-path $manifest --target-dir target
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Erro build: $name (exit $LASTEXITCODE)" -ForegroundColor Red
        return $false
    }

    if (-not (Test-Path $binPath)) {
        Write-Host "Erro: binario nao encontrado apos build: $binPath" -ForegroundColor Red
        return $false
    }

    Write-Host "Ok: $binPath" -ForegroundColor Green
    return $true
}

function Invoke-Build-All([string]$name, [string]$rustTarget, [switch]$forceBuild, [switch]$useDefaultTarget) {
    # Build CLI
    $cliOk = Invoke-Build -name $name -rustTarget $rustTarget -kind "cli" -forceBuild:$forceBuild -useDefaultTarget:$useDefaultTarget
    if (-not $cliOk) { return $false }

    # Build Runtime
    $rtOk = Invoke-Build -name $name -rustTarget $rustTarget -kind "runtime" -forceBuild:$forceBuild -useDefaultTarget:$useDefaultTarget
    if (-not $rtOk) { return $false }

    return $true
}

$targetsToBuild = if ($target -eq "all") {
    $TargetMap.GetEnumerator() | Sort-Object Name | ForEach-Object { [PSCustomObject]@{ Name = $_.Name; Value = $_.Value; UseDefault = $false } }
} elseif ($target -eq "user") {
    $userTarget = Get-UserTarget
    $userName = if ($userTarget -like "*windows*") { "win-user" } elseif ($userTarget -like "*apple-darwin") { "mac-user" } else { "linux-user" }
    @([PSCustomObject]@{ Name = $userName; Value = $userTarget; UseDefault = $false })
} elseif ($TargetMap.ContainsKey($target)) {
    @([PSCustomObject]@{ Name = $target; Value = $TargetMap[$target]; UseDefault = $false })
} else {
    throw "Target invalido: $target"
}

$failed = New-Object System.Collections.Generic.List[string]
foreach ($entry in $targetsToBuild) {
    $ok = Invoke-Build-All -name $entry.Name -rustTarget $entry.Value -forceBuild:$force -useDefaultTarget:$entry.UseDefault
    if (-not $ok) {
        $failed.Add($entry.Name)
    }
}

if ($failed.Count -gt 0) {
    Write-Host "Falhas: $($failed -join ', ')" -ForegroundColor Red
    exit 1
}

Write-Host "Build finalizado sem falhas" -ForegroundColor Green
