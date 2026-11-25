import type { MatchView } from '../rendering/SkylineView';

export const hazardWarnings: Record<MatchView['hazard'], string> = {
  quiet: '',
  west_warning: 'West walkway failing',
  east_warning: 'East walkway failing',
  gust_warning: 'High winds approaching',
  gust: 'Sudden death · Crosswind',
};
