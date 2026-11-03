param([string]$Repo = 'D:\ProyectosProgramacion\hp67emu')
$ErrorActionPreference = 'Stop'
$PreviousLocation = Get-Location
$PreviousRustFlags = $env:RUSTFLAGS
try {
    Set-Location -LiteralPath $Repo
    & py -3 (Join-Path $PSScriptRoot 'tools\importar_todos_v2_0.py') verify --repo $Repo
    if ($LASTEXITCODE -ne 0) { throw 'PAC INPUT/OUTPUT VERIFICATION FAILED' }
    & cargo fmt --all
    if ($LASTEXITCODE -ne 0) { throw 'CARGO FMT FAILED' }
    & cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'FMT CHECK FAILED' }
    $env:RUSTFLAGS = '-Dwarnings'
    & cargo test --locked --bin hp67emu program_library::tests -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'PROGRAM LIBRARY FAILED' }
    & cargo test --locked --bin hp67emu embedded_card_artwork -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'ARTWORK ATLAS FAILED' }
    & cargo test --locked --all-targets
    if ($LASTEXITCODE -ne 0) { throw 'ALL TARGETS FAILED' }
    & cargo build --locked --release --bins
    if ($LASTEXITCODE -ne 0) { throw 'RELEASE BUILD FAILED' }
    & cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'CUSTOM DIAGNOSTIC PAC FAILED' }
    Write-Host 'HP67 PAC IMPORT v2.0 FULL GATE GREEN' -ForegroundColor Green
} finally {
    $env:RUSTFLAGS = $PreviousRustFlags
    Set-Location -LiteralPath $PreviousLocation
}
