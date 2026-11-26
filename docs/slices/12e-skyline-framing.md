# Framing complete fighters on Skyline

The camera fits visible fighters into the central 70% of the viewport, leaving a 15% inset on each side. Each limb has a bounding sphere derived from the map-compatible fighter definition. A sphere uses its radius, a capsule adds its half-height to its radius, and a rounded box adds its border to the length of its half-extents vector. The spheres enclose the drawn shapes at every joint orientation.

For each horizontal or vertical inset plane, let `s` be its half-angle tangent, `p` the limb center relative to the current camera focus and `r` its radius. The required distance is at least `depth(p) + abs(axis(p)) / s + r × hypot(1, 1/s)`. The last term accounts for the limb extending both sideways and toward the lens. The camera checks both planes at the current orbit angle. It expands immediately and eases inward. A narrow viewport can increase the far clipping distance to keep the back edge of the roof visible.

An active online fighter has a center weight of 1.35. Each other tracked fighter has weight 1. Local players and spectators use equal weights. A falling fighter remains in frame while the authoritative state still marks it visible. Eliminated fighters leave both the center and the fit. An empty group returns to the rooftop overview.

## Verification

The framing regression projects every vertex of the drawn rig geometry for two separated, rotated fighters, including one below the roof. It samples all eight orbit intervals, including intermediate angles, and resizes between portrait and landscape during each turn. It checks both the screen inset and clip depth. Separate checks exercise the owned fighter, a departed owner and all four overview corners at a 0.2 viewport aspect ratio. These checks previously failed the inset and the owner bias.

Run `npm --prefix web test -- --run src/rendering/SkylineFraming.test.ts`. See the [Skyline occlusion slice](12f-skyline-occlusion.md) for concealed fighters. Camera collision and human readability remain open.

All 47 web unit tests and the production build passed. The local and online orbit cases and the existing grip, mantle, ring-out and rematch cases passed in Chromium. The full run passed 74 of 76 scenarios. Its two failures concerned delivery of an eight-player elimination and the deadline for a spectator’s content load.
