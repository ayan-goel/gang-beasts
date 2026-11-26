import { createConnection, createServer, type AddressInfo, type Socket } from 'node:net';

// Schedule by arrival time so the link can carry many packets in flight at once.
export async function delayRelay(origin: string) {
  const target = new URL(origin), sockets = new Set<Socket>();
  let halfRtt = 0, jitter = 0;
  const server = createServer(front => {
    const back = createConnection({host:target.hostname,port:Number(target.port)});
    const forward = (source: Socket, sink: Socket) => {
      const pending: {data: Buffer; due: number}[] = [];
      let timer: ReturnType<typeof setTimeout> | undefined;
      function schedule() {
        if (timer || !pending.length || sink.writableNeedDrain) return;
        timer = setTimeout(() => {
          timer = undefined;
          while (pending.length && pending[0]!.due <= performance.now()) {
            const chunk = pending.shift()!.data;
            queued -= chunk.length;
            if (!sink.write(chunk)) { source.pause(); break; }
          }
          schedule();
        }, Math.max(0, Math.ceil(pending[0]!.due - performance.now())));
      }
      let queued = 0, previous = 0, index = 0;
      sockets.add(source);
      source.setNoDelay();
      source.on('data', (data: Buffer) => {
        queued += data.length;
        if (queued + sink.writableLength > 1024 * 1024) { source.destroy(); return; }
        const now = performance.now();
        // TCP preserves byte order even as the emulated propagation delay decreases.
        const due = previous = Math.max(previous, now + halfRtt + [0,1,0,-1][index++ % 4]! * jitter);
        pending.push({data,due});
        schedule();
      });
      sink.on('drain', () => { source.resume(); schedule(); });
      source.on('error', () => source.destroy());
      source.on('close', () => {
        sockets.delete(source);
        clearTimeout(timer);
        pending.length = 0;
        sink.destroy();
      });
    };
    forward(front, back);
    forward(back, front);
  });
  await new Promise<void>((resolve,reject) => {
    server.once('error',reject);
    server.listen(0,'127.0.0.1',resolve);
  });
  return {
    port:(server.address() as AddressInfo).port,
    profile(rtt: number, variation = 0) { halfRtt = rtt / 2; jitter = variation; },
    async close() {
      for (const socket of sockets) socket.destroy();
      await new Promise<void>(resolve => server.close(() => resolve()));
    },
  };
}
