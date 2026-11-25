import type { Playground, PlaygroundSnapshot } from '../game/Playground';

export interface InspectorProps {
  state: PlaygroundSnapshot;
  act: (action: (playground: Playground) => void) => void;
}
