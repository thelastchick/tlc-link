# Verifies the core executables against src-tauri/binaries/cores.lock.json.
# FIRST USE: download sing-box and Xray ONLY from their official GitHub releases, compare the file
# hash with the checksum/digest published by the project, then run with -Pin to record version + SHA-256.
param([switch]$Pin, [string]$SingBoxVersion, [string]$XrayVersion)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$lockPath = Join-Path $root 'src-tauri\binaries\cores.lock.json'
$cores = @{
  'sing-box' = Join-Path $root 'src-tauri\binaries\sing-box\sing-box.exe'
  'xray'     = Join-Path $root 'src-tauri\binaries\xray\xray.exe'
}
foreach ($k in $cores.Keys) { if (!(Test-Path $cores[$k])) { throw "Missing core executable: $($cores[$k])" } }
if ($Pin) {
  if (!$SingBoxVersion -or !$XrayVersion) { throw 'With -Pin you must pass -SingBoxVersion and -XrayVersion (the exact upstream release versions).' }
  $lock = @{
    'sing-box' = @{ version = $SingBoxVersion; sha256 = (Get-FileHash $cores['sing-box'] -Algorithm SHA256).Hash }
    'xray'     = @{ version = $XrayVersion;    sha256 = (Get-FileHash $cores['xray'] -Algorithm SHA256).Hash }
  }
  $lock | ConvertTo-Json | Set-Content $lockPath -Encoding UTF8
  Write-Host "Pinned core hashes written to $lockPath. Commit it and record versions in THIRD-PARTY-NOTICES.md."
  exit 0
}
if (!(Test-Path $lockPath)) { throw 'cores.lock.json is missing. Run: scripts\verify-cores.ps1 -Pin -SingBoxVersion <v> -XrayVersion <v> after verifying official checksums.' }
$lock = Get-Content $lockPath -Raw | ConvertFrom-Json
foreach ($k in $cores.Keys) {
  $actual = (Get-FileHash $cores[$k] -Algorithm SHA256).Hash
  if ($lock.$k.sha256 -ne $actual) { throw "Hash mismatch for $k (expected $($lock.$k.sha256), got $actual). Refusing to package an unknown or modified executable." }
  Write-Host "$k $($lock.$k.version) OK"
}
