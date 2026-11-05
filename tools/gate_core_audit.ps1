$ErrorActionPreference = 'Stop'
Push-Location (Split-Path $PSScriptRoot -Parent)
$PreviousRustFlags = $env:RUSTFLAGS
try {
    & cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT FMT KO' }
    $env:RUSTFLAGS = '-Dwarnings'
    & cargo test --locked --lib core_audit_ -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT FOCUSED KO' }
    & cargo test --locked --all-targets
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT TESTS KO' }
    & cargo test --locked --all-targets --features pin-capture
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT PIN CAPTURE KO' }
    & cargo build --locked --release --bins
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT RELEASE KO' }
    & cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'CORE AUDIT DIAGNOSTICS KO' }
    Write-Host 'CORE INTEGRITY AUDIT FULL GATE GREEN'
} finally {
    $env:RUSTFLAGS = $PreviousRustFlags
    Pop-Location
}
