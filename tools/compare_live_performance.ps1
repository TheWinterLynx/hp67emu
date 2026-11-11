[CmdletBinding()]
param(
    [string]$Baseline = '7544c45fd86149fc1cc1b9f75c9a597b61466135',
    [string]$Candidate = 'HEAD'
)

$ErrorActionPreference = 'Stop'
$Invariant = [Globalization.CultureInfo]::InvariantCulture
$PreviousRustFlags = $env:RUSTFLAGS
$PreviousTargetDir = $env:CARGO_TARGET_DIR
$TranscriptStarted = $false
$Worktrees = @{}
$Root = Split-Path $PSScriptRoot -Parent
$Scratch = Join-Path ([IO.Path]::GetTempPath()) ('hp67-perf-' + [guid]::NewGuid().ToString('N'))
$Benchmark = 'm14o_live_authority_realtime_benchmark'
$Workloads = [ordered]@{
    Idle = 'live firmware idle / full authority'
    Ram = 'synthetic RAM ports / reference bridge'
    Crc = 'synthetic CRC ports / reference bridge'
}

function Resolve-Revision([string]$Revision) {
    $Commit = & git rev-parse --verify --end-of-options "$Revision^{commit}"
    if ($LASTEXITCODE -ne 0 -or $Commit -notmatch '^[0-9a-f]{40}$') {
        throw "CANNOT RESOLVE BENCHMARK REVISION: $Revision"
    }
    return $Commit
}

Push-Location $Root
try {
    $Commits = @{ A = (Resolve-Revision $Baseline); B = (Resolve-Revision $Candidate) }
    if ($Commits.A -eq $Commits.B) { throw 'Select two distinct commits.' }
    $Active = & rustup show active-toolchain
    if ($LASTEXITCODE -ne 0 -or -not $Active) { throw 'RUSTUP TOOLCHAIN UNAVAILABLE' }
    $Toolchain = ($Active -split '\s+')[0]
    $CargoToolchain = "+$Toolchain"
    $LogDirectory = Join-Path $Root 'target/performance-comparison'
    New-Item -ItemType Directory -Force -Path $LogDirectory, $Scratch | Out-Null
    $LogPath = Join-Path $LogDirectory ("{0}-{1}-{2}.txt" -f $Commits.A.Substring(0, 7), $Commits.B.Substring(0, 7), (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    Start-Transcript -Path $LogPath | Out-Null
    $TranscriptStarted = $true
    Write-Host "Baseline A: $($Commits.A)"
    Write-Host "Candidate B: $($Commits.B)"
    & rustc $CargoToolchain -vV
    if ($LASTEXITCODE -ne 0) { throw 'RUSTC VERSION KO' }
    & cargo $CargoToolchain --version
    if ($LASTEXITCODE -ne 0) { throw 'CARGO VERSION KO' }
    $env:RUSTFLAGS = '-Dwarnings'

    foreach ($Label in @('A', 'B')) {
        $Path = Join-Path $Scratch $Label
        & git worktree add --detach $Path $Commits[$Label]
        if ($LASTEXITCODE -ne 0) { throw "WORKTREE KO: $Label" }
        $Worktrees[$Label] = $Path
        Push-Location $Path
        try {
            $env:CARGO_TARGET_DIR = Join-Path $Path 'target'
            Write-Host "Building $Label before measurement"
            & cargo $CargoToolchain test --release --locked --bin hp67emu --no-run
            if ($LASTEXITCODE -ne 0) { throw "BENCHMARK BUILD KO: $Label" }
        } finally {
            Pop-Location
        }
    }

    $Samples = @{ A = @(); B = @() }
    $Run = 0
    foreach ($Label in @('A', 'B', 'B', 'A')) {
        $Run += 1
        Write-Host "[$Run/4] Measuring $Label"
        Push-Location $Worktrees[$Label]
        try {
            $env:CARGO_TARGET_DIR = Join-Path $Worktrees[$Label] 'target'
            $Output = & cargo $CargoToolchain test --release --locked --bin hp67emu $Benchmark -- --ignored --nocapture
            $BenchExit = $LASTEXITCODE
            $Output | ForEach-Object { Write-Host $_ }
            if ($BenchExit -ne 0) { throw "BENCHMARK RUN KO: $Label" }
            $Sample = @{}
            foreach ($Key in $Workloads.Keys) {
                $Pattern = '^' + [regex]::Escape($Workloads[$Key]) + '\s+([0-9]+\.[0-9]+)\s+'
                $Rows = @($Output | Where-Object { $_ -match $Pattern })
                if ($Rows.Count -ne 1) { throw "BENCHMARK ROW MISSING OR AMBIGUOUS: $Label/$Key" }
                $Value = [double]::Parse([regex]::Match($Rows[0], $Pattern).Groups[1].Value, $Invariant)
                if ($Value -le 0) { throw "INVALID BENCHMARK MEDIAN: $Label/$Key" }
                $Sample[$Key] = $Value
            }
            $Samples[$Label] += $Sample
        } finally {
            Pop-Location
        }
    }

    Write-Host 'Workload | A runs (us/word) | B runs (us/word) | B vs A time'
    foreach ($Key in $Workloads.Keys) {
        $A1 = $Samples.A[0][$Key]
        $A2 = $Samples.A[1][$Key]
        $B1 = $Samples.B[0][$Key]
        $B2 = $Samples.B[1][$Key]
        $Delta = 100 * (($B1 + $B2) / ($A1 + $A2) - 1)
        Write-Host ([string]::Format($Invariant, '{0} | {1:F3}, {2:F3} | {3:F3}, {4:F3} | {5:+0.00;-0.00;0.00}%', [object[]]@($Key, $A1, $A2, $B1, $B2, $Delta)))
    }
    Write-Host 'PERFORMANCE COMPARISON COMPLETE (positive delta means slower; no pass threshold)'
    Write-Host "Evidence: $LogPath"
} finally {
    try {
        foreach ($Path in $Worktrees.Values) {
            & git worktree remove $Path
            if ($LASTEXITCODE -ne 0) { Write-Warning -WarningAction Continue "Worktree retained: $Path" }
        }
        if ((Test-Path -LiteralPath $Scratch) -and -not (Get-ChildItem -LiteralPath $Scratch -Force)) {
            Remove-Item -LiteralPath $Scratch -ErrorAction Continue
        }
    } finally {
        $env:RUSTFLAGS = $PreviousRustFlags
        $env:CARGO_TARGET_DIR = $PreviousTargetDir
        if ($TranscriptStarted) { Stop-Transcript | Out-Null }
        Pop-Location
    }
}
