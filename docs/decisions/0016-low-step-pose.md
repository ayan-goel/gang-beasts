# ADR 0016 — Advance the feet when approaching a low tread

The stair failure begins before the head drops. On the old 60-tick approach the right foot hit the second riser with a straight requested knee while the pelvis was already 0.264 m ahead of it. Adding swing clearance alone left the obstructed stance foot behind. The accepted controller changes hip and ankle pitch on both legs when it detects room to plant the foot above a low face. Rapier still determines where the sole lands.

## Detection

The shared world updates its Rapier scene-query index during each solve. With a supporting foot and forward movement within 45 degrees of the pelvis heading, each foot can probe from its center projected onto its support plane. An airborne foot can use the other foot’s support plane. A horizontal ray just above the plane looks ahead by the projected sole half-length and a further thigh-to-shin center distance. Four downward rays check the corners of a prospective footprint beyond the face. All four must find the same fixed collider, with a walkable top between the contact margin and half the thigh-to-shin distance above the plane. The bundled upper bound is 0.15 m. Filters exclude sensors and respect the foot’s collision groups. Tall walls, narrow rails and short treads fail the geometric checks.

## Pose

On detection, bias both hips forward by 0.40 radians minus 0.50 times the pelvis up-axis projection onto the movement input. This lets a planted leg extend as the torso tips forward. Add 0.50 radians at the ankles to present an upturned toe. Movement intensity scales the offsets. Each joint retains its anatomical limit, stiffness, damping and torque ceiling. The ordinary gait continues to choose stance, swing and knee flexion. Detection persists for 20 ticks after the last valid sample so brief loss of sight at a tread edge does not immediately erase the pose. Release takes four more ticks. Neutral input, turning away, an explicit jump, loss of upright posture and controller reset clear the approach. Airborne ticks retain the existing pose-release and ground-drive rules.

Version-three locomotion configurations may supply a `low_step` object. Omitting the object selects the bundled defaults. The change adds no movement packets, world-space foot anchors, vertical thrust or animation-only motion. See the [verification record](../slices/06c2b4b-stairs.md#verified-ascent-and-landing).
