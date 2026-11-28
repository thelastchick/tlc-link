$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$portableZip = Join-Path $root 'dist\TLC-LINK-Portable.zip'

if (!(Test-Path -LiteralPath $portableZip -PathType Leaf)) {
  throw "Portable artifact not found; checksum generation must run after build-portable-windows.ps1: $portableZip"
}

$outDir = Join-Path $root 'dist'
$out = Join-Path $outDir 'SHA256SUMS.txt'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$hash = (Get-FileHash -LiteralPath $portableZip -Algorithm SHA256).Hash
"{0}  {1}" -f $hash, (Split-Path $portableZip -Leaf) | Set-Content -LiteralPath $out -Encoding ASCII
Write-Host "Wrote $out"
