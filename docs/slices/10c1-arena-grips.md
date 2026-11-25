# Physical grips on Skyline

Each hand can hold one fighter limb or one authored arena object. Both use the same three-axis compliant joint, local anchors, 25 mm surface gap, occlusion test, force cap and stamina budget. Closest surface wins. Equal distances resolve in stable fighter-limb order followed by map ID order. Keyboard release removes the joint immediately. The game HUD identifies the hand and target.

## Reach and eligibility

A reaching shoulder begins pitching toward a low crate or vent within 1.5 arm lengths and in front of the chest. The extra half arm length gives the motor time to rotate during approach. Only the ordinary hand-surface contact test can create the grip. A grounded fighter can lift a movable crate with the same lift control as an opponent.

Decks and walkways only accept contact within one palm radius of an exposed upper lip. A neighboring walkway that covers the edge blocks acquisition there until its geometry moves away. The façade and fall sensor offer no handholds. Both anchors follow their bodies through a panel detachment. Retiring debris releases its joints before the next solve. Round reconstruction clears all attachments.

## Verification

- The native player-input test walks Coral to the southern 7 kg crate, holds the hand controls, identifies object 7, raises the crate by over 12 cm, and verifies releasing leaves its pose unchanged.
- The native edge tests exercise deck interiors, low walls, exposed lips and the shared seam before and after panel failure. A separately positioned physical rig catches the west lip, keeps its grip at detachment and loses it when the panel retires.
- The Chromium keyboard test starts a normal match, grabs the crate on foot, holds through the lift input and checks the grip display clears on release.

The subsequent [ledge catch slice](10c2-ledge-catch.md) adds limited slip and fall-arrest overload calibration for roof and walkway lips. Pulling the pelvis back onto the roof and climbing under obstruction remain open acceptance criteria.
