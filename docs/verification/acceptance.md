# MVP acceptance record

No release criteria have passed yet. Update each row with a slice, build/config identity, environment, scenario, result, and limitations. The root specification §33 controls acceptance.

| Requirement | Current evidence |
| --- | --- |
| Browser entry/download budget | Foundation loads in Chromium; [fixture evidence](foundation.md). Final game, browser matrix and critical asset budget pending. |
| 2–8 friend joining and ninth-player handling | Pending |
| Active ragdoll/body/joint/mass | Passive rig has verified 15 bodies/14 joints/40 kg, native/WASM trace parity and [browser inspection](../slices/04b2-ragdoll-browser-inspector.md). [Pose motors](../slices/05a-bounded-pose-motors.md) and [native standing](../slices/05b1-native-standing.md) add bounded actuation evidence. [Browser standing](../slices/05b2-standing-browser.md) verifies the driven native/WASM trace and target/support/effort inspection. Gameplay acceptance remains pending. |
| Locomotion and human playability | [Native standing](../slices/05b1-native-standing.md) and [WASM standing](../slices/05b2-standing-browser.md) pass a 60-second flat-floor stand and yield to pushes. [Native assisted locomotion](../slices/06a-native-locomotion.md) adds flat-ground direction/speed, alternating gait, stop, wall, translating-support and ledge fixtures. Browser locomotion, terrain, jump/recovery and human playability remain pending. |
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
