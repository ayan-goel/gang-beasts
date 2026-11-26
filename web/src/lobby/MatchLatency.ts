export class MatchLatency {
  private sequence = 0;
  private sentAt = -Infinity;
  private waiting = false;
  private recent: number[] = [];
  private high = false;

  probe(now: number) {
    // Keep one outstanding sample and retire it if a suspended tab missed its reply.
    if (now - this.sentAt < (this.waiting ? 3000 : 1000)) return null;
    this.sentAt = now;
    this.waiting = true;
    this.sequence = (this.sequence + 1) >>> 0;
    const packet = Uint8Array.of(36,1,0,0,0,0);
    new DataView(packet.buffer).setUint32(2, this.sequence, true);
    return packet;
  }

  reply(packet: Uint8Array, now: number): boolean | null {
    if (packet.length !== 6 || packet[0] !== 37 || packet[1] !== 1) throw new Error('Invalid latency reply');
    const sequence = new DataView(packet.buffer, packet.byteOffset).getUint32(2, true);
    if (!this.waiting || sequence !== this.sequence) return null;
    this.waiting = false;
    const elapsed = now - this.sentAt;
    if (elapsed < 0 || elapsed >= 3000) return null;
    this.recent.push(elapsed);
    if (this.recent.length > 3) this.recent.shift();
    if (this.recent.length === 3) {
      // A three-probe median rejects a single slow frame. Hysteresis avoids flicker near the threshold.
      const median = [...this.recent].sort((a,b) => a-b)[1]!;
      this.high = this.high ? median > 140 : median >= 180;
    }
    return this.high;
  }
}
