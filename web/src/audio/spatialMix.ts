export interface CameraPosition { pan: number; distance: number }

export function distanceGain(distance: number) {
  return 8 / Math.max(8, distance);
}

export function playSpatialBuffer(context: BaseAudioContext, buffer: AudioBuffer, rate: number, level: number,
    position: CameraPosition, destination: AudioNode) {
  const voice = context.createBufferSource();
  const gain = context.createGain();
  const pan = context.createStereoPanner();
  voice.buffer = buffer;
  voice.playbackRate.value = rate;
  gain.gain.value = level * distanceGain(position.distance);
  pan.pan.value = position.pan;
  voice.connect(gain).connect(pan).connect(destination);
  const disconnect = () => { voice.disconnect(); gain.disconnect(); pan.disconnect(); };
  voice.onended = disconnect;
  voice.start();
  return () => { voice.stop(); disconnect(); };
}
