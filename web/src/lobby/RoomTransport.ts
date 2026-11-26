export type LinkState = 'connected' | 'recovering' | 'disconnected';

export class RoomTransport {
  private ticket?: Uint8Array;
  private deadline?: number;
  private deadlineTimer?: ReturnType<typeof setTimeout>;
  private retryTimer?: ReturnType<typeof setTimeout>;
  private delay = 250;
  expired = false;

  constructor(private code: string, private retry: () => void, private status: (state: LinkState) => void) {}

  acceptTicket(packet: Uint8Array) {
    if (packet.length !== 10 || packet[0] !== 31 || packet[1] !== 1) throw new Error('Invalid resume ticket');
    const id = packet.slice(2);
    if (this.ticket && !id.every((byte, index) => byte === this.ticket![index])) throw new Error('Resume ticket changed');
    this.ticket = id;
  }

  connectPacket() {
    const resume = this.deadline !== undefined;
    const packet = new Uint8Array(resume ? 16 : 8);
    packet.set([resume ? 32 : 16, 1]);
    packet.set(new TextEncoder().encode(this.code), 2);
    if (resume) packet.set(this.ticket!, 8);
    return packet;
  }

  closed(inMatch: boolean, code: number) {
    if (this.expired) return;
    if (!this.ticket || (this.deadline === undefined && !inMatch) || code === 4002) {
      this.fail();
      return;
    }
    if (this.deadline === undefined) {
      this.deadline = performance.now() + 20000;
      this.deadlineTimer = setTimeout(() => this.fail(), 20000);
      this.status('recovering');
    }
    this.retryTimer = setTimeout(() => {
      if (performance.now() >= this.deadline!) this.fail();
      else this.retry();
    }, this.delay);
    this.delay = Math.min(2000, this.delay * 2);
  }

  restored() {
    if (this.deadline === undefined) return;
    this.dispose();
    this.status('connected');
  }

  dispose() {
    clearTimeout(this.retryTimer);
    clearTimeout(this.deadlineTimer);
    this.deadline = undefined;
    this.delay = 250;
  }

  private fail() {
    this.dispose();
    this.expired = true;
    this.status('disconnected');
  }
}
