# Embedded PDF viewer v1.0

Desktop distribution is one executable: the twelve original PDFs, card media, artwork atlas and photographed UI are linked into it. The PDFs total about 598 MB before executable overhead; no extra compression or scan alteration is applied. Only the selected PDF is parsed and only the requested page rendered. The embedded font-enabled Hayro 0.7.1 renderer is pure Rust; it requires Rust 1.92 or newer for development/building. Existing locked package versions are retained; 43 new dependency packages are added.

Selecting a card exposes **Ver instrucciones del programa** for the 349 source-backed section links. It opens the integrated native viewer at the physical PDF page from the reviewed application profile. The button says **Ver manual completo del PAC** when only the full manual is identified. Help remains right-aligned, and packs/cards/PAC manuals remain alphabetically ordered. The owner handbook remains in its own documentation section.

The viewer provides previous/next page, zoom and return to the selected application's first section page. It runs in its own native viewport. A background worker retains one parsed document and rendering cache, bounds queued work, rejects stale results and releases its resources when closed. No PDF work is performed in the electrical engine or on every UI frame. Unsupported PDF rendering produces a visible error; renderer accuracy is not claimed beyond owner raster/visual validation.

Browser builds retain packaged PDF links with #page, and do not link the native renderer or the 598 MB of manuals into the WASM module. The single-EXE/internal-viewer guarantee applies to desktop.

Sources for the renderer API: https://docs.rs/hayro/0.7.1/hayro/ and the crate's published render example. Section provenance remains tools/pac_import/tools/profiles-v2.0.json and programs/HP67/ImportedPACs/import_manifest.json. These are reviewed application/card-section pages, not inferred printed page numbers or unverified text search results.

Run tools/gate_embedded_pdf.ps1 locally. Its final smoke runs the compiled executable from a fresh folder containing only the EXE, parses all twelve manuals, renders a representative page from each and checks all section page ranges. Agent checks performed: Rust formatting, Cargo locked metadata resolution, retention of all pre-existing dependency versions, Python source parsing/catalog generation, all 12 source PDFs parsed with PyMuPDF, all 349 target pages in bounds, and 11 PAC SHA256 matches. No Rust compile/test or Hayro raster pass is claimed here.

The previous tools/pac_import publisher remains a historical bundle importer. On a checkout with the integrated viewer, update/build this branch and use the new gate rather than reinstalling its older external-reader template.

Renderer caches are retained for the current page/zoom and reset on page changes. Closing the viewer releases the worker channels, parsed document and textures.
