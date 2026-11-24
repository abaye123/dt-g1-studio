# Package a portable Windows build as a .zip. The portable app keeps its data in
# a `data\` folder next to the executable (triggered by the `portable.txt`
# marker), so it leaves nothing in the user profile and can run from a USB stick.
#
# Needs the release binary (cargo build --release -p dt-g1-studio-app) and the
# bundled resources (scripts/vendor.ps1). Usage: pwsh scripts/portable.ps1
# -Lite packages the lite edition. Build it first with TAURI_CONFIG set to the text of
# src-tauri/tauri.lite.conf.json: cargo build --release --features lite -p dt-g1-studio-app --target-dir target/lite
param([string]$Config = "release", [switch]$Lite)
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$ver  = (Get-Content (Join-Path $root "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json).version
$name = if ($Lite) { "DT G1 Studio Lite" } else { "DT G1 Studio" }
$slug = $name.Replace(" ", "-")
$targetDir = if ($Lite) { "target\lite" } else { "target" }
$exe  = Join-Path $root "$targetDir\$Config\dt-g1-studio-app.exe"
$res  = Join-Path $root "src-tauri\resources"
if (-not (Test-Path $exe)) { throw "missing $exe - run: cargo build --$Config -p dt-g1-studio-app" }
if (-not (Test-Path $res)) { throw "missing resources - run: scripts\vendor.ps1" }

$stage = Join-Path $env:TEMP ("dtg1_portable_" + [guid]::NewGuid().ToString("N"))
$app   = Join-Path $stage $slug
New-Item -ItemType Directory -Force -Path $app | Out-Null

Copy-Item $exe (Join-Path $app "$name.exe")
Copy-Item -Recurse $res (Join-Path $app "resources")
# The lite edition has no app builder.
if ($Lite) { Remove-Item -Recurse -Force (Join-Path $app "resources\gx"), (Join-Path $app "resources\launcher") }
Set-Content -Encoding utf8 -Path (Join-Path $app "portable.txt") `
  -Value "Portable mode: this program stores its data in the data\ folder next to this file."
# A readme so the zip is self-explanatory.
Set-Content -Encoding utf8 -Path (Join-Path $app "README.txt") `
  -Value "$name (portable)`r`nRun '$name.exe'. All data is kept in the data\ folder beside it.`r`nRequires Microsoft Edge WebView2 (preinstalled on Windows 10/11)."

$out = Join-Path $root ("$slug-$ver-portable-win-x64.zip")
if (Test-Path $out) { Remove-Item -Force $out }
# Zip the folder itself so it extracts into a DT-G1-Studio\ directory.
Compress-Archive -Path $app -DestinationPath $out
Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
Write-Host "portable -> $out" -ForegroundColor Green
