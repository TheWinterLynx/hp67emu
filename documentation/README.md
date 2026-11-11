# HP-67 PDF documentation

These original PDF files are build inputs. documentation/embedded.rs includes all twelve into the desktop EXE; documentation/sections.rs associates source-backed card references with physical PDF section pages. Desktop users need only the executable. Trunk still copies this folder for browser PDF links, avoiding a very large WASM payload. Regenerate/check the catalogs with tools/generate_embedded_pdf.py. See docs/EMBEDDED_PDF_VIEWER_v1.0.md for provenance, limits and the isolated owner gate.
