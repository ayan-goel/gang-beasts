# Physical headbutts

Q triggers an online fighter or local Coral. M triggers local Blue. The server consumes bit 5 of the existing input record. A press and release between physics ticks produces one attempt. Holding the button leaves the move on its own cooldown, and a blur or disconnected controller discards an unsampled press. A fighter can use either hand during the move.

## Simulation

The [move settings](../../config/headbutt.json) enter the engine build hash. A conscious fighter can start the move during play or sparring while it has at least six stamina. A get-up or active ledge pull owns the upper-body pose and cancels the action. Knockout, elimination and round reset clear it.

The accepted press spends six stamina and allocates an attack ID from the same sequence as the fighter’s punches. Six wind-up ticks move the neck and chest toward positive pitch. Fourteen strike ticks reverse their motor targets. Eighteen recovery ticks blend the joint targets and effort limits back into the ordinary pose. Each joint stays within its anatomical limits and applies internal, paired motor effort. The action adds no forward impulse to the fighter as a whole.

The contact resolver samples pre-solve body velocities, then inspects contacts involving the attacking head after the solver step. A forward-facing contact must carry positive impulse and sufficient closing speed and energy. The resolver records at most one hit per opponent per attack. Punch and headbutt use the same region multipliers and damage cap. Browser hit records distinguish the two actions and return a null hand for a head strike.

## Verification

Native fixtures cover a close frontal hit, a distant miss, a separating wall, a held key, a fresh press at the cooldown boundary, an unaffordable attempt, a canceled tap, knockout, countdown and round reset. A head strike and simultaneous punch receive separate attack IDs. The sparring trace approaches from the normal starting positions, drives the head into a real opponent, and compares both rigs, stamina and hit data in native Rust and browser WASM. The room test feeds a complete press/release through the binary input queue. Browser scenarios exercise both shared-keyboard controls and verify that two live peers receive the same server-owned stamina change.

The release workspace run passed 288 native tests. The web suite passed 59 unit tests, and 11 targeted Chromium scenarios passed, including the 481-frame replay, grabs, knockouts, reconnect and an eight-player ring-out. Both headbutt scenarios also passed against the production preview. The compiled client and server agreed on engine hash `b938ed3bdb4c0bbbb83205bfa009a34507119c3db0c32e926a04ea89f45fac46`.

The subsequent [directional punch slice](08c2-punch-targeting.md) covers targeting for the hands. The move tuning still needs human playtesting.
