import configuration from '../generated/physics.json?raw';

export function exportConfiguration(): void {
  const url = URL.createObjectURL(new Blob([configuration], { type: 'application/json' }));
  const link = document.createElement('a');
  link.href = url;
  link.download = 'physics-v1.json';
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}
