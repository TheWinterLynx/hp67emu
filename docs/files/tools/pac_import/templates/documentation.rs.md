# `tools/pac_import/templates/documentation.rs`

## Purpose

Version the exact presentation module deployed by the local documentation installer.

## Why it exists

The owner imported branch contains media not yet in GitHub, so integration must preserve that local catalog while installing the same reviewed UI module.

## Relationships

Copied to src/documentation.rs by install_documentation_v2_3.py; source and template must be identical.

## Responsibilities

Keep the documentation menu and PDF access reproducible without replacing imported program/card state.

## Implementation

A byte-identical template of src/documentation.rs. The owner local gate compiles the installed module.
