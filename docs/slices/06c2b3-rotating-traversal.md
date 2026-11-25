# Slice 06c2b3: Rotating-support traversal

Status: native traversal and force contracts verified November 25, 2026. Separate browser diagnostics verified; browser terrain and human movement acceptance remain open.

## Behavior and boundaries

A bounded horizontal foot/body pair compensates the estimated contact load from a rotating support's changing velocity field. It preserves the existing propulsion/stopping laws and shares their absolute impulse budget. [ADR 0014](../decisions/0014-rotating-support-moment.md) defines the load estimate, residual clamp, moment/force accounting, assumptions and alternatives. Root specification §§12.1 and 26.2 remain authoritative.

Locomotion schema 3 adds mandatory `support_moment_cap_nm`, finite [0,120], bundled 120 N·m. Zero disables the pair; older schemas are rejected. This incremental moment is additional to stopping, pure balance and yaw. It remains available during settled supported idle, with no vertical force or transform override. Static/constant translating supports receive zero correction. Native state and the additive owned WASM buffer report the normalized world-space support moment; the existing eleven-value movement layout is unchanged. The playground displays the magnitude separately.

## Reproduction and diagnosis

Baseline `c4f2c7f` failed the formerly ignored centered-deck run 60/release case at release tick 11. A larger-deck fixture holding movement for 240 ticks also failed: rotation −.4 rad/s, radius 0, walk +X lost head clearance at held tick 68. The 30 m half-extent deck excludes edge departure as a false diagnosis; the original finite 6m deck remains a separate regression.

Forward speed can remain above 4 m/s while lateral lean grows on a rotating deck. Both feet move to the same side of COM and then lose support. The old scalar foot allocation tied forward friction compensation to the lateral carry acceleration. Longer observation also found baseline +Z run 45 falling at release 909; the former 360-tick stop fixture was insufficient.

A full vector replacement, gain-unit adjustments, longitudinal clipping and footprint-only settling gates were rejected: they shifted failures into established slope/release cases or did not address late idle drift. Equivalent f32 regrouping changed some rejected candidates' pass counts, indicating inadequate margin rather than a reason to select favorable arithmetic. Unsubtracted carry compensation passed held movement but failed late release; the accepted residual subtracts distributed assistance already supplying the load and clamps to the original estimate.

## Native regression gates

| Fixture | Required checks and observed result |
| --- | --- |
| 32 held/released traversals | ±.4 rad/s × radius 0/2 × walk/run × four world cardinals; settle 60, hold 240, release 360. All pass head >1.1 m, finite state, anchors <.05 m, acceleration/yaw/support caps; held COM travel relative to support at COM exceeds 70% of target distance. Terminal pelvis-relative speed <.2 m/s, active motor strength and stopping moment zero. |
| 64 short deck stops | Center/radius 2, translating/rotating, walk/run, two durations, four cardinals. Release observation extended from 360 to 1200 ticks (20 s); all preserve posture, finite state, anchor and force/moment bounds, terminal speed and complete stopping relaxation. |
| Four finite-deck run 60 stops | Formerly ignored regression now enabled; 1200 release ticks with the same checks. |
| Isolated impulse/moment audit | ±4 rad/s, one/two supported feet, caps0/30/120; actual velocity changes prove no vertical impulse, net diagnostic agreement, absolute impulse ≤8.0002 N·s and saturated moment within .003 N·m. No transform changes. |
| Translation/vanishing-rotation | Matched primed carry history, frame translations0/±8 m/s and rotations0/±.1; moment/command agree within .001. At ±1e−6 rad/s moment <.001 N·m. This does not claim zero-reference startup is frame invariant. |
| Combined stopping/support | Positive-strength neutral braking with both moments nonzero and shared normalization binding. Measured orbital angular impulse equals the sum of reported moments within .003 N·m; absolute impulse remains bounded. Removing the stopping diagnostic's scale factor makes this test fail. |
| Lifecycle/zero budget | Prime nonzero support moment; disable/reset clear immediately, absent support/ineligible posture clear on controller evaluation without modifying body state. Zero acceleration budget applies no impulse. |
| External overpowering | Centered deck, ±.4 rad/s, +X run, twelve 10 N·s cardinal chest impulses before held steps 100–111 (after 99 steps), observe 480 held steps. All eight cases reach head <.8 m with finite state and bounded assistance. |

The overpowering result is phase-specific. Independent repetition with a one-tick later shove or radius 2 produced seven topples out of eight; positive-spin/backward-shove remained upright (radius 2/after 100 minimum head about 1.231 m). No universal twelve-shove, arbitrary-phase or human-feel claim is made. The first parent impact test used those different conditions and failed; it was corrected to reproduce the original experiment exactly, with this limitation retained.

## Verification

- `cargo test --workspace --locked --release`: **126 passed**, zero ignored (86 library + 40 integration).
- `cargo test --locked -p brawler-engine --lib rotating_traversal_tests::contracts`: **3 passed** in debug, including combined-moment measurement. The complete physics regression ran in release; no full-debug result is claimed for this slice.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `npm --prefix web test -- --run`: **17 passed**. New inspector expectation first failed because support moment was absent, then passed with finite/length validation for both moment buffers.
- `npm --prefix web run build`: passed, including rebuilt WASM and strict TypeScript. Existing Vite large-chunk advisory remains.
- Chromium suite on the final rebuilt artifact: **30 passed** with two workers (49.3s). Coverage includes native/WASM flat locomotion/reversal/stopping traces, owned buffer shape/lifetime, empty buffers for unsupported variants, configuration/reset cleanup, and the visible zero-valued support moment on the flat scene.
- Independent source review found no implementation defect and requested the combined-moment regression above; the regression was added and mutation-checked.
- Desktop inspector screenshot reviewed at 1440×1000: support/stopping rows are distinct and readable without crowding the canvas. This does not establish browser rotating-terrain behavior.

Build identity: engine source/compiler fingerprint `ea3147e400d6c6b0fd441eed34556e82d690db7afd150e0496bab211a64a3e20`; WASM SHA-256 `b037e37b7e4a016fbb0c6027684cfd22179ff612318b6df62168215f51fbd9dc`; locomotion JSON SHA-256 `bc3ff443723cfd82990fabe60a85962d763d9bd2a390c1119c8dd74a3613b853`. Rust 1.88.0, Rapier 0.26.1 with enhanced determinism, Node 20.18.0. Production WASM 1,677.46 kB (gzip 547.06 kB); JS 750.06 kB (gzip 203.78 kB). These diagnostic bundle measurements are not final-game performance acceptance.

Raw local logs, rejected experiments and screenshots are ignored under `.cache/`; committed tests and this record preserve the reproducible contracts. No physics acceptance threshold or actuator ceiling was increased to pass the traversal cases.

## Remaining work

Backward/cross-slope travel, steps, terrain entry/exit, changing support normals and browser terrain fixtures remain in slices 06c2b4/06c3. Arbitrarily accelerating/tilting decks and other radii/phases are not certified by these finite fixtures. Jump/get-up, combat, grips, Skyline, networking, full presentation and all §33 release gates remain incomplete.
