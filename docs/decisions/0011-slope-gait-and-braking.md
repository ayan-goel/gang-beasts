# ADR 0011: Slope gait and bounded braking

Date: November 25, 2026. Status: implemented; verification is recorded in [slice 06c2a](../slices/06c2a-native-slopes.md). Scope: the native locomotion controller, not general terrain acceptance.

**Extension:** [ADR 0012](0012-reversal-braking.md) adds bounded reversal braking to the signed-allocation eligibility described here. The original release-only policy below records this decision’s historical scope.

## Problem

The flat gait requests level soles and does not account for the height change along a stride. The original +15° fixture fell during idle at tick 115 even after increasing spawn clearance to 0.08 m and proving both feet initially clear of the surface. The short original spawn at 0.04 m penetrated the uphill toe by about 14 mm, so it was unsuitable for attributing the failure solely to control.

Simply aligning the sole using the actual shin orientation delayed the fall but failed the 60-second stand at tick 374. That feedback follows the very tilt the ankle needs to resist. A terrain pose must retain a desired upright frame. Later tests exposed a separate downhill stop failure: friction already brakes a moving foot, but a positive-only drive allocation cannot counter that drag while braking the torso.

## Decision

Add `terrain_gait` after the ordinary gait generator. It consumes existing, freshly validated foot supports and emits relative joint targets. Each leg uses its own support normal, or the other supporting foot's normal when airborne. Neither foot supported means the existing controller releases gait and world assistance. Nearly flat normals (`1 − normal.y < 0.0001`) retain the ordinary gait to avoid responding to query-normal noise.

Terrain adaptation uses gravity-up and the pelvis heading, rather than actual root/shin tilt. This keeps the desired stance restorative. It does not make the entire controller independent of root tilt: the preceding gait still transforms input through the actual root orientation.

The authored straight legs have a hip with pitch/roll freedom, a knee hinge about X, and identity bind joint frames. Derive upper/lower lengths from their anchors. Starting with the nominal gait's ankle displacement, raise/lower the desired ankle along a virtual plane with the measured normal. Reserve extension on the downward side of the stride by:

```
reserve = gait_blend × leg_length × hip_amplitude × horizontal_normal_length / normal.y
height_change = reserve − (normal.x × horizontal_offset.x + normal.z × horizontal_offset.z) / normal.y
```

The linear angular envelope conservatively bounds `sin(hip_amplitude)`. The virtual plane includes hip spacing and changes requested poses only. It is not an absolute foot anchor, a raycast onto a future tread, or a body-height controller. A two-link inverse-kinematics solve supplies hip/knee angles; unreachable distances project onto leg reach with a 1 mm margin at the folded singularity. Match the desired sole normal through the requested shin frame, then independently clamp hip, knee and ankle targets to the authored limits.

Position has priority over exact sole orientation when only the ankle saturates. Rotating the hip to remove ankle orientation error displaced a representative uphill swing target by roughly 0.20 m and caused traversal failures. If a hip or knee itself clips, both the final endpoint and sole normal may miss their desired values. This is a bounded pose request, not a solver guaranteeing all constraints are simultaneously achievable. Preserve arithmetic order in the deterministic build; algebraic regrouping may change floating-point results and must be reverified.

## Drive allocation

The existing controller retains horizontal velocity feedback, support-motion tracking, the 12 m/s² maximum command and the unchanged motor/balance/yaw limits. Extend the friction/lean allocation heuristic to account for the downhill normal reaction and whether friction assists release braking. Only the explicit release-braking interval permits counteracting foot impulses; active movement retains its prior nonnegative foot allocation. During braking, use the signed directional cosine of COM-relative horizontal motion, blended continuously toward the requested direction below one maximum-command tick of speed. This is an estimate from COM motion, not a measurement of sole slip. A hard sign switch failed a near-perpendicular flat-steering regression and was rejected.

During release braking the foot share `f` can range from −1 to 1; active movement retains [0,1]. Negative values apply an opposing impulse at supported feet while a larger same-direction share brakes the distributed body mass. Normalize the requested acceleration before application:

```
scale = 1 / (1 − f + abs(f))
net_acceleration = requested_acceleration × scale
body_impulse_i = mass_i × net_acceleration × dt × (1 − f)
supported_foot_impulse = total_mass × net_acceleration × dt × f / support_count
```

The summed impulse is `total_mass × net_acceleration × dt`. The sum of absolute commanded shares is at most the original command budget: 8 N·s per tick for 40 kg, 12 m/s² and 60 Hz. Combining two impulses on a foot can only reduce the measured sum of per-body impulse magnitudes. Diagnostics report the scaled net acceleration. An idle controller keeps `f = 0`, and losing support removes drive. No vertical command, larger pure balance/yaw torque, position correction or extra impulse budget is introduced.

This remains game assistance and an allocation heuristic, not a complete Coulomb contact solver, traction cap or equal/opposite reaction on the platform. It may reduce net braking acceleration to keep the absolute effort budget. Native/WASM fixture equivalence must still be tested after this change.

## Alternatives rejected

Increasing balance torque, increasing stride amplitude, following actual shin/root tilt, independently moving the hip to recover a saturated sole angle, early motor relaxation, and retaining gait direction during braking did not resolve the tested failures together. They are not part of this change. No acceptance threshold was relaxed to accommodate them.

## Limits

The measured scenarios cover uphill and downhill on ±15° planes in four world orientations, not cross-slope travel. Stairs, abrupt terrain transitions, repeated reversals, browser terrain scenes/parity, and human movement assessment remain open. Existing broad flat-ground and moving-support tests remain required; the original full MVP acceptance gates are unchanged.
