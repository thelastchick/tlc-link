# Downloads the two QR libraries ONCE into frontend/vendor so the app never loads scripts from a CDN.
# First run records SHA-256 hashes in frontend/vendor/vendor-hashes.json; later runs refuse changed files.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$dir = Join-Path $root 'frontend\vendor'
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$files = @{
  'qrcode.min.js' = 'https://cdn.jsdelivr.net/npm/qrcode@1.5.1/build/qrcode.min.js'
  'jsQR.js'       = 'https://cdn.jsdelivr.net/npm/jsqr@1.4.0/dist/jsQR.js'
}
$hashFile = Join-Path $dir 'vendor-hashes.json'
$pinned = if (Test-Path $hashFile) { Get-Content $hashFile -Raw | ConvertFrom-Json } else { $null }
$result = @{}
foreach ($name in $files.Keys) {
  $target = Join-Path $dir $name
  if (!(Test-Path $target)) { Invoke-WebRequest -Uri $files[$name] -OutFile $target -UseBasicParsing }
  $hash = (Get-FileHash $target -Algorithm SHA256).Hash
  if ($pinned -and $pinned.$name -and $pinned.$name -ne $hash) { throw "Hash mismatch for $name (expected $($pinned.$name), got $hash). Delete the file only if you intend to update it." }
  $result[$name] = $hash
}
if (!$pinned) { $result | ConvertTo-Json | Set-Content $hashFile -Encoding UTF8; Write-Host 'Pinned SHA-256 hashes written to vendor-hashes.json' }
Write-Host 'QR libraries are available locally.'
