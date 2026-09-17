param(
    [string]$Url = "https://www.teenix.org/HP67.zip",
    [string]$ExpectedCal67Sha256 = "C8563E7982F38DEE9B6004B1194727AA2942DEB72D327C4D2F008CFDD057804B",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$researchDir = Join-Path $repoRoot ".research"
$zipPath = Join-Path $researchDir "teenix-hp67.zip"
$extractDir = Join-Path $researchDir "teenix-hp67-module"
$cal67Path = Join-Path $researchDir "cal67.pfl"
$tsvPath = Join-Path $researchDir "teenix-2026-hp67.tsv"

New-Item -ItemType Directory -Force -Path $researchDir | Out-Null

if ($Force -or -not (Test-Path $zipPath)) {
    Write-Host "Downloading Teenix HP-67 module from $Url"
    Invoke-WebRequest -Uri $Url -OutFile $zipPath -UseBasicParsing
}

if (Test-Path $extractDir) {
    Remove-Item -Recurse -Force $extractDir
}
New-Item -ItemType Directory -Force -Path $extractDir | Out-Null
Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force

$cal67Candidates = @(Get-ChildItem -Path $extractDir -Filter "cal67.pfl" -File -Recurse)
if ($cal67Candidates.Count -ne 1) {
    throw "Expected exactly one cal67.pfl in $zipPath, found $($cal67Candidates.Count)."
}

$actualSha = (Get-FileHash -Algorithm SHA256 -Path $cal67Candidates[0].FullName).Hash.ToUpperInvariant()
if ($actualSha -ne $ExpectedCal67Sha256.ToUpperInvariant()) {
    throw "Teenix cal67.pfl SHA-256 mismatch. Expected $ExpectedCal67Sha256, got $actualSha. Upstream may have changed; review before accepting a new corpus."
}

Copy-Item -Force $cal67Candidates[0].FullName $cal67Path

Push-Location $repoRoot
try {
    & cargo run --locked --bin rom_compare -- analyze-teenix-hp67 $cal67Path
    if ($LASTEXITCODE -ne 0) { throw "Teenix HP-67 analysis failed." }

    & cargo run --locked --bin rom_compare -- extract-teenix-hp67 $cal67Path $tsvPath
    if ($LASTEXITCODE -ne 0) { throw "Teenix HP-67 corpus extraction failed." }

    & cargo run --locked --bin rom_compare -- verify-startup $tsvPath
    if ($LASTEXITCODE -ne 0) { throw "Teenix HP-67 startup verification failed." }
}
finally {
    Pop-Location
}

Write-Host "READY: $tsvPath"
Write-Host "cal67.pfl SHA-256: $actualSha"
