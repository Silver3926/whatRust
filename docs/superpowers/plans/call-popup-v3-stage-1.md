# Stage 1 — WebView2 popup harness

## Purpose

The popup problem must be tested independently from WhatsApp before the host integration is changed. This stage adds a small page-side harness at `settings-ui/popup-harness.html`.

The page tests the observable contract needed by WhatsApp:

```js
const popup = window.open(url, ...);
popup !== null;
```

It also checks whether the opener retains a `WindowProxy` and can call `popup.closed` and `popup.close()`.

## Important limitation

The page alone cannot create a WebView2 replacement. The host-side cases still need to be wired in a later implementation step:

1. Default WebView2 popup.
2. `NewWindowRequested` with `SetHandled(true)` and no replacement.
3. `NewWindowRequested` with `SetNewWindow(replacement)` followed by `SetHandled(true)`.

This commit intentionally does not change production popup handling and does not claim that case 3 passes yet.

## Manual execution plan

The harness must be loaded inside a Windows WebView2 instance, not only in a normal browser. For each host configuration:

1. Open the harness.
2. Click **Open popup**.
3. Record whether the result says `WindowProxy` or `null`.
4. Click **Check existing popup**.
5. Confirm that `popup.closed` can be read.
6. Click **Close popup**.
7. Confirm that the child window closes.
8. Repeat after minimizing the opener.

For each run record:

- Windows version
- WebView2 Runtime version
- host configuration (default / handled-only / replacement)
- `window.open()` result
- whether the child window is visible
- whether the child stays visible after the opener is minimized
- whether the opener/child relationship remains usable
- crash, hang, or orphaned window

## Stage 1 checkpoint

The stage is ready for host-side integration when:

- The page loads without JavaScript errors.
- The default case can be exercised in a WebView2 host.
- The result format distinguishes `null` from a valid `WindowProxy`.
- The child can be checked and closed from the opener.
- No production popup behavior was changed by this stage.

The stage is not considered to have proven `SetNewWindow` until the host-side replacement case has been implemented and tested.
