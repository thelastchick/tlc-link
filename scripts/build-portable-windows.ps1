$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$releaseDir = Join-Path $root 'src-tauri\target\release'
$candidates = @(
    (Join-Path $releaseDir 'TLC LINK.exe'),
    (Join-Path $releaseDir 'tlc-link.exe'),
    (Join-Path $releaseDir 'TLC-LINK.exe'),
    (Join-Path $releaseDir 'tlc_link.exe')
)

$exe = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (!$exe) {
    throw "Release executable not found. Build the app first; checked: $($candidates -join ', ')"
}

$sing = Join-Path $root 'src-tauri\binaries\sing-box\sing-box.exe'
$xray = Join-Path $root 'src-tauri\binaries\xray\xray.exe'
if (!(Test-Path $sing) -or !(Test-Path $xray)) {
    throw 'Both official core executables are required for the portable package.'
}

$distDir = Join-Path $root 'dist'
$out = Join-Path $distDir 'TLC-LINK-Portable'
$zipPath = Join-Path $distDir 'TLC-LINK-Portable.zip'

New-Item -ItemType Directory -Force -Path $distDir | Out-Null
if (Test-Path $out) { Remove-Item $out -Recurse -Force }
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }

# The application log shows that TLC LINK looks for cores beside the portable
# executable in: TLC-LINK-Portable\binaries\<core>\<core>.exe
New-Item -ItemType Directory -Force -Path (Join-Path $out 'binaries\sing-box') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $out 'binaries\xray') | Out-Null

Copy-Item $exe (Join-Path $out 'TLC-LINK.exe') -Force
Copy-Item $sing (Join-Path $out 'binaries\sing-box\sing-box.exe') -Force
Copy-Item $xray (Join-Path $out 'binaries\xray\xray.exe') -Force

@'
TLC LINK Portable
Run TLC-LINK.exe. WebView2 Runtime is required on Windows if it is not already installed.

Included cores:
- binaries\sing-box\sing-box.exe
- binaries\xray\xray.exe
'@ | Set-Content (Join-Path $out 'README.txt') -Encoding UTF8

# Ship honest docs with the portable folder
$known = Join-Path $root 'docs\KNOWN-LIMITATIONS.md'
if (Test-Path $known) {
  Copy-Item $known (Join-Path $out 'KNOWN-LIMITATIONS.txt') -Force
}
$clog = Join-Path $root 'CHANGELOG-USER.md'
if (Test-Path $clog) {
  Copy-Item $clog (Join-Path $out 'CHANGELOG-USER.txt') -Force
}
$rcprep = Join-Path $root 'docs\RC-PREPARATION.md'
if (Test-Path $rcprep) {
  Copy-Item $rcprep (Join-Path $out 'RC-PREPARATION.txt') -Force
}


Compress-Archive -Path $out -DestinationPath $zipPath -Force

# Verify the archive contains the exact paths expected by the application.
$archive = [System.IO.Compression.ZipFile]::OpenRead($zipPath)
try {
    $entries = $archive.Entries.FullName
    $required = @(
        'TLC-LINK-Portable/binaries/sing-box/sing-box.exe',
        'TLC-LINK-Portable/binaries/xray/xray.exe',
        'TLC-LINK-Portable/TLC-LINK.exe'
    )
    foreach ($item in $required) {
        if ($entries -notcontains $item) {
            throw "Portable ZIP verification failed; missing entry: $item"
        }
    }
}
finally {
    $archive.Dispose()
}

Write-Host "Portable package created and verified: $zipPath"

# Generate SHA-256 only after the verified Portable ZIP has been created.
& (Join-Path $root 'scripts\make-checksums.ps1')
