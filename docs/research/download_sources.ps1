param(
    [switch]$IncludeFirmware,
    [switch]$IncludeLargeArchives
)

$ErrorActionPreference = 'Stop'
$dest = Join-Path $PSScriptRoot 'downloaded'
New-Item -ItemType Directory -Force -Path $dest | Out-Null

# Publicly reachable reference material used by the emulator research.
# Third-party files keep their original copyright/license. This script only
# downloads from the original/archive URLs; the project does not relicense them.
$sources = @(
    @{ Name='classic-notes.pdf'; Url='https://literature.hpcalc.org/community/classic-notes.pdf' },
    @{ Name='hp67-schematic.pdf'; Url='https://www.hpcc.org/cdroms/schematics5.1/handhelds/classic/hp67.pdf' },
    @{ Name='hp97-service-manual.pdf'; Url='https://literature.hpcalc.org/community/hp97-sm-en.pdf' },
    @{ Name='hp67-owner-handbook.pdf'; Url='https://literature.hpcalc.org/community/hp67-oh-en.pdf' },
    @{ Name='hp-journal-1975-11-woodstock.pdf'; Url='https://www.keesvandersanden.nl/calculators/hp_journals/HP_Journal_7511_Three_New_Pocket_Calculators_Smaller_Less_Costly_More_Powerful.pdf' },
    @{ Name='hp-journal-1976-11-hp67-hp97.pdf'; Url='https://www.keesvandersanden.nl/calculators/hp_journals/HP_Journal_7611_A_Pair_of_Program-Compatible_Personal_Programmable_Calculators.pdf' }
)

if ($IncludeFirmware) {
    # These archives may contain original HP firmware/microcode. They are not
    # redistributed by hp67emu; download them only from their upstream source.
    $sources += @(
        @{ Name='teenix-ROMreader.zip'; Url='https://www.teenix.org/ROMreader.zip' },
        @{ Name='teenix-HP67.zip'; Url='https://www.teenix.org/HP67.zip' }
    )
}

if ($IncludeLargeArchives) {
    $sources += @(
        @{ Name='jacques-laporte-site.zip'; Url='https://archived.hpcalc.org/laporte.zip' }
    )
}

foreach ($source in $sources) {
    $out = Join-Path $dest $source.Name
    Write-Host "Downloading $($source.Name)"
    Invoke-WebRequest -Uri $source.Url -OutFile $out
}

$hashes = foreach ($file in Get-ChildItem -File $dest) {
    $hash = Get-FileHash -Algorithm SHA256 $file.FullName
    [pscustomobject]@{
        File = $file.Name
        Size = $file.Length
        SHA256 = $hash.Hash.ToLowerInvariant()
    }
}

$hashPath = Join-Path $dest 'SHA256SUMS.local.csv'
$hashes | Sort-Object File | Export-Csv -NoTypeInformation -Encoding UTF8 $hashPath
$hashes | Sort-Object File | Format-Table -AutoSize
Write-Host "Hashes written to $hashPath"
