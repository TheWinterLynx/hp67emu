param(
    [string]$Uasm = "",
    [string]$TeenixPfl = ".\docs\research\downloaded\teenix-HP67\cal67.pfl",
    [string]$ResearchDir = ".research"
)

$ErrorActionPreference = "Stop"
$NonpareilCommit = "c347bc1ab20170c253512042f7aac0d952f304ea"
$SourceNames = @("67", "6797", "67b1")
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path

function Resolve-Executable([string]$Candidate) {
    if ([string]::IsNullOrWhiteSpace($Candidate)) { return $null }
    if (Test-Path $Candidate -PathType Leaf) { return (Resolve-Path $Candidate).Path }
    $Command = Get-Command $Candidate -ErrorAction SilentlyContinue
    if ($Command) { return $Command.Source }
    return $null
}

function Get-UasmProbe([string]$Executable) {
    $OldPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        return (& $Executable 2>&1 | Out-String)
    } catch {
        return ($_ | Out-String)
    } finally {
        $ErrorActionPreference = $OldPreference
    }
}

function Test-NonpareilUasm([string]$Executable) {
    if ([string]::IsNullOrWhiteSpace($Executable) -or !(Test-Path $Executable -PathType Leaf)) {
        return $false
    }
    $Probe = Get-UasmProbe $Executable
    return $Probe -match "uasm microassembler"
}

function Convert-ToWslPath([string]$WindowsPath) {
    if ([string]::IsNullOrWhiteSpace($WindowsPath)) {
        throw "Cannot convert an empty Windows path to WSL."
    }

    $FullPath = [System.IO.Path]::GetFullPath($WindowsPath)
    if ($FullPath -match '^([A-Za-z]):\\(.*)$') {
        $Drive = $Matches[1].ToLowerInvariant()
        $Tail = $Matches[2] -replace '\\', '/'
        return "/mnt/$Drive/$Tail"
    }

    throw "Unsupported Windows path for WSL conversion: $FullPath"
}

New-Item -ItemType Directory -Force $ResearchDir | Out-Null
$ResearchDir = (Resolve-Path $ResearchDir).Path
$ExternalDir = Join-Path $ResearchDir "external"
New-Item -ItemType Directory -Force $ExternalDir | Out-Null
$NonpareilDir = Join-Path $ExternalDir "nonpareil-$NonpareilCommit"
New-Item -ItemType Directory -Force $NonpareilDir | Out-Null
$NonpareilSourceDir = Join-Path $ExternalDir "nonpareil-src-$NonpareilCommit"

