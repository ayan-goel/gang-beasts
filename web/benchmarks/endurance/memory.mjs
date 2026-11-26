export function memoryPlateau(samples) {
  if (!samples.length || samples.at(-1).elapsedS < 3600) throw new Error('Incomplete hour of RSS observations');
  const minutes = new Map();
  for (const {elapsedS,rssKiB} of samples) {
    const minute = Math.round(elapsedS/60);
    if (Math.abs(elapsedS - minute*60) > 5 || !Number.isFinite(rssKiB) || rssKiB <= 0) continue;
    minutes.set(minute,rssKiB);
  }
  if (Array.from({length:61},(_,i) => !minutes.has(i)).some(Boolean)) throw new Error('Missing minute samples');
  // Ten-minute medians damp brief allocator spikes. The peak still records every sampled high-water mark.
  const lateMediansKiB = [31,41,51].map(start => {
    const values = Array.from({length:10},(_,offset) => minutes.get(start+offset)).sort((a,b) => a-b);
    return (values[4]+values[5])/2;
  });
  const baselineKiB = lateMediansKiB[0];
  const toleranceKiB = Math.max(1024,baselineKiB*.10);
  return {peakKiB:Math.max(...minutes.values()),lateMediansKiB,toleranceKiB,
    plateau:Math.max(...lateMediansKiB)-Math.min(...lateMediansKiB) <= toleranceKiB};
}
