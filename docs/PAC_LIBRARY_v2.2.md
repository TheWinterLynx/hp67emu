# HP-67 PAC library v2.2

All displayed pack names use `HP-67 ` followed by the full subject. Official card references, historical source filenames and byte payloads remain unchanged. Roman I replaces the mixed 1/I convention. Internal short directory keys are stable identifiers, not UI labels.

| Existing display | Canonical display |
| --- | --- |
| Demo Pac 1 | HP-67 Demonstration Pac I |
| HP-67 Games Pac 1 | HP-67 Games Pac I |
| HP-67 Standard Pac | HP-67 Standard Pac |
| HP-67 Diagnostic Cards | HP-67 Diagnostic Pac |
| Custom Diagnostic Pacs | HP-67 Custom Diagnostic Pac |
| Business Decisions Pac | HP-67 Business Decisions Pac I |
| HP-67 Civil Engineering Pac I | HP-67 Civil Engineering Pac I |
| HP-67 EE Pac I | HP-67 Electrical Engineering Pac I |
| HP-67 Math Pac I | HP-67 Mathematics Pac I |
| HP-67 ME Pac | HP-67 Mechanical Engineering Pac I |
| HP-67 Stat Pac I | HP-67 Statistics Pac I |
| HP-67 Clinical Lab and Nuclear Medicine Pac | HP-67 Clinical Laboratory and Nuclear Medicine Pac |
| HP-67 Navigation Pac I | HP-67 Navigation Pac I |
| HP-67 Surveying Pac I | HP-67 Surveying Pac I |
| Dave Eaton HP-67 | HP-67 Dave Eaton Pac |
| Dave Eaton HP-67/97 2026 | HP-67 Dave Eaton Supplementary Pac (2026) |

## Scope and publication state

The feature branch contains the canonical catalog naming, reviewed extraction profiles, importer, lossless native converter, artwork repair and owner publication gate. The complete 401-card imported payload and eleven manuals currently reside in the owner's v2.0 import workspace/repository. They are not present in the feature branch until the owner runs the publication script. The script publishes the actual locally installed media, not fabricated stand-ins or an empty catalog.

## Local publication

Requires the already installed v2.0 import on `agent/import-all-pacs-local-v2`, Python dependencies from `tools/pac_import/requirements.txt`, Git authentication, and Rust. Close the emulator. Keep the original `G:\Downloads\HP67PACS-preparado-v2.0\review` crops.

`tools/pac_import/publish_local_v2_2.ps1` checks the 401-card manifest and expected origin, rejects unrelated working changes, creates `agent/hp67-pacs-imported`, copies versioned tools, normalizes names, repairs 232 source faces and rebuilds the existing atlas. Existing card references, media paths and payloads are preserved. It runs Python/media validation, Rust formatting, focused catalog/artwork tests, all targets, the release build and the custom diagnostic suite locally. Only after success does it stage the scoped corpus/tool files, commit and push the feature branch. It never merges main or force-pushes.

Source crops always come from first-level PDF extraction. Repair v2.2 uses the preserved crops and fits the whole card into the output canvas uniformly. It restricts boundary cleanup to long thin lines; merely being near a border does not authorize deletion of a glyph. Historical scan wear remains. The repair backs up installed image/atlas bytes and creates a separate review-v2.2 gallery. Programs and original PDFs are unchanged.

GitHub Actions is limited to Rust formatting/check/save on the two feature branches. Compilation and tests are owner local gates. Native checksum validation is not a claim that every application has been executed against its manual.


## Documentation UI v2.3

Help → Documentation lists the Spanish HP-67 owner handbook and all eleven full PAC manuals. Each selected card also offers its PAC manual in desktop, wide browser and compact browser views, including source-linked diagnostic entries. The desktop opens the existing PDF reader; web builds open a new tab. Missing documents are disabled. Availability is cached in UI memory and explicitly refreshed, so opening the library adds no filesystem scan to every frame. No machine/core code or PDF rendering engine is added.

The publisher now requires `hp67-oh-es.pdf` directly in the original source root and all eleven hash-verified imported PAC PDFs. It validates all twelve and the source patch anchors before changing the local branch. Files are packaged in `documentation/`; keep this folder beside a portable executable. Trunk copies it into the web distribution. The initial tools branch deliberately advertises no web PDFs until local validated files are published.
