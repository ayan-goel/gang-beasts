# Slice 05b2: Standing rig in the browser

Status: verified on November 24, 2026. Specification §§11.3, 12.2, 26.2, 27.

## Behavior and boundaries

**Test scene → Standing ragdoll** starts the shared fifteen-body rig paused at floor level with the bundled standing motors and balance controller enabled. The passive drop scene remains available. Rendering uses the existing body geometry, joint frames and collider overlay; the standing camera targets the lower spawn height. Playback controls precede the longer motor inspector so resume and single-step remain reachable on desktop.

The inspector shows support used on the last tick, root balance effort, the selected joint's gains/cap and final-substep local effort. An expandable panel shows the actual XYZ solver velocity targets in parent-joint coordinates. Pale parent joint axes represent the desired relative bind pose, while saturated child axes represent the completed pose. The pelvis has a free-yaw upright root target; fixed wrists have no motor. These readings do not claim tick-average torque or solve-time world torque; [ADR 0007](../decisions/0007-support-aware-standing.md) defines their meaning.

A separate control toggle disables both relative motors and balance without pausing physics. Reset and physics tuning preserve this setting. Scene selection starts a fresh default world and discards unsubmitted tuning drafts. The new shove applies one 10 N·s +X impulse to the selected body on the next tick, sharing the existing bounded pending-command slot. Existing 4 N·s upward/forward impulses remain available.

## Implementation choices

- `RagdollFixture` owns either a passive or standing native rig. Both variants share the same body/joint/configuration methods; a single frontend adapter and renderer preserve the existing cleanup boundary. Boxing avoids making the enum's storage scale with the full simulation structs.
- `LocalRagdoll.standing(configuration)` selects the driven fixture explicitly. The existing constructor retains the passive drop. Construction uses canonical character, standing-motor and balance assets.
- Controller metadata is supplied by the native motor configuration. The frontend joins body, joint and motor records by child body name, rather than assuming their indices coincide.
- Configuration replacement is built and validated before replacing the current world. It preserves the standing variant and enable state. Invalid configuration leaves the simulation unchanged.
- Enable values must be JavaScript booleans; validation precedes coercion. Limb indices retain strict finite integer/range validation. Passive fixtures reject controller changes.
- Enable transitions clear cached targets and efforts, including toggling twice while paused. Reset clears diagnostics. A new native regression checks displaced-pose targets, transitions and reset.
- React keys tuning drafts by scenario and canonical configuration. Review reproduced a passive draft surviving a switch to standing when both scenes had identical JSON; the regression now checks both visible inputs and the exported running configuration.

## Owned diagnostic buffers

All arrays are copied out of WASM and remain valid after later allocation, configuration replacement and native object disposal. The existing 210-float body and 238-float joint contracts are unchanged.

| Method | Standing layout | Passive result |
| --- | --- | --- |
| `motor_configuration_json()` | Bundled motor configuration JSON | Absent |
| `motor_targets()` | 14 × XYZ relative velocity targets, rad/s, definition joint order | Empty |
| `motor_efforts()` | 14 × XYZ child-side local solver efforts, N·m, definition joint order | Empty |
| `controller_state()` | Enabled, left support, right support, world pelvis torque XYZ | Empty |

Effort is from the final solver substep. Targets are the commands last sampled for the control tick. Initial, reset and disabled diagnostics are zero; the enabled flag reflects current control state. No full-game snapshot, rollback checkpoint or character recording contract is introduced.

## Verification

Behavioral tests began with missing standing factory/scene failures. Native target lifecycle tests began with the missing getter. The scene-draft regression exposed the React key defect before its correction. Independent review also required strengthening the browser duration test from head height alone to all native standing criteria; the completed test now checks every tick.

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm --prefix web test -- --run
npm --prefix web run build
npm --prefix web run test:e2e
cargo run --quiet --locked --release -p brawler-engine --example standing_trace
```

The full workspace passed **54 native tests**, **12 TypeScript tests**, and **20 Chromium integration tests**. Rust formatting, Clippy with warnings denied, strict TypeScript and the production WASM/browser build passed. Independent review found no remaining source blocker after the two corrections above.

The native and WASM trace compares 601 frames, including the initial state, with all 210 body, 238 joint, 42 effort, 42 target and six controller components. Maximum allowed absolute deviation is below `1e-5`. Before steps 121–123 it queues a 10 N·s +X chest impulse; before steps 360, 420 and 480 it disables, enables and resets respectively. Each operation precedes that iteration's physics step.

A separate uninterrupted **3,600-tick WASM stand** passed: all body/joint/control/effort values finite, head >1.15 m, pelvis >0.65 m, head X/Z drift <0.2 m, anchor separation <0.05 m, support on >3,500 ticks, all local joint efforts within configured vector caps, and root effort ≤30 N·m. The visible UI test also confirms that three chest shoves topple the rig, control toggles/reset work, and switching back releases the standing controls while retaining one canvas.

Boundary tests cover invalid enable/limb types, passive capabilities, owned retained buffers, transactional failed tuning, valid tuning that preserves a disabled standing controller, reset and cleared targets. A frontend unit test deliberately uses differing body/joint/motor ordering to verify selected-joint readings.

Desktop and mobile screenshots were inspected after the final build. At widths 320, 768, 1024 and 1440 px, document width matched viewport width, no page errors occurred, and the support display reported both feet. Temporary screenshots are ignored under `.cache/standing-{desktop,targets,mobile}.png`. They are development evidence, not permanent visual snapshots. The bundle measured 528.58 kB gzipped WASM and 201.04 kB gzipped JavaScript; Vite's raw-chunk advisory and wasm-pack's existing tool notices remain.

## Remaining work

This is a physical standing diagnostic. Walking/running, jumping, recovery, combat, grips, network play, human playability, editable motor gains, complete contact/force overlays and character recording remain unfinished. Measurements use the pinned local toolchain and Chromium, not a supported-browser matrix or release performance environment. All specification §33 release gates remain open. Slice 06 starts locomotion with independently verified native behavior before browser integration.
