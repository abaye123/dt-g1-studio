# Package a portable Windows build as a .zip. The portable app keeps its data in
# a `data\` folder next to the executable (triggered by the `portable.txt`
# marker), so it leaves nothing in the user profile and can run from a USB stick.
#
# Needs the release binary (cargo build --release -p dt-g1-studio-app) and the
# bundled resources (scripts/vendor.ps1). Usage: pwsh scripts/portable.ps1
param([string]$Config = "release")
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$ver  = (Get-Content (Join-Path $root "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json).version
$exe  = Join-Path $root "target\$Config\dt-g1-studio-app.exe"
$res  = Join-Path $root "src-tauri\resources"
if (-not (Test-Path $exe)) { throw "missing $exe - run: cargo build --$Config -p dt-g1-studio-app" }
if (-not (Test-Path $res)) { throw "missing resources - run: scripts\vendor.ps1" }

$stage = Join-Path $env:TEMP ("dtg1_portable_" + [guid]::NewGuid().ToString("N"))
$app   = Join-Path $stage "DT-G1-Studio"
New-Item -ItemType Directory -Force -Path $app | Out-Null

Copy-Item $exe (Join-Path $app "DT G1 Studio.exe")
Copy-Item -Recurse $res (Join-Path $app "resources")
Set-Content -Encoding utf8 -Path (Join-Path $app "portable.txt") `
  -Value "Portable mode: this program stores its data in the data\ folder next to this file."
# A readme so the zip is self-explanatory.
Set-Content -Encoding utf8 -Path (Join-Path $app "README.txt") `
  -Value "DT G1 Studio (portable)`r`nRun 'DT G1 Studio.exe'. All data is kept in the data\ folder beside it.`r`nRequires Microsoft Edge WebView2 (preinstalled on Windows 10/11)."

$out = Join-Path $root ("DT-G1-Studio-$ver-portable-win-x64.zip")
if (Test-Path $out) { Remove-Item -Force $out }
# Zip the folder itself so it extracts into a DT-G1-Studio\ directory.
Compress-Archive -Path $app -DestinationPath $out
Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
Write-Host "portable -> $out" -ForegroundColor Green
