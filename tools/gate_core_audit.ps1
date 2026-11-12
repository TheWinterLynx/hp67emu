$ErrorActionPreference = 'Stop'
Push-Location (Split-Path $PSScriptRoot -Parent)
$PreviousRustFlags = $env:RUSTFLAGS
try {
    & cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT FMT KO' }
    $env:RUSTFLAGS = '-Dwarnings'
    & cargo test --locked --lib core_audit_ -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT FOCUSED KO' }
    & cargo test --release --locked --lib core_audit_ -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT RELEASE INVARIANTS KO' }
    & cargo test --locked --bin hp67emu core_audit_ -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT LIVE FOCUSED KO' }
    & cargo test --release --locked --bin hp67emu core_audit_ -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT LIVE RELEASE KO' }
    & cargo test --release --locked --test electrical_failure_contract -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT ELECTRICAL ERROR CONTRACT KO' }
    & cargo test --locked --all-targets
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT TESTS KO' }
    & cargo test --locked --all-targets --features pin-capture
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT PIN CAPTURE KO' }
    & cargo build --locked --release --bins
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT RELEASE KO' }
    & cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT DIAGNOSTICS KO' }
    & cargo test --release --locked --bin hp67emu plot_official_polynomial_samples_match_standard_pac -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'FIRMWARE PLOT SAMPLE KO' }
    & cargo test --release --locked --bin hp67emu plot_biorhythms_reads_games_pac_triplets -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'BIORHYTHMS PRINT OUTPUT KO' }
    Write-Host 'CORE INTEGRITY AUDIT FULL GATE GREEN'
} finally {
    $env:RUSTFLAGS = $PreviousRustFlags
    Pop-Location
}
