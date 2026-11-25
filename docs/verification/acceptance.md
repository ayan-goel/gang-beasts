# MVP acceptance record

No release criteria have passed yet. Update each row with a slice, build/config identity, environment, scenario, result, and limitations. The root specification §33 controls acceptance.

| Requirement | Current evidence |
| --- | --- |
| Browser entry/download budget | Foundation loads in Chromium; [fixture evidence](foundation.md). Final game, browser matrix and critical asset budget pending. |
| 2–8 friend joining and ninth-player handling | Pending |
| Active ragdoll/body/joint/mass | Passive rig has verified 15 bodies/14 joints/40 kg, native/WASM trace parity and [browser inspection](../slices/04b2-ragdoll-browser-inspector.md). [Pose motors](../slices/05a-bounded-pose-motors.md) and [native standing](../slices/05b1-native-standing.md) add bounded actuation evidence. [Browser standing](../slices/05b2-standing-browser.md) verifies the driven native/WASM trace and target/support/effort inspection. Gameplay acceptance remains pending. |
| Locomotion and human playability | [Native standing](../slices/05b1-native-standing.md) and [WASM standing](../slices/05b2-standing-browser.md) pass a 60-second flat-floor stand and yield to pushes. [Native assisted locomotion](../slices/06a-native-locomotion.md) adds flat-ground direction/speed, alternating gait, stop, wall, translating-support and ledge fixtures. [Browser locomotion](../slices/06b-browser-locomotion.md) adds a 481-frame native/WASM trace, focused movement controls, native target axes, live motor caps and lifecycle cleanup tests. [Native moving-support transitions](../slices/06c1-moving-supports.md) add rotating/cardinal deck rides, start/stop, impact and removal evidence. [Native slope fixtures](../slices/06c2a-native-slopes.md) add ±15° uphill/downhill stands, walk/run/stop, impact and bounded-command evidence. [Reversal fixtures](../slices/06c2b1-reversals.md) add repeated flat-ground direction changes, keyboard neutral gaps, bounded transition state and native/WASM traces. [Stopping phase work](../slices/06c2b2-stopping.md) adds flat/forward-slope stops, explicit force-pair moment accounting and short moving-support cleanup. [Rotating-support traversal](../slices/06c2b3-rotating-traversal.md) resolves the formerly ignored regression, adds 32 held/released cases and extends 64 short stops to twenty seconds with measured impulse/moment contracts. The native [terrain traversal and stumble fixtures](../decisions/0015-terrain-locomotion.md) cover backward and cross-slope travel. Steps, terrain transitions, browser terrain, jump/recovery and human playability remain pending. |
| Independent hands | Pending |
| Physical lift/throw | Pending |
| Knockout/recovery | Pending |
| Ledge rescue | Pending |
| Skyline mechanics/reset | Pending |
| Camera readability | Pending |
| Server authority | Pending |
| Smooth multiplayer/latency | Pending |
| Round/match scoring | Pending |
| Disconnect/reconnect/host transfer | Pending |
| Repeated rematch | Pending |
| Measured eight-player performance | Pending |
| Endurance/stability | Pending |
| Original assets/provenance | Pending |
