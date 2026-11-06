# Sign the Windows installer of a drafted release and publish it.
#
# The CI (release.yml) builds installers and creates a DRAFT prerelease, but the
# GitHub runner has no access to the local code-signing certificate. This script
# runs on the dev machine: it downloads the draft's Windows installer, signs it
# with the "abaye ai" certificate, re-uploads it, and publishes the release.
#
# Usage:  pwsh scripts/sign-and-publish.ps1 -Tag v0.2.0 [-Publish] [-PreRelease]
#   -Publish     also flip the release out of draft (otherwise sign + upload only)
#   -PreRelease  keep it marked as a prerelease when publishing
param(
  [Parameter(Mandatory = $true)][string]$Tag,
  [switch]$Publish,
  [switch]$PreRelease
)
$ErrorActionPreference = "Stop"

$THUMB = "EBAE118B2C6434273890DE749EC457C34FE1A882"
$TS = "http://timestamp.digicert.com"

function Find-SignTool {
  $pref = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"
  if (Test-Path $pref) { return $pref }
  $cand = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin" -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match "x64" } | Sort-Object FullName -Descending | Select-Object -First 1
  if ($cand) { return $cand.FullName }
  throw "signtool.exe not found - install the Windows SDK"
}

$signtool = Find-SignTool
$tmp = Join-Path $env:TEMP ("dtg1_sign_" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

Write-Host "[sign] downloading Windows installer from draft $Tag" -ForegroundColor Cyan
# NSIS setup installer produced by Tauri.
gh release download $Tag --pattern "*-setup.exe" --dir $tmp --clobber
$installers = Get-ChildItem $tmp -Filter "*-setup.exe"
if (-not $installers) { throw "no *-setup.exe asset found on release $Tag" }

foreach ($f in $installers) {
  Write-Host "[sign] signing $($f.Name)" -ForegroundColor Cyan
  & $signtool sign /sha1 $THUMB /fd sha256 /tr $TS /td sha256 $f.FullName
  if ($LASTEXITCODE -ne 0) { throw "signtool failed on $($f.Name)" }
  # Confirm a signature + timestamp are present (UnknownError is expected for a
  # self-signed cert; what matters is the signer thumbprint and a timestamp).
  $sig = Get-AuthenticodeSignature $f.FullName
  Write-Host "[sign]   signer: $($sig.SignerCertificate.Thumbprint)  status: $($sig.Status)"
  if ($sig.SignerCertificate.Thumbprint -ne $THUMB) { throw "unexpected signer on $($f.Name)" }
  Write-Host "[sign] uploading signed $($f.Name)" -ForegroundColor Cyan
  gh release upload $Tag $f.FullName --clobber
}

if ($Publish) {
  $pre = if ($PreRelease) { "true" } else { "false" }
  Write-Host "[sign] publishing release $Tag (prerelease=$pre)" -ForegroundColor Green
  gh release edit $Tag --draft=false --prerelease=$pre
} else {
  Write-Host "[sign] signed and uploaded. Re-run with -Publish to take it out of draft." -ForegroundColor Yellow
}

Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
