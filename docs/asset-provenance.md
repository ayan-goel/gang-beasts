# Playable content provenance

| Content | Authoring source | Current use |
| --- | --- | --- |
| Skyline footprint, spawn layout, crates, ventilation units, breakaway walkways and schedule | [Original map manifest](../assets/maps/skyline/map.json) | Shared collision and gameplay data |
| Rooftop trim, vent grilles, crate bands, exterior buildings and materials | [Original procedural scene](../web/src/rendering/SkylineView.ts) | Geometry generated locally at scene creation |
| Roof Crew skinned work suit, helmet, headlamp, goggles, gloves and boots | [Original mesh authoring source](../tools/fighter/model.py) and [exported GLB](../assets/fighters/roof-crew.glb) | Local and online Skyline fighters. Skin weights and joint pivots follow the physical rig |
| Coral and Blue diagnostic fighter geometry | [Rig dimensions](../config/character.json) and [procedural view](../web/src/rendering/RagdollView.ts) | Physics playground inspection and sparring |
| Match interface and rooftop mark | [Game screen](../web/src/app/SkylineGame.tsx) and [styles](../web/src/app/skyline.css) | Local start, scoreboard, pause and rematch |
| Skyline lobby image | [WebP capture](../web/src/lobby/skyline.webp) of the project’s own rendered Skyline arena | The room page presents the map without loading the game renderer |
| Walkway and wind warnings | [Original oscillator scores and envelopes](../web/src/audio/matchTone.ts) | Four runtime-synthesized global cues. [Pulse timings and peak level](slices/17c-hazard-audio.md#original-cues) |
| Combat impact | [Original synthesized thud](../web/src/audio/impactSound.ts) | A seeded noise transient and downward pitch sweep, generated once per audio context. [Spatialization and level limits](slices/17e-strike-audio.md) |
| Knockout and get-up announcements | [Original condition tone definitions](../web/src/audio/matchTone.ts) | One descending knockout tone and two rising get-up pulses. [Timing and peak level](slices/17d-fighter-condition.md#sound) |

These entries use geometry, colors and layouts authored in this repository. Three.js supplies the MIT-licensed rendering library and geometry primitives. The local presentation uses system-installed fonts. The fighter model uses original repository geometry and colors. Other core effects and final audiovisual acceptance remain open.
