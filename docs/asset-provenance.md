# Playable content provenance

| Content | Authoring source | Current use |
| --- | --- | --- |
| Skyline footprint, spawn layout, crates, ventilation units, breakaway walkways and schedule | [Original map manifest](../assets/maps/skyline/map.json) | Shared collision and gameplay data |
| Rooftop trim, vent grilles, crate bands, exterior buildings and materials | [Original procedural scene](../web/src/rendering/SkylineView.ts) | Geometry generated locally at scene creation |
| Coral and Blue diagnostic fighter geometry | [Rig dimensions](../config/character.json) and [procedural view](../web/src/rendering/RagdollView.ts) | Local physical silhouettes and team colors |
| Match interface and rooftop mark | [Game screen](../web/src/app/SkylineGame.tsx) and [styles](../web/src/app/skyline.css) | Local start, scoreboard, pause and rematch |
| Skyline lobby image | [WebP capture](../web/src/lobby/skyline.webp) of the project’s own rendered Skyline arena | The room page presents the map without loading the game renderer |

These entries use geometry, colors and layouts authored in this repository. Three.js supplies the MIT-licensed rendering library and geometry primitives. The local presentation uses system-installed fonts. Original skinned fighters, final mesh assets and core audio are outstanding release work.
