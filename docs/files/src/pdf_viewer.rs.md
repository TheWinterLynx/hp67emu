# `src/pdf_viewer.rs`

## Purpose

Provide an internal desktop PDF viewer from embedded original manual bytes.

## Why it exists

The desktop emulator must carry its documentation in one executable and open the instructions for the selected card.

## Relationships

Presentation-only integration with app.rs, documentation.rs and the reviewed PAC import metadata. No machine or electrical state is changed.

## Responsibilities

Preserve PDF bytes, program references and deterministic machine behavior; keep document processing outside the live calculator path.

## Implementation

Hayro 0.7.1 performs CPU rasterization on a lazily started worker. The worker owns one parsed PDF and RenderCache, with bounded request/result queues; stale generations are discarded. PDF data wraps a static slice in Arc without duplicating the document. Rendering is capped at 2200 pixels wide / 3000 high and occurs only after navigation/zoom. A separate native viewport shows one page; closing it drops textures and worker channels. No PDF files, shell commands, PDFium DLL or network are needed at runtime. The isolated --verify-embedded-pdfs command parses all twelve embedded manuals, renders representative pages and validates all section ranges. Windows compilation and real Hayro raster smoke remain owner gates.


The renderer cache is reset when changing pages and reused for zooming on the same page, so decoded rendering resources for every visited page do not accumulate indefinitely.
