$ErrorActionPreference = 'Stop'
Push-Location (Split-Path $PSScriptRoot -Parent)
$PreviousRustFlags = $env:RUSTFLAGS
$TranscriptStarted = $false
try {
    $Revision = & git rev-parse HEAD
    if ($LASTEXITCODE -ne 0) { throw 'CANNOT IDENTIFY RELEASE CANDIDATE' }
    $Changes = & git status --porcelain --untracked-files=no
    if ($LASTEXITCODE -ne 0) { throw 'CANNOT CHECK RELEASE CANDIDATE' }
    if ($Changes) { throw 'Tracked changes exist: commit them before certifying an exact release candidate.' }
    $LogDirectory = Join-Path (Get-Location) 'target/release-gate'
    New-Item -ItemType Directory -Force -Path $LogDirectory | Out-Null
    $LogPath = Join-Path $LogDirectory ("{0}-{1}.txt" -f $Revision, (Get-Date -Format 'yyyyMMdd-HHmmss'))
    Start-Transcript -Path $LogPath | Out-Null
    $TranscriptStarted = $true
    Write-Host "Candidate: $Revision"
    Write-Host '[1/4] Functional integrity, feature coverage, release build and diagnostics'
    & ./tools/gate_core_audit.ps1
    $env:RUSTFLAGS = '-Dwarnings'
    Write-Host '[2/4] Current electrical/structural benchmark'
    & cargo test --release --locked --test hp67_electrical_realtime_benchmark hp67_electrical_realtime_benchmark -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'ELECTRICAL BENCHMARK KO' }
    Write-Host '[3/4] Historical DATA comparison benchmark'
    & cargo test --release --locked --test hp67_data_realtime_benchmark hp67_data_realtime_benchmark -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'DATA BENCHMARK KO' }
    Write-Host '[4/4] Production live benchmark and labelled reference workloads'
    & cargo test --release --locked --bin hp67emu m14o_live_authority_realtime_benchmark -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'LIVE BENCHMARK KO' }
    Write-Host "RELEASE CANDIDATE AUTOMATED GATE GREEN | $Revision"
    Write-Host 'Manual desktop acceptance and source-blocked hardware claims are separate.'
    Write-Host "Evidence: $LogPath"
} finally {
    $env:RUSTFLAGS = $PreviousRustFlags
    if ($TranscriptStarted) { Stop-Transcript | Out-Null }
    Pop-Location
}
