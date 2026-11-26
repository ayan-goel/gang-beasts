# Directional punch targeting

On a fresh punch press, that hand checks the other active fighters once. The shared native and WASM controller uses the attacker's chest orientation for forward intent and the punching shoulder for the origin. The left and right hands can select different opponents. A punch without a candidate still follows its ordinary motor sequence.

## Reach, ordering and obstruction

The controller checks the current collision shapes against the distance from the rest shoulder to the hand plus the palm radius. It first tries the shoulder’s forward ray. If the ray misses, it projects a point on that ray at the candidate’s depth onto the actual shape. The surface must fall within reach and within the 40-degree half-angle in the [hand settings](../../config/hands.json). Disabled bodies and sensors are ineligible. A knocked-out fighter can still physically obstruct a punch.

Candidates rank by forward alignment, shorter shoulder-to-surface distance, fighter slot, then anatomical part index. A solid between the shoulder and the candidate surface rules it out. The test excludes the attacker’s own rig and the queried target collider. A visible alternative can still win.

## Motion and damage

A surface within one palm radius of the ordinary punch path needs no correction. The ordinary swing also stays intact in a clinch, when the gap between the two pelvis shapes is at most one fist width. This keeps the arm from steering away from an opponent already pressed against the fighter. For a surface farther to either side or above or below, the controller adds up to 0.35 radians in each of shoulder yaw and pitch. The joint’s anatomical limits and the existing punch motor torque cap still apply. The adjustment stays fixed through wind-up and extension and fades during recovery. Holding the button into a grab hands control back to the existing reach logic.

The chosen fighter receives no damage from selection itself. After the physics step, the contact resolver inspects the limb that the fist actually hit. An opponent who gets out of the way can make the punch miss completely.

## Verification

- Native tests cover forward alignment, distance, slot ties, separate hands, blocked targets with a visible alternative, sensors, incapacitated opponents, targets behind the fighter and targets outside the cone or arm span. A paired close-range test keeps the target fixed and checks that bringing the fighters hip-to-hip removes steering.
- In a paired sparring fixture, the same right punch missed with steering disabled and landed one hit with steering enabled. The selected yaw was about 0.24 radians and the fist reached about 8 cm farther sideways. A separate fixture moved the opponent away after selection and checked that its health stayed full.
- The 581-frame native/WASM trace includes an approach, a stop and an assisted left punch against the opponent’s upper arm. The comparison covers hit identity, damage, both physical rigs, grips and stamina.
- The authoritative Skyline room test runs 95 ticks from a normal spawn using encoded movement input, then sends both hand edges. A real left-fist contact damages the other player, and the two recipient snapshots contain the same reduced health.

The release workspace passed 296 native tests, and the web suite passed 59 unit tests. All 14 targeted Chromium scenarios passed, including the 781-tick knockout and get-up, two rematches and an eight-player ring-out. Three combat and multiplayer scenarios also passed on the packaged local preview. The native server and packaged WASM shared engine hash `97703ec79172d87c5a5d93d1463c8839d391d5531bb8cb84a4ad5f7a7117d4ef`.

These are scripted encounters. The width of the cone and the strength of the steering still need assessment in multiplayer playtests.
