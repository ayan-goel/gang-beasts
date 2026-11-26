import { SkylineSound } from './audio/SkylineSound';
import { SkylineGame } from './app/SkylineGame';
import { mountSimulation } from './app/mountSimulation';

const dispose = mountSimulation(<SkylineSound><SkylineGame /></SkylineSound>);
if (import.meta.hot) import.meta.hot.dispose(dispose);
