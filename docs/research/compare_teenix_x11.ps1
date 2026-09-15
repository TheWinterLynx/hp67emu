param(
    [string]$TeenixPfl = ".\docs\research\downloaded\teenix-HP67\cal67.pfl",
    [string]$ResearchDir = ".research"
)

$ErrorActionPreference = "Stop"
$X11Commit = "9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983"
$X11Url = "https://raw.githubusercontent.com/mike632t/x11-calc/$X11Commit/src/x11-calc-67.c"

New-Item -ItemType Directory -Force $ResearchDir | Out-Null
$ExternalDir = Join-Path $ResearchDir "external"
New-Item -ItemType Directory -Force $ExternalDir | Out-Null

$X11Source = Join-Path $ExternalDir "x11-calc-67-$X11Commit.c"
$TeenixTsv = Join-Path $ResearchDir "teenix-2026-hp67.tsv"
$X11Tsv = Join-Path $ResearchDir "x11-hp67-$X11Commit.tsv"
$HashReport = Join-Path $ResearchDir "hp67-rom-corpus-hashes.txt"

if (!(Test-Path $TeenixPfl)) {
    throw "Teenix HP-67 module not found: $TeenixPfl"
}

if (!(Test-Path $X11Source)) {
    Write-Host "Downloading pinned x11-calc HP-67 source at $X11Commit ..."
    Invoke-WebRequest -UseBasicParsing -Uri $X11Url -OutFile $X11Source
}

$TeenixHash = (Get-FileHash $TeenixPfl -Algorithm SHA256).Hash
$X11Hash = (Get-FileHash $X11Source -Algorithm SHA256).Hash
@(
    "Teenix cal67.pfl SHA256=$TeenixHash",
    "x11-calc commit=$X11Commit",
    "x11-calc source SHA256=$X11Hash",
    "x11-calc source URL=$X11Url"
) | Set-Content -Encoding UTF8 $HashReport

Write-Host "=== SOURCE PROVENANCE ==="
Get-Content $HashReport

Write-Host "`n=== EXTRACT TEENIX 2026 ==="
& cargo run --quiet --bin rom_compare -- extract-teenix-hp67 $TeenixPfl $TeenixTsv
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n=== EXTRACT X11-CALC ==="
& cargo run --quiet --bin rom_compare -- extract-x11 $X11Source $X11Tsv
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n=== X11-CALC PHYSICAL STARTUP CHECK ==="
& cargo run --quiet --bin rom_compare -- verify-startup $X11Tsv
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n=== TEENIX 5120-WORD SUBSET VS X11-CALC ==="
& cargo run --quiet --bin rom_subset -- $TeenixTsv $X11Tsv
exit $LASTEXITCODE
