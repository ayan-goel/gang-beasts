# Playable content provenance

| Content | Authoring source | Current use |
| --- | --- | --- |
| Skyline footprint, spawn layout, crates, ventilation units, breakaway walkways and schedule | [Original map manifest](../assets/maps/skyline/map.json) | Shared collision and gameplay data |
| Rooftop trim, vent grilles, crate bands, exterior buildings and materials | [Original procedural scene](../web/src/rendering/SkylineView.ts) | Geometry generated locally at scene creation |
| Roof Crew skinned work suit, helmet, headlamp, goggles, gloves and boots | [Original mesh authoring source](../tools/fighter/model.py) and [exported GLB](../assets/fighters/roof-crew.glb) | Local and online Skyline fighters. Skin weights and joint pivots follow the physical rig |
| Coral and Blue diagnostic fighter geometry | [Rig dimensions](../config/character.json) and [procedural view](../web/src/rendering/RagdollView.ts) | Physics playground inspection and sparring |
| Match interface and rooftop mark | [Game screen](../web/src/app/SkylineGame.tsx) and [styles](../web/src/app/skyline.css) | Local start, scoreboard, pause and rematch |
| Main menu rooftop throw | [Original WebP still](../web/src/menu/rooftop-throw.webp) | Two repository Roof Crew fighters in the Skyline simulation, captured ten fixed ticks after releasing a lift at the east edge with a presentation camera. [Menu capture](slices/16k-game-entry.md) |
| Skyline lobby image | [WebP capture](../web/src/lobby/skyline.webp) of the project’s own rendered Skyline arena | The room page presents the map without loading the game renderer |
| Walkway and wind warnings | [Original oscillator scores and envelopes](../web/src/audio/matchTone.ts) | Four runtime-synthesized global cues. [Pulse timings and peak level](slices/17c-hazard-audio.md#original-cues) |
| Grip acquisition, strain and release | [Original filtered oscillator sweeps](../web/src/audio/gripSound.ts) | Three camera-relative cues generated at runtime. [Peak levels and duration](slices/17f-grip-audio.md) |
| Rooftop landings | [Original downward triangle sweep](../web/src/audio/landingSound.ts) | Camera-relative thump scaled by solved upward impulse. [Measured levels and duration](slices/17g-landing-audio.md) |
| Combat impact | [Original synthesized thud](../web/src/audio/impactSound.ts) | A seeded noise transient and downward pitch sweep, generated once per audio context. [Spatialization and level limits](slices/17e-strike-audio.md) |
| Knockout and get-up announcements | [Original condition tone definitions](../web/src/audio/matchTone.ts) | One descending knockout tone and two rising get-up pulses. [Timing and peak level](slices/17d-fighter-condition.md#sound) |
| Round countdown and results | [Original sine sweeps and envelopes](../web/src/audio/matchTone.ts) | Five centered cues for countdown changes, the start of play, round wins, draws and match wins. [Pitch and level](slices/17h-round-audio.md) |
| Skyline arena music | [Original eight-bar score and synthesizer](../web/src/audio/arenaScore.ts) | Looped instrumental for active local and private rounds. [Source levels and mix](slices/17i-arena-mix.md#measurements) |

These entries use geometry, colors and layouts authored in this repository. Three.js supplies the MIT-licensed rendering library and geometry primitives. The local presentation uses system-installed fonts. The fighter model uses original repository geometry and colors. Other core effects and final audiovisual acceptance remain open.
