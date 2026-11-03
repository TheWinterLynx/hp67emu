param([string]$Repo='D:\ProyectosProgramacion\hp67emu',[string]$Work='G:\Downloads\HP67PACS-preparado-v2.0')
$ErrorActionPreference='Stop'
function Invoke-GitChecked {
    param([string[]]$GitArgs)
    & git -C $Repo @GitArgs
    if ($LASTEXITCODE -ne 0) { throw "GIT FAILED: $($GitArgs -join ' ')" }
}
$previous=Get-Location
try {
    $Repo=(Resolve-Path -LiteralPath $Repo).Path
    $manifest=Join-Path $Repo 'programs\HP67\ImportedPACs\import_manifest.json'
    if (-not (Test-Path -LiteralPath $manifest)) { throw 'Import v2.0 must already be installed; no repository changes made.' }
    $entries=(Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json).entries
    if ($entries.Count -ne 401) { throw "Expected 401 imported cards, found $($entries.Count). No changes made." }
    & py -3 (Join-Path $PSScriptRoot 'tools\install_documentation_v2_3.py') --repo $Repo --work $Work --check
    if ($LASTEXITCODE -ne 0) { throw 'DOCUMENTATION PREREQUISITES FAILED; no changes made.' }
    $remote=& git -C $Repo remote get-url origin
    if ($LASTEXITCODE -ne 0 -or $remote -notmatch 'github\.com[:/]TheWinterLynx/hp67emu(?:\.git)?$') { throw 'Unexpected origin; no publication performed.' }
    $branch=& git -C $Repo branch --show-current
    if ($LASTEXITCODE -ne 0 -or $branch -notin @('agent/import-all-pacs-local-v2','agent/hp67-pacs-imported')) { throw "Run from the imported PAC branch, found $branch" }
    if (Test-Path -LiteralPath (Join-Path $Repo 'artwork-backup-v2.2')) { throw 'Repair backup v2.2 already exists. Review existing results before rerunning.' }
    $allowed=@('src/program_library.rs','src/app.rs','src/main.rs','src/documentation.rs','index.html','documentation/','docs/files/src/documentation.rs.md','docs/files/src/main.rs.md','assets/hp67-card-artwork-atlas.png','assets/hp67-imported-artwork/','programs/HP67/ImportedPACs/','docs/PAC_IMPORT_V2.md','docs/files/src/program_library.rs.md','docs/files/src/app.rs.md','tools/pac_import/','docs/PAC_LIBRARY_v2.2.md','docs/files/tools/pac_import/','.github/workflows/pac-library-rustfmt.yml','artwork-backup-v2.1/','artwork-backup-v2.2/')
    $changes=& git -C $Repo -c core.quotepath=false status --porcelain
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect local changes.' }
    foreach ($line in $changes) {
        $path=$line.Substring(3).Trim('"')
        $ok=$false
        foreach ($prefix in $allowed) { if ($path -eq $prefix -or ($prefix.EndsWith('/') -and $path.StartsWith($prefix))) { $ok=$true; break } }
        if (-not $ok) { throw "Unrelated local change: $path. Save it separately before publication." }
    }
    Write-Host '[1/6] Preserve imported work on publication branch'
    if ($branch -ne 'agent/hp67-pacs-imported') { Invoke-GitChecked -GitArgs @('switch','-c','agent/hp67-pacs-imported') }
    Write-Host '[2/6] Install versioned import, repair and publication tools'
    $destination=Join-Path $Repo 'tools\pac_import'
    if ([IO.Path]::GetFullPath($PSScriptRoot) -ne [IO.Path]::GetFullPath($destination)) {
        New-Item -ItemType Directory -Path $destination -Force | Out-Null
        Copy-Item -Path (Join-Path $PSScriptRoot '*') -Destination $destination -Recurse -Force
    }
    # Companion documentation and policy files travel with the same Git bundle.
    $bundleRoot=Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
    foreach ($relative in @('docs\PAC_LIBRARY_v2.2.md','docs\files\tools\pac_import','.github\workflows\pac-library-rustfmt.yml')) {
        $source=Join-Path $bundleRoot $relative
        if (Test-Path -LiteralPath $source) {
            $target=Join-Path $Repo $relative
            New-Item -ItemType Directory -Path (Split-Path $target -Parent) -Force | Out-Null
            Copy-Item -LiteralPath $source -Destination $target -Recurse -Force
        }
    }
    Write-Host '[3/6] Normalize all catalog and manifest PAC names'
    & py -3 (Join-Path $destination 'tools\normalize_pack_names_v2_2.py') --repo $Repo
    if ($LASTEXITCODE -ne 0) { throw 'PACK NAMES FAILED' }
    Add-Content -LiteralPath (Join-Path $Repo 'docs\files\src\program_library.rs.md') -Value "`nPAC library v2.2: all displayed packs use full HP-67 names; legacy media paths/references remain stable."
    & py -3 (Join-Path $destination 'tools\install_documentation_v2_3.py') --repo $Repo --work $Work
    if ($LASTEXITCODE -ne 0) { throw 'DOCUMENTATION INSTALL FAILED; no commit or push performed.' }
    Write-Host '[4/6] Repair all 232 card faces and rebuild installed atlas'
    & py -3 (Join-Path $destination 'tools\reparar_graficos_v2_1.py') --work $Work --repo $Repo
    if ($LASTEXITCODE -ne 0) { throw 'ARTWORK REPAIR FAILED; no commit or push performed.' }
    Write-Host '[5/6] Full local Rust/media gate'
    & (Join-Path $destination 'gate_v2_0.ps1') -Repo $Repo
    Write-Host '[6/6] Commit only PAC files and publish feature branch'
    Set-Location -LiteralPath $Repo
    $paths=@('src/program_library.rs','src/app.rs','src/main.rs','src/documentation.rs','index.html','documentation','docs/files/src/documentation.rs.md','docs/files/src/main.rs.md','assets/hp67-card-artwork-atlas.png','assets/hp67-imported-artwork','programs/HP67/ImportedPACs','docs/PAC_IMPORT_V2.md','docs/files/src/program_library.rs.md','docs/files/src/app.rs.md','tools/pac_import')
    foreach ($optional in @('docs/PAC_LIBRARY_v2.2.md','docs/files/tools/pac_import','.github/workflows/pac-library-rustfmt.yml')) { if (Test-Path -LiteralPath $optional) { $paths += $optional } }
    foreach ($path in $paths) {
        Get-ChildItem -LiteralPath $path -Recurse -File -ErrorAction Stop | Where-Object Length -GE 100MB | ForEach-Object { throw "File exceeds GitHub regular Git limit: $($_.FullName)" }
    }
    Invoke-GitChecked -GitArgs (@('add','--')+$paths)
    Invoke-GitChecked -GitArgs @('diff','--cached','--check')
    Invoke-GitChecked -GitArgs @('commit','-m','Import complete HP-67 PAC library with full pack names and preserved card glyphs')
    Invoke-GitChecked -GitArgs @('push','--set-upstream','origin','HEAD:refs/heads/agent/hp67-pacs-imported')
    Write-Host 'HP-67 PAC LIBRARY PUBLISHED: 401 native cards, calculator handbook, 11 PAC manuals, corrected artwork and full names. main unchanged.' -ForegroundColor Green
} finally { Set-Location -LiteralPath $previous }
