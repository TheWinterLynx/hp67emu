# Owner-only gate. Run after fetching and checking out the exact review commit.
$ErrorActionPreference = 'Stop'
$previousRustFlags = $env:RUSTFLAGS
Push-Location (Join-Path $PSScriptRoot '..')
try {
    $dirty = & git status --porcelain -- src tests tools Cargo.toml Cargo.lock
    if ($LASTEXITCODE -ne 0) { throw 'Git status KO' }
    if ($dirty) { throw 'Hay cambios locales en source/tests/tools/Cargo; guardalos antes del gate' }
    $head = & git rev-parse HEAD
    if ($LASTEXITCODE -ne 0) { throw 'Git HEAD KO' }
    Write-Host "M16H owner gate v1 | commit=$head"
    function Invoke-HP67M16H {
        param([string[]]$CargoArgs)
        & cargo @CargoArgs
        if ($LASTEXITCODE -ne 0) { throw "M16H GATE v1 KO: cargo $($CargoArgs -join ' ')" }
    }
    Write-Host '[1/9] M16H FORMATO'
    Invoke-HP67M16H -CargoArgs @('fmt', '--all', '--', '--check')
    # Compiler warnings denied, as required by AGENTS.md. Clippy is a separate task.
    $env:RUSTFLAGS = "$previousRustFlags -Dwarnings".Trim()
    Write-Host '[2/9] M16H MOTOR'
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--lib', 'm16h_', '--', '--nocapture')
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--lib', '--features', 'pin-capture', 'm16h_', '--', '--nocapture')
    Write-Host '[3/9] M16H LIVE'
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--bin', 'hp67emu', 'm16h_', '--', '--nocapture')
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--bin', 'hp67emu', '--features', 'pin-capture', 'm16h_', '--', '--nocapture')
    Write-Host '[4/9] M16H ALL TARGETS / WARNINGS DENIED'
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--all-targets')
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--all-targets', '--features', 'pin-capture')
    Write-Host '[5/9] M16H RELEASE'
    Invoke-HP67M16H -CargoArgs @('build', '--locked', '--release', '--bins')
    Invoke-HP67M16H -CargoArgs @('build', '--locked', '--release', '--bins', '--features', 'pin-capture')
    Write-Host '[6/9] M16H DIAGNOSTICO'
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--release', '--bin', 'hp67emu', 'live_custom_diagnostic_pac_suite_reports_ok_ko', '--', '--ignored', '--nocapture', '--test-threads=1')
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--release', '--bin', 'hp67emu', '--features', 'pin-capture', 'live_custom_diagnostic_pac_suite_reports_ok_ko', '--', '--ignored', '--nocapture', '--test-threads=1')
    Write-Host '[7/9] M16H BENCH ELECTRICO'
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--release', '--test', 'hp67_electrical_realtime_benchmark', 'hp67_electrical_realtime_benchmark', '--', '--ignored', '--nocapture', '--test-threads=1')
    Write-Host '[8/9] M16H BENCH DATA Y RAM'
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--release', '--test', 'hp67_data_realtime_benchmark', '--', '--ignored', '--nocapture', '--test-threads=1')
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--release', '--lib', 'm15n_autonomous_serial_ram_realtime_benchmark', '--', '--ignored', '--nocapture', '--test-threads=1')
    Write-Host '[9/9] M16H BENCH LIVE'
    Invoke-HP67M16H -CargoArgs @('test', '--locked', '--release', '--bin', 'hp67emu', 'm14o_live_authority_realtime_benchmark', '--', '--ignored', '--nocapture', '--test-threads=1')
    Write-Host "M16H FULL GATE GREEN | HP67-M16H-RESOLVED-MTRS-v1 | ENTREGA 11/15 | commit=$head"
}
finally {
    $env:RUSTFLAGS = $previousRustFlags
    Pop-Location
}
