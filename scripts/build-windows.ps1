$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$sing = Join-Path $root 'src-tauri\binaries\sing-box\sing-box.exe'
$xray = Join-Path $root 'src-tauri\binaries\xray\xray.exe'
if (!(Test-Path $sing)) { throw "Missing official sing-box binary: $sing" }
if (!(Test-Path $xray)) { throw "Missing official Xray binary: $xray" }

& (Join-Path $root 'scripts\fetch-vendor.ps1')
& (Join-Path $root 'scripts\verify-cores.ps1')

# Gate: the build stops if the unit tests or offline frontend checks fail.
Push-Location $root
try {
  node tests/frontend-checks.mjs
  if ($LASTEXITCODE -ne 0) { throw 'Frontend checks failed.' }
  cargo test --manifest-path src-tauri/Cargo.toml
  if ($LASTEXITCODE -ne 0) { throw 'cargo test failed.' }
} finally { Pop-Location }

$configPath = Join-Path $root 'src-tauri\tauri.conf.json'
$originalConfig = Get-Content $configPath -Raw
try {
  $config = $originalConfig | ConvertFrom-Json
  $config.bundle | Add-Member -NotePropertyName resources -NotePropertyValue @(
    'binaries/sing-box/sing-box.exe',
    'binaries/xray/xray.exe'
  ) -Force
  $json = $config | ConvertTo-Json -Depth 30
  [System.IO.File]::WriteAllText($configPath, $json, [System.Text.UTF8Encoding]::new($false))

  Push-Location $root
  try {
    npm install
    npm run build
  } finally { Pop-Location }
} finally {
  # Build-time resource injection is temporary. Restore the checked-in config
  # even if npm or Tauri fails, so the working tree is not left modified.
  [System.IO.File]::WriteAllText($configPath, $originalConfig, [System.Text.UTF8Encoding]::new($false))
}

# Checksums are generated only after the Portable ZIP exists.
