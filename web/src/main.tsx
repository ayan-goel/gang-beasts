import { App } from './app/App';
import { mountSimulation } from './app/mountSimulation';

const dispose = mountSimulation(<App />);
if (import.meta.hot) import.meta.hot.dispose(dispose);
