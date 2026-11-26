# Skyline graphics presets

The Graphics selector appears in the local game header and in the private-room header, including the lobby. Auto is the default. A player can change the preset before a match, during play or while spectating. The preference applies to Skyline scenes. The diagnostic playground retains its existing rendering configuration.

| Choice | Drawing buffer per CSS pixel | Directional shadow map |
| --- | --- | --- |
| Low | 0.75 × min(device pixel ratio, 1) | 512 × 512, basic |
| Medium | min(device pixel ratio, 1.5) | 1024 × 1024, soft PCF |
| High | min(device pixel ratio, 2) | 2048 × 2048, soft PCF |

Auto resolves to Low on recognized software drivers and Medium otherwise. A manual selection overrides that decision. The renderer keeps its canvas and WebGL context. Changing the shadow resolution releases the old GPU target. Changing the shadow filter invalidates the material programs so the next draw uses the selected filter. [Preset selection](../../web/src/rendering/skylineQuality.ts) · [Renderer changes](../../web/src/rendering/applySkylineQuality.ts)

The preference uses browser storage. Same-origin tabs apply a newly saved selection to their own scene. When storage is unavailable, the selector still works for the current page. Separate browser profiles can use different graphics settings in the same match. [Preference lifecycle](../../web/src/rendering/graphicsPreference.ts)

## Verification

At a device pixel ratio of two, Chromium drew at 0.75, 1.5 and 2 drawing-buffer pixels per CSS pixel. The allocated shadow targets were 512, 1024 and 2048 pixels wide. Repeated switches retained the original WebGL canvas, the original context and round one’s advancing clock. A listener on the original shadow target observed a single disposal. Reload restored High, and a change in a second same-origin lobby tab updated the running local scene. [Local and private browser scenarios](../../web/tests/graphics-settings.spec.ts)

In a two-player private match, the host selected Low and the guest selected High. The guest ran off the roof with the keyboard and both clients awarded the same point. The eight-player scoring and narrow-screen local menu regressions also passed. The graphics controls fit a 390 × 800 viewport. Tests ran on an Apple M3 Pro, macOS 26.6.2, with Chromium and Metal over loopback.

A separate Chromium run forced SwiftShader. Auto used a 0.75 drawing-buffer ratio and 512-pixel shadows, Medium overrode both, and returning to Auto restored both. The release build, TypeScript check and 84 web unit tests passed. [Software browser regression](../../web/tests/graphics-software.spec.ts)

These measurements cover resolution, shadow allocation and gameplay continuity. Eight-player frame-time budgets, frame caps and additional effects have separate verification work.
