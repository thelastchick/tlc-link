# Install / uninstall the optional TLC LINK Kill Switch helper service.
# Requires Administrator. The portable UI works without this service by applying
# WFP filters in-process when elevated; the service keeps protection after UI exit.
param(
    [ValidateSet("install", "uninstall", "status")]
    [string]$Action = "status"
)

$ErrorActionPreference = "Stop"
$ServiceName = "TLCKillSwitch"
$DisplayName = "TLC LINK Kill Switch"
$ExeName = "tlc-ks-service.exe"

function Find-ServiceExe {
    $candidates = @(
        (Join-Path $PSScriptRoot "..\src-tauri\target\release\$ExeName"),
        (Join-Path $PSScriptRoot "..\dist\TLC-LINK-Portable\$ExeName"),
        (Join-Path (Get-Location) $ExeName)
    )
    foreach ($c in $candidates) {
        $full = [System.IO.Path]::GetFullPath($c)
        if (Test-Path $full) { return $full }
    }
    return $null
}

switch ($Action) {
    "install" {
        $exe = Find-ServiceExe
        if (-not $exe) {
            throw "Could not find $ExeName. Build the project first (cargo build --release --bin tlc-ks-service)."
        }
        $existing = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
        if ($existing) {
            Write-Host "Service already exists; stopping and recreating..."
            Stop-Service $ServiceName -Force -ErrorAction SilentlyContinue
            sc.exe delete $ServiceName | Out-Null
            Start-Sleep -Seconds 1
        }
        sc.exe create $ServiceName binPath= "`"$exe`"" start= auto DisplayName= "$DisplayName"
        if ($LASTEXITCODE -ne 0) { throw "sc create failed" }
        sc.exe description $ServiceName "Owns TLC LINK WFP Kill Switch filters so protection survives UI exit."
        Start-Service $ServiceName
        Write-Host "Installed and started $ServiceName from $exe"
    }
    "uninstall" {
        $existing = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
        if (-not $existing) {
            Write-Host "Service not installed."
            return
        }
        Stop-Service $ServiceName -Force -ErrorAction SilentlyContinue
        sc.exe delete $ServiceName | Out-Null
        Write-Host "Removed $ServiceName. Run Emergency Network Reset in TLC LINK if filters remain."
    }
    "status" {
        $existing = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
        if (-not $existing) {
            Write-Host "Service: not installed (UI can still arm KS in-process when elevated)."
        } else {
            Write-Host "Service: $($existing.Status) ($ServiceName)"
        }
        $exe = Find-ServiceExe
        Write-Host "Binary: $(if ($exe) { $exe } else { 'not found' })"
    }
}
