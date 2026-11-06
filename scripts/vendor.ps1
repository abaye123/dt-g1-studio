# Assemble the bundled tool resources into src-tauri/resources (gitignored).
# These ship inside the app so it works offline; at runtime newer versions of
# the map compiler and launcher are fetched when available, else these are used.
#
# Usage:  pwsh scripts/vendor.ps1 [-Platform win32-x64] [-UseLocalGx]
#   -UseLocalGx  copy the Glyphix CLI from the local toolchain instead of npm
#                (faster on the dev machine; CI uses npm).
param(
  [string]$Platform = "win32-x64",
  [switch]$UseLocalGx
)
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot          # repo root (dt-g1-studio)
$res  = Join-Path $root "src-tauri\resources"
$tmp  = Join-Path $env:TEMP ("dtg1_vendor_" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $res, $tmp | Out-Null

$GX_VERSION = "0.8.0-rc"
# Emulator-only payload we never use (keeps the bundle ~6 MB smaller). The build
# itself uses gx.exe / image-forge.exe / glyphix-jsc.exe / pngquant.exe only.
$GX_DROP = @(
  "glyphix-emu.exe","SDL2.dll","avcodec-61.dll","avformat-61.dll","avutil-59.dll",
  "opus.dll","libmp3lame.DLL","simpleble-c.dll","simpleble.dll",
  "swresample-5.dll","swscale-8.dll"
)

function Say($m) { Write-Host "[vendor] $m" -ForegroundColor Cyan }

# --- gx toolchain -----------------------------------------------------------
Say "assembling gx ($Platform)"
$gxDst = Join-Path $res "gx"
if (Test-Path $gxDst) { Remove-Item -Recurse -Force $gxDst }
New-Item -ItemType Directory -Force -Path $gxDst | Out-Null

$localCli = "C:\dev\dt-g1-watch\dt-g1-research\tools\glyphix-toolchain\node_modules\@glyphix\cli-win32-x64"
if ($UseLocalGx -and $Platform -eq "win32-x64" -and (Test-Path $localCli)) {
  Say "copying gx from local toolchain"
  Copy-Item -Recurse -Force (Join-Path $localCli "bin")   (Join-Path $gxDst "bin")
  Copy-Item -Recurse -Force (Join-Path $localCli "share") (Join-Path $gxDst "share")
} else {
  Say "fetching @glyphix/cli-$Platform@$GX_VERSION via npm pack"
  Push-Location $tmp
  npm pack "@glyphix/cli-$Platform@$GX_VERSION" --silent | Out-Null
  $tgz = Get-ChildItem $tmp -Filter "*.tgz" | Select-Object -First 1
  tar -xzf $tgz.FullName -C $tmp
  # npm tarball extracts to "package/"
  Copy-Item -Recurse -Force (Join-Path $tmp "package\bin")   (Join-Path $gxDst "bin")
  Copy-Item -Recurse -Force (Join-Path $tmp "package\share") (Join-Path $gxDst "share")
  Pop-Location
}
# Trim emulator payload.
foreach ($f in $GX_DROP) {
  $p = Join-Path $gxDst "bin\$f"
  if (Test-Path $p) { Remove-Item -Force $p }
}
# Embed the device profile so gx needs no %APPDATA% setup (gx searches
# share/glyphix/devices next to the binary). Source is the repo's own copy.
$devDst = Join-Path $gxDst "share\glyphix\devices"
New-Item -ItemType Directory -Force -Path $devDst | Out-Null
$devSrc = Join-Path $root "src-tauri\device"
foreach ($f in @("dtg1-c175r.yml", "image-convert-pal.scm")) {
  $src = Join-Path $devSrc $f
  if (Test-Path $src) { Copy-Item -Force $src $devDst } else { Say "WARN: missing $f" }
}

# --- map compiler (Windows only) -------------------------------------------
$mapVer = ""
if ($Platform -eq "win32-x64") {
  Say "downloading map compiler (dtg1-map-tools)"
  $rel = Invoke-RestMethod "https://api.github.com/repos/purrrock/dtg1-map-tools/releases/latest" -Headers @{ "User-Agent" = "dt-g1-studio" }
  $mapVer = $rel.tag_name
  $asset = $rel.assets | Where-Object { $_.name -like "*windows*.zip" } | Select-Object -First 1
  if (-not $asset) { throw "no windows map-compiler asset" }
  $zip = Join-Path $tmp "mapc.zip"
  Invoke-WebRequest $asset.browser_download_url -OutFile $zip -Headers @{ "User-Agent" = "dt-g1-studio" }
  $mx = Join-Path $tmp "mapc"; New-Item -ItemType Directory -Force -Path $mx | Out-Null
  Expand-Archive -Force $zip $mx
  $mapDst = Join-Path $res "map-compiler"
  if (Test-Path $mapDst) { Remove-Item -Recurse -Force $mapDst }
  New-Item -ItemType Directory -Force -Path $mapDst | Out-Null
  Get-ChildItem -Recurse $mx | Where-Object {
    $_.Extension -in ".exe", ".dll" -or $_.Name -eq "features.csv"
  } | ForEach-Object { Copy-Item -Force $_.FullName $mapDst }
} else {
  Say "skipping map compiler (no $Platform build upstream)"
}

# --- launcher (PurrLaunch, platform-independent) ----------------------------
Say "downloading launcher (PurrLaunch)"
$lrel = Invoke-RestMethod "https://api.github.com/repos/purrrock/PurrLaunch/releases/latest" -Headers @{ "User-Agent" = "dt-g1-studio" }
$launVer = $lrel.tag_name
$lasset = $lrel.assets | Where-Object { $_.name -like "*.pkg" } | Select-Object -First 1
if (-not $lasset) { throw "no launcher .pkg asset" }
$launDst = Join-Path $res "launcher"
if (Test-Path $launDst) { Remove-Item -Recurse -Force $launDst }
New-Item -ItemType Directory -Force -Path $launDst | Out-Null
Invoke-WebRequest $lasset.browser_download_url -OutFile (Join-Path $launDst "com.xfaith.woodenfish.pkg") -Headers @{ "User-Agent" = "dt-g1-studio" }

# --- manifest ---------------------------------------------------------------
$manifest = @{ gx = $GX_VERSION; "map-compiler" = $mapVer; launcher = $launVer; platform = $Platform }
$manifest | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $res "manifest.json")

Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
Say "done -> $res"
Get-ChildItem $res | Format-Table Name, Mode