# Always pin the complete upstream source tree. Native uasm only needs the three
# .asm files, while the WSL fallback also builds the official assembler itself.
if (!(Test-Path (Join-Path $NonpareilSourceDir ".git") -PathType Container)) {
    Write-Host "Cloning pinned Nonpareil source ..."
    & git clone --filter=blob:none --no-checkout https://github.com/brouhaha/nonpareil.git $NonpareilSourceDir
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
& git -C $NonpareilSourceDir fetch origin $NonpareilCommit --depth=1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& git -C $NonpareilSourceDir checkout --detach $NonpareilCommit
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$CheckedOutCommit = (& git -C $NonpareilSourceDir rev-parse HEAD).Trim()
if ($CheckedOutCommit -ne $NonpareilCommit) {
    throw "Nonpareil checkout mismatch: expected $NonpareilCommit, got $CheckedOutCommit"
}

$NativeUasm = $null
if (![string]::IsNullOrWhiteSpace($Uasm)) {
    $NativeUasm = Resolve-Executable $Uasm
    if (!$NativeUasm) { throw "Requested uasm was not found: $Uasm" }
    if (!(Test-NonpareilUasm $NativeUasm)) {
        $Probe = (Get-UasmProbe $NativeUasm).Trim()
        throw "The requested executable is not Nonpareil uasm: $NativeUasm`n$Probe"
    }
} else {
    $Candidates = @(
        (Join-Path $RepoRoot "tools\nonpareil-uasm.exe"),
        "nonpareil-uasm",
        "uasm"
    )
    foreach ($Candidate in $Candidates) {
        $Resolved = Resolve-Executable $Candidate
        if ($Resolved -and (Test-NonpareilUasm $Resolved)) {
            $NativeUasm = $Resolved
            break
        }
    }
}

# A very common name collision is Terraspace/UASM, the MASM-compatible x86
# assembler. It is unrelated to Nonpareil despite also being called uasm.
$WrongRepoUasm = Join-Path $RepoRoot "tools\uasm64.exe"
if (!$NativeUasm -and (Test-Path $WrongRepoUasm -PathType Leaf)) {
    $WrongProbe = (Get-UasmProbe $WrongRepoUasm).Trim()
    if ($WrongProbe -match "Masm-compatible assembler|UASM v") {
        Write-Host "Ignoring tools\uasm64.exe: it is the MASM-compatible UASM, not Nonpareil's calculator microassembler."
    }
}

$Objects = @()
$HashLines = @("Nonpareil commit=$NonpareilCommit")

if ($NativeUasm) {
    Write-Host "Using verified Nonpareil uasm: $NativeUasm"
    $UasmHash = (Get-FileHash $NativeUasm -Algorithm SHA256).Hash
    $HashLines += "uasm mode=native"
    $HashLines += "uasm=$NativeUasm"
    $HashLines += "uasm SHA256=$UasmHash"

    foreach ($Name in $SourceNames) {
        $Source = Join-Path $NonpareilSourceDir "ncd\67-97\$Name.asm"
        $Object = Join-Path $NonpareilDir "$Name.obj"
        $Listing = Join-Path $NonpareilDir "$Name.lst"
        Write-Host "Assembling $Name.asm with verified Nonpareil uasm ..."
        & $NativeUasm -o $Object -l $Listing $Source
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        if (!(Test-Path $Object -PathType Leaf)) { throw "uasm did not produce $Object" }
        $Objects += $Object
    }
} else {
    $Wsl = Get-Command wsl.exe -ErrorAction SilentlyContinue
    if (!$Wsl) {
        Write-Host "No verified native Nonpareil uasm was found, and WSL is unavailable."
        Write-Host "The file tools\uasm64.exe is a different MASM-compatible assembler and cannot assemble Nonpareil sources."
        Write-Host "Current Nonpareil does not publish precompiled binaries; build its uasm from the pinned source or enable WSL."
        exit 3
    }

    Write-Host "No verified native Nonpareil uasm found; building the official pinned uasm under WSL ..."
    & $Wsl.Source -e sh -lc "command -v gcc >/dev/null 2>&1 && command -v flex >/dev/null 2>&1 && command -v bison >/dev/null 2>&1"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "WSL is present but gcc/flex/bison are missing."
        Write-Host "Install once inside WSL with: sudo apt-get update && sudo apt-get install -y build-essential flex bison"
        exit 4
    }

    $BuildDir = Join-Path $ResearchDir "nonpareil-uasm-wsl-$NonpareilCommit"
    New-Item -ItemType Directory -Force $BuildDir | Out-Null
    $Helper = Join-Path $RepoRoot "docs\research\build_nonpareil_uasm_wsl.sh"

    # Do not invoke wslpath through wsl.exe with a raw Windows path. Depending
    # on the Windows/WSL argument handoff, backslashes can be consumed before
    # wslpath sees them (for example D:\foo\bar becoming D:foobar). Convert
    # ordinary drive-letter paths deterministically on the PowerShell side.
    $SourceWsl = Convert-ToWslPath $NonpareilSourceDir
    $ObjectWsl = Convert-ToWslPath $NonpareilDir
    $BuildWsl = Convert-ToWslPath $BuildDir

    # Git for Windows may check shell scripts out with CRLF. Feed WSL a local
    # LF-only copy so bash behaviour does not depend on core.autocrlf.
    $HelperLf = Join-Path $ResearchDir "build_nonpareil_uasm_wsl.lf.sh"
    $HelperText = (Get-Content $Helper -Raw) -replace "`r`n", "`n"
    [System.IO.File]::WriteAllText($HelperLf, $HelperText, [System.Text.UTF8Encoding]::new($false))
    $HelperWsl = Convert-ToWslPath $HelperLf

    Write-Host "WSL source path: $SourceWsl"
    Write-Host "WSL output path: $ObjectWsl"
    Write-Host "WSL build path : $BuildWsl"

    & $Wsl.Source -e test -d $SourceWsl
    if ($LASTEXITCODE -ne 0) { throw "WSL cannot access pinned Nonpareil source at $SourceWsl" }
    & $Wsl.Source -e test -f $HelperWsl
    if ($LASTEXITCODE -ne 0) { throw "WSL cannot access build helper at $HelperWsl" }

    & $Wsl.Source -e bash $HelperWsl $SourceWsl $ObjectWsl $BuildWsl
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $WslUasmArtifact = Join-Path $BuildDir "uasm"
    if (!(Test-Path $WslUasmArtifact -PathType Leaf)) {
        throw "WSL build did not produce $WslUasmArtifact"
    }
    $UasmHash = (Get-FileHash $WslUasmArtifact -Algorithm SHA256).Hash
    $HashLines += "uasm mode=WSL official source build"
    $HashLines += "uasm=$WslUasmArtifact"
    $HashLines += "uasm SHA256=$UasmHash"

    foreach ($Name in $SourceNames) {
        $Object = Join-Path $NonpareilDir "$Name.obj"
        if (!(Test-Path $Object -PathType Leaf)) { throw "WSL uasm did not produce $Object" }
        $Objects += $Object
    }
}

foreach ($Name in $SourceNames) {
    $Source = Join-Path $NonpareilSourceDir "ncd\67-97\$Name.asm"
    $SourceHash = (Get-FileHash $Source -Algorithm SHA256).Hash
    $Object = Join-Path $NonpareilDir "$Name.obj"
    $ObjectHash = (Get-FileHash $Object -Algorithm SHA256).Hash
    $HashLines += "Nonpareil $Name.asm SHA256=$SourceHash"
    $HashLines += "Nonpareil $Name.asm URL=https://raw.githubusercontent.com/brouhaha/nonpareil/$NonpareilCommit/ncd/67-97/$Name.asm"
    $HashLines += "Nonpareil $Name.obj SHA256=$ObjectHash"
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
