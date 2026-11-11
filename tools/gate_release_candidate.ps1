$ErrorActionPreference = 'Stop'
Push-Location (Split-Path $PSScriptRoot -Parent)
$PreviousRustFlags = $env:RUSTFLAGS
$TranscriptStarted = $false
try {
    $RustVersion = & rustc --version
    if ($LASTEXITCODE -ne 0 -or $RustVersion -notmatch 'rustc (\d+)\.(\d+)') {
        throw 'RUST TOOLCHAIN UNAVAILABLE'
    }
    if ([int]$Matches[1] -eq 1 -and [int]$Matches[2] -lt 92) {
        throw 'Desktop PDF rendering requires Rust 1.92 or newer. Update the stable toolchain first.'
    }
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
    Write-Host '[1/5] Functional integrity, feature coverage, release build and diagnostics'
    & ./tools/gate_core_audit.ps1
    Write-Host '[2/5] Embedded PDF verification from isolated executable'
    $PortableDirectory = Join-Path ([IO.Path]::GetTempPath()) ('hp67-candidate-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $PortableDirectory | Out-Null
    try {
        $PortableExe = Join-Path $PortableDirectory 'hp67emu.exe'
        Copy-Item -LiteralPath 'target/release/hp67emu.exe' -Destination $PortableExe
        # Release uses the Windows GUI subsystem: explicitly wait for this
        # process and inspect its exit code, never a previous LASTEXITCODE.
        $PdfStdout = "$LogPath.pdf.stdout.txt"
        $PdfStderr = "$LogPath.pdf.stderr.txt"
        $PdfProcess = $null
        try {
            $PdfProcess = Start-Process -FilePath $PortableExe -ArgumentList '--verify-embedded-pdfs' -WorkingDirectory $PortableDirectory -RedirectStandardOutput $PdfStdout -RedirectStandardError $PdfStderr -Wait -PassThru
            $PdfOutput = Get-Content -LiteralPath $PdfStdout -Raw
            $PdfErrors = Get-Content -LiteralPath $PdfStderr -Raw
            if ($PdfOutput) { Write-Host $PdfOutput }
            if ($PdfErrors) { Write-Host $PdfErrors }
            if ($null -eq $PdfProcess.ExitCode -or $PdfProcess.ExitCode -ne 0) {
                throw "PORTABLE EMBEDDED PDF KO: exit=$($PdfProcess.ExitCode); logs: $PdfStdout ; $PdfStderr"
            }
            if ($PdfOutput -notmatch '(?m)^EMBEDDED PDFS OK: 12;') {
                throw "PORTABLE EMBEDDED PDF KO: success report missing; logs: $PdfStdout ; $PdfStderr"
            }
        } finally {
            if ($null -ne $PdfProcess) { $PdfProcess.Dispose() }
        }
    } finally {
        try {
            Remove-Item -LiteralPath $PortableDirectory -Recurse -Force -ErrorAction Stop
        } catch {
            # Cleanup must not replace a verifier error (or invalidate its result).
            Write-Warning -WarningAction Continue ("Temporary EXE retained at {0}: {1}" -f $PortableDirectory, $_.Exception.Message)
        }
    }
    $env:RUSTFLAGS = '-Dwarnings'
    Write-Host '[3/5] Current electrical/structural benchmark'
    & cargo test --release --locked --test hp67_electrical_realtime_benchmark hp67_electrical_realtime_benchmark -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'ELECTRICAL BENCHMARK KO' }
    Write-Host '[4/5] Historical DATA comparison benchmark'
    & cargo test --release --locked --test hp67_data_realtime_benchmark hp67_data_realtime_benchmark -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'DATA BENCHMARK KO' }
    Write-Host '[5/5] Production live benchmark and labelled reference workloads'
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
