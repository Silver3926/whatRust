# Call Popup v3 Baseline

## Scope

This document records the baseline for the staged call-popup-v3 experiment. It is intentionally documentation-only: no application behavior has been changed yet.

## Repository baseline

- Repository: `Silver3926/whatRust`
- Base branch: `master`
- Base commit: `cc0d52107dcda85a789b5632c9f3e3bbf5bea866`
- New branch: `feat/call-popup-v3`
- Application version: `0.6.3`
- Relevant WebView implementation: Windows WebView2 through `webview2-com`
- Existing popup experiments deliberately excluded from this branch:
  - `feat/call-popup-window`
  - `feat/call-popup-v2`

## Current implementation expectations

The `master` code does not contain the experimental `call_popup.rs` module. It uses the normal WebView2 popup behavior. The current investigation is focused on two separate behaviors:

1. The automatic WhatsApp call popup can remain owned by the account window and therefore follow it when minimized.
2. WhatsApp's explicit “Pop out to new window” action can report that popups are blocked when the host does not provide a valid WebView2 replacement through `NewWindowRequested`/`SetNewWindow`.

These are hypotheses to be verified on the Windows test machine; this document does not claim that a runtime test has been completed.

## Manual baseline checklist

Run these checks on the same Windows machine and WebView2 runtime before implementing the next stage:

- [ ] Record Windows version.
- [ ] Record WebView2 Runtime version.
- [ ] Record Rust and Tauri CLI versions used for the build.
- [ ] Build the unmodified `master` branch.
- [ ] Run `cargo test --locked` from `src-tauri`.
- [ ] Login to one WhatsApp account.
- [ ] Start an outgoing audio call.
- [ ] Start an outgoing video call.
- [ ] Minimize the account window during a call and record whether the call window follows it.
- [ ] Use WhatsApp's “Pop out to new window” action and record whether the “Allow pop-ups” warning appears.
- [ ] End the call and start a second call.
- [ ] If available, repeat with a second WhatsApp account.
- [ ] Record any crash, hang, login loss, permission failure, or orphaned window.

## Stage 0 acceptance criteria

Stage 0 is complete when:

- This branch is based directly on `master`.
- No call-popup implementation has been copied from either experimental branch.
- The baseline build and test results are recorded above or in the associated test report.
- The two current behaviors are reproducible or explicitly marked as not reproducible.
- The next stage can be rolled back independently without touching `master`.

## Next stage

The next stage is a minimal WebView2 popup harness. It must verify the difference between a handled popup without a replacement and a handled popup supplied through `SetNewWindow`, before WhatsApp integration is attempted.
