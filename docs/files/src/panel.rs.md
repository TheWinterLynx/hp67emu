# `src/panel.rs`

## Purpose

Render the photorealistic HP-67 front panel and expose mechanical switch/key interactions.

## Why it exists

The photographed keys need accurate hit regions and visual travel, but calculator behavior must remain outside the UI. The panel therefore reports the currently held physical key identity rather than emitting a semantic key-click operation.

## Relationships

Uses `Hp67State` for the two slide-switch positions, `HardwareDisplayFrame` for physical LED segments, `KeyAction` only as the identity of a photographed key, the classic display painter for emitted LED geometry, and `ProgramCardView` for the passive card-holder presentation.

## Responsibilities

Render the source photograph; map all 35 key rectangles; animate key travel; report at most one currently held key contact; emit only power/mode switch events; render the hardware display; expose the lateral card-reader/window UI interactions without changing machine state; and keep the EEX key mapped to `KeyAction::Exponent`.

## Implementation

`Hp67Panel::show()` returns `Hp67PanelOutput { events, key_contact }`. Each key uses `Response::is_pointer_button_down_on()` so contact state exists for the whole mouse hold rather than only on click release. The app translates that identity to `Hp67Key`; the panel never writes ACT state, display state, stack values, arithmetic results, CRC card flags or program RAM. Card rendering is delegated to `ui::program_card`.
