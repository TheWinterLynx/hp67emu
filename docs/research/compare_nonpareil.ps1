param(
    [string]$Uasm = "",
    [string]$TeenixPfl = ".\docs\research\downloaded\teenix-HP67\cal67.pfl",
    [string]$ResearchDir = ".research"
)

$ErrorActionPreference = "Stop"
$NonpareilCommit = "c347bc1ab20170c253512042f7aac0d952f304ea"
$SourceNames = @("67", "6797", "67b1")

New-Item -ItemType Directory -Force $ResearchDir | Out-Null
$ExternalDir = Join-Path $ResearchDir "external"
New-Item -ItemType Directory -Force $ExternalDir | Out-Null
$NonpareilDir = Join-Path $ExternalDir "nonpareil-$NonpareilCommit"
New-Item -ItemType Directory -Force $NonpareilDir | Out-Null

if ([string]::IsNullOrWhiteSpace($Uasm)) {
    $Found = Get-Command uasm -ErrorAction SilentlyContinue
    if ($Found) {
        $Uasm = $Found.Source
    }
}
if ([string]::IsNullOrWhiteSpace($Uasm) -or !(Test-Path $Uasm -PathType Leaf)) {
    Write-Host "Nonpareil uasm was not found."
    Write-Host "Build the official Nonpareil uasm locally, then rerun with -Uasm <path-to-uasm-or-uasm.exe>."
    Write-Host "Pinned Nonpareil commit: $NonpareilCommit"
    exit 3
}
$Uasm = (Resolve-Path $Uasm).Path

$Objects = @()
$HashLines = @("Nonpareil commit=$NonpareilCommit", "uasm=$Uasm")
foreach ($Name in $SourceNames) {
    $Url = "https://raw.githubusercontent.com/brouhaha/nonpareil/$NonpareilCommit/ncd/67-97/$Name.asm"
    $Source = Join-Path $NonpareilDir "$Name.asm"
    if (!(Test-Path $Source)) {
        Write-Host "Downloading Nonpareil $Name.asm at $NonpareilCommit ..."
        Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $Source
    }
    $SourceHash = (Get-FileHash $Source -Algorithm SHA256).Hash
    $HashLines += "Nonpareil $Name.asm SHA256=$SourceHash"
    $HashLines += "Nonpareil $Name.asm URL=$Url"

    $Object = Join-Path $NonpareilDir "$Name.obj"
    $Listing = Join-Path $NonpareilDir "$Name.lst"
    Write-Host "Assembling $Name.asm with official uasm ..."
    & $Uasm -o $Object -l $Listing $Source
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    if (!(Test-Path $Object)) { throw "uasm did not produce $Object" }
    $ObjectHash = (Get-FileHash $Object -Algorithm SHA256).Hash
    $HashLines += "Nonpareil $Name.obj SHA256=$ObjectHash"
    $Objects += $Object
}

$HashReport = Join-Path $ResearchDir "nonpareil-hp67-hashes.txt"
$HashLines | Set-Content -Encoding UTF8 $HashReport
Write-Host "`n=== NONPAREIL PROVENANCE ==="
Get-Content $HashReport

$NonpareilTsv = Join-Path $ResearchDir "nonpareil-hp67-$NonpareilCommit.tsv"
Write-Host "`n=== NORMALIZE OFFICIAL UASM OBJECTS ==="
& cargo run --quiet --bin nonpareil_rom -- $NonpareilTsv @Objects
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n=== NONPAREIL PHYSICAL STARTUP CHECK ==="
& cargo run --quiet --bin rom_compare -- verify-startup $NonpareilTsv
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (!(Test-Path $TeenixPfl)) {
    throw "Teenix HP-67 module not found: $TeenixPfl"
}
$TeenixTsv = Join-Path $ResearchDir "teenix-2026-hp67.tsv"
if (!(Test-Path $TeenixTsv)) {
    Write-Host "`n=== EXTRACT TEENIX 2026 ==="
    & cargo run --quiet --bin rom_compare -- extract-teenix-hp67 $TeenixPfl $TeenixTsv
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "`n=== TEENIX 5120-WORD SUBSET VS NONPAREIL ==="
& cargo run --quiet --bin rom_subset -- $TeenixTsv $NonpareilTsv
$TeenixResult = $LASTEXITCODE

$X11Tsv = Get-ChildItem $ResearchDir -File -Filter "x11-hp67-*.tsv" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($X11Tsv) {
    Write-Host "`n=== NONPAREIL SUBSET VS X11-CALC ==="
    & cargo run --quiet --bin rom_subset -- $NonpareilTsv $X11Tsv.FullName
    $X11Result = $LASTEXITCODE
} else {
    Write-Host "`nNo x11-calc TSV found in $ResearchDir; skipping Nonpareil vs x11-calc."
    $X11Result = 0
}

if ($TeenixResult -ne 0) { exit $TeenixResult }
exit $X11Result
