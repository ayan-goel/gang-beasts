# Reduced camera motion

Local and private Skyline players can choose **Camera motion** in the page header before play or in the match menu. **Use device setting** follows the operating system’s reduced-motion preference, including changes during a round. **Reduced** and **Full** override it. The choice persists in this browser under `skyline.camera.v1.motion`. Tabs on the same origin pick up changes to the setting. An absent or unrecognized value uses the device setting. A browser that denies storage keeps the choice for the current page.

## Framing and controls

In Reduced mode, minor ragdoll movement leaves the camera still. The group’s usual weighted center can move within an ellipsoid around the current aim, with a 0.75 m horizontal radius and a 0.25 m vertical radius. Outside it, the camera follows the excess displacement using the existing time-based damping. The smaller vertical margin keeps the camera tracking a climb or fall. The overview continues to aim at the arena center.

When the fighters need a wider shot, the camera fits them on that frame with 12% extra distance. It holds the zoom until the required distance exceeds the available space or the camera is more than 25% farther away than necessary. In the latter case it eases closer. Every candidate camera position still goes through full-body framing and the obstacle clearance check. [Camera controller](../../web/src/rendering/GroupCamera.ts)

A press of Z or X selects the next 45° view in a single rendered frame. Held movement follows the visible heading. Full motion retains the original animated orbit, continuous pan and zoom. Local fighters share one view. Private players and spectators each control their own camera. The camera preference is independent of graphics quality. [Orbit and controls](12d-camera-orbit.md)

## Verification

- The [motion fixtures](../../web/src/rendering/reducedCamera.test.ts) measure camera position during 120 frames of small fighter movements, then separate and drop the fighters. They check viewport bounds throughout and compare input direction with the selected orbit.
- The [full-rig projection test](../../web/src/rendering/SkylineFraming.test.ts) checks mesh vertices against the central 70% in both modes across eight headings and a portrait-to-landscape resize. The [clearance tests](../../web/src/rendering/CameraClearance.test.ts) check the near lens against a blocking wall in both modes.
- The [browser tests](../../web/tests/camera-motion.spec.ts) measure the yaw change on the first actual draw after a keyboard turn. They cover a live device preference change, a saved menu override, two players using different modes, and local controls at viewport widths of 320, 768 and 1024 pixels. Human comfort and readability remain playtest criteria.

The production build passed both camera cases in Chromium, Firefox and Playwright WebKit on macOS. Chromium also passed the two graphics-setting and two match-menu regressions. All 115 web unit tests and TypeScript checks passed.

Run `npm --prefix web test -- --run` and `npm --prefix web run test:e2e -- camera-motion.spec.ts graphics-settings.spec.ts match-menu.spec.ts`.
