param([string]$Repo='D:\ProyectosProgramacion\hp67emu')
$ErrorActionPreference='Stop'
$previous=Get-Location
$previousFlags=$env:RUSTFLAGS
try {
    Set-Location -LiteralPath $Repo
    $version=& rustc --version
    if ($LASTEXITCODE -ne 0 -or $version -notmatch 'rustc (\d+)\.(\d+)') { throw 'Rust toolchain unavailable' }
    if ([int]$Matches[1] -eq 1 -and [int]$Matches[2] -lt 92) { throw 'Requires Rust 1.92 or newer; run rustup update stable first.' }
    & py -3 tools/generate_embedded_pdf.py --check
    if ($LASTEXITCODE -ne 0) { throw 'EMBEDDED DOCUMENT CATALOG KO' }
    & cargo fmt --all
    if ($LASTEXITCODE -ne 0) { throw 'FMT KO' }
    & cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'FMT CHECK KO' }
    $env:RUSTFLAGS='-Dwarnings'
    & cargo test --locked --bin hp67emu documentation::tests -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'DOCUMENTATION TESTS KO' }
    & cargo test --locked --bin hp67emu pdf_viewer::tests -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'PDF VIEWER TESTS KO' }
    & cargo test --locked --all-targets
    if ($LASTEXITCODE -ne 0) { throw 'ALL TARGETS KO' }
    & cargo build --locked --release --bins
    if ($LASTEXITCODE -ne 0) { throw 'RELEASE KO' }
    & cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'CUSTOM DIAGNOSTIC KO' }
    $portable=Join-Path ([IO.Path]::GetTempPath()) ('HP67-Portable-'+[guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $portable | Out-Null
    $exe=Join-Path $portable 'hp67emu.exe'
    Copy-Item -LiteralPath 'target\release\hp67emu.exe' -Destination $exe
    Push-Location -LiteralPath $portable
    try {
        & $exe --verify-embedded-pdfs
        if ($LASTEXITCODE -ne 0) { throw 'ISOLATED EXE PDF SMOKE KO' }
    } finally { Pop-Location }
    Write-Host 'HP-67 EMBEDDED PDF VIEWER FULL GATE GREEN' -ForegroundColor Green
    Start-Process -FilePath $exe -WorkingDirectory $portable
} finally {
    $env:RUSTFLAGS=$previousFlags
    Set-Location -LiteralPath $previous
}
