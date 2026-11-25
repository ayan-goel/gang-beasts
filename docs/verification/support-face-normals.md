# Cuboid support-face normals

Status: implemented in the working tree and verified in focused native tests. Integration with locomotion is still in progress; the movement slice is not accepted or committed.

## Reproduction

A captured rounded foot has core half extents [.08,.05,.14] m and border radius .02 m. Against a cuboid with half extents [30,.5,30] m, the fresh contact query reports a 4.10977 µm gap at surface-local point [−.0503407,.5,17.787502]. This point is more than 12 m from an edge.

The query normal is [−.234177,.821457,−.519970], approximately 31° from the actual face normal [−.258819,.965926,0]. The solved manifold is close to the actual normal. Using the fresh query normal directly for support makes two feet on the same plane receive conflicting terrain targets and friction estimates.

`support_normal_tests.rs` reproduces this with only one dynamic rounded foot and one static deck. A small preload establishes the cached pair; the test then restores the captured pose and zero velocities. Captured f32 quaternion components are retained exactly. The test failed before the correction, independently of the gait and force-controller experiments.

The fixture also covers −.5 mm penetration, +2 mm separation, and loss of support at +30 mm. The correction must preserve the fresh query's contact point and distance.

## Selection rule

`support.rs::contact_normal` uses the current collider pose, including its current parent pose and local transform. For an unrounded cuboid, transform the fresh surface witness into local coordinates. Replace the query normal only if the witness is within numerical tolerance of exactly one face and strictly inside that face's other two boundaries. Return the corresponding current world-space axis normal.

The tolerance is 32 × f32 machine epsilon × max(1, cuboid half extents, absolute world translation, absolute witness coordinates). World coordinates matter: inverse transformation of an exact edge at a 64 m translation can move it several micrometres inside another face. An extent-only tolerance incorrectly replaced that edge's diagonal contact normal. The additional translated/rotated regression failed with the initial tolerance and passes with the coordinate-aware margin. At poorly resolved large coordinates the rule conservatively retains the query normal.

Edges, corners, off-face witnesses and non-cuboid shapes retain the fresh query normal. Do not substitute the normal of the first reported feature: the pinned Parry point projection can return a face identifier for an exact edge or corner. This was reproduced for two contacts whose correct normals point mostly upward while the reported face normal is horizontal.

The selected normal feeds all three consumers: minimum-slope eligibility, relative separation-speed gating, and `FootSupport.normal`. Contact queries, enabled/sensor checks, pair eligibility, friction combination, witness position, contact distance and support point velocity retain their existing roles. No cached manifold normal is used as a replacement for current geometry.

## Focused verification

```sh
cargo test --locked --release -p brawler-engine --lib character::support::
```

All **15 focused tests** pass: nine established support tests plus six new tests. The added cases cover the captured near-touching face, edge/corner/curved fallbacks, the translated edge, independent parent/local-collider rotations after a solve, correct relative departure versus moving support, tangent velocity, and witnesses on both sides of a rotated edge boundary. Independent review found the extent-only margin defect; the revised margin and current-frame handling passed that review's additional fixtures.

The implementation uses no new dependency. Owning source inspected: Parry 0.21.1 `query/point/point_aabb.rs`, `query/point/point_cuboid.rs`, `query/point/point_round_shape.rs`, and `shape/shape.rs`. In particular, a `Face` feature identifier alone does not establish that a witness is in a unique face interior.

Ignored local evidence: `.cache/face-normal-parent-red.txt`, `translated-face-parent-red.txt`, `face-normal-parent-focused.txt`, and the corresponding isolated traces under `.cache/transverse-review/`. These are reproduction conveniences; the committed-ready regression sources do not depend on them.

## Integration boundary

Correct normals alter the existing controller trajectories. The correction alone on the previously committed gait/controller still leaves movement failures; a passing support test does not prove locomotion. The current combined candidate additionally reallocates the existing joint motor budget by predicted directional demand. It has **105/107 library tests passing**, all **40 integration tests passing**, and clean workspace Clippy. All 48 forward/backward/cross-slope traversals and the varied slope-stop matrix pass. Rotating impact compliance and a sustained rotating-platform release remain red. See [the active slice](../slices/06c2b4-transverse-slopes.md) and [motor allocation investigation](motor-effort-allocation.md) for current evidence. Browser/WASM and human acceptance have not been rerun for this candidate.
