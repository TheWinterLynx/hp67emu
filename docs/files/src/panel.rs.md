# `src/panel.rs`

## Purpose

Render the photorealistic HP-67 front panel and expose mechanical switch/key interactions.

## Why it exists

The photographed keys need accurate hit regions and visual travel, but calculator behavior must remain outside the UI. The panel therefore reports the currently held physical key identity rather than emitting a semantic key-click operation.

## Relationships

Uses `Hp67State` for the two slide-switch positions, `HardwareDisplayFrame` for physical LED segments, `KeyAction` only as the identity of a photographed key, the classic display painter for emitted LED geometry, and `ProgramCardView` for the passive card-holder presentation.

## Responsibilities

Render the source photograph; map all 35 key rectangles; animate key travel; report at most one currently held key contact; emit only power/mode switch events; render the hardware display; expose the lateral reader, left-exit tab, left-exit double-click and holder-card interactions without changing machine state; and keep the EEX key mapped to `KeyAction::Exponent`.

## Implementation

`Hp67Panel::show()` returns mechanical/key output plus separate card-reader, left-tab, left-tab-double-click and holder interaction flags. Key contact is captured on the physical primary-button down edge: the key under the pointer at that instant is latched in viewport-local egui state and remains the reported `key_contact` for as long as the primary mouse button stays down. Moving the pointer away from the key therefore cannot release it; only the host mouse-up edge opens the contact. Starting a press outside all calculator keys does not acquire a key merely by dragging over one. The same latched identity drives the key-travel animation. The app translates that identity to `Hp67Key`; the panel never writes ACT state, display state, stack values, arithmetic results, CRC card flags or program RAM. Card rendering is delegated to `ui::program_card`. The panel also passes the original front-panel `TextureHandle` into that renderer so the holder can restore exact photographed chassis pixels over the card along the measured sloped right rail; no flat-color surrogate is used for that occlusion.


M13 blank-media interaction: the panel forwards both the primary reader click and a distinct secondary-click blank-card request from `ui/program_card.rs`. It does not create media itself; the app owns that host-side physical-card action.

Reader waiting-state interaction: the panel forwards a distinct `card_waiting_reader_clicked` event from the visible right-edge card while it is physically inserted but the firmware-owned motor has not started. The panel does not withdraw the card itself; `src/app.rs` routes that request through `Hp67LiveMachine::withdraw_unstarted_magnetic_card()` so the card-present contact and transport ownership remain authoritative.
