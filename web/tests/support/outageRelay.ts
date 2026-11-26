import { createConnection, createServer, type AddressInfo, type Socket } from 'node:net';

// Keep the original TCP connection open while blackholing one direction.
// Only that connection is impaired, so a reconnect gets the same path as a recovered network.
export async function outageRelay(origin: string) {
  const target = new URL(origin);
  const sockets = new Set<Socket>();
  let cut = (_direction: 'up' | 'down') => {};
  const server = createServer({allowHalfOpen: true}, browser => {
    const backend = createConnection({host:target.hostname, port:Number(target.port), allowHalfOpen:true});
    let outage: 'up' | 'down' | undefined;
    cut = direction => { outage = direction; };
    const forward = (source: Socket, sink: Socket, direction: 'up' | 'down') => {
      sockets.add(source);
      source.on('data', (data: Buffer) => {
        if (outage !== direction && !sink.destroyed && !sink.write(data)) source.pause();
      });
      sink.on('drain', () => source.resume());
      source.on('end', () => { if (outage !== direction) sink.end(); });
      source.on('error', () => source.destroy());
      source.on('close', () => {
        sockets.delete(source);
        if (outage !== direction) sink.destroy();
      });
    };
    forward(browser, backend, 'up');
    forward(backend, browser, 'down');
  });
  await new Promise<void>((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  return {
    port:(server.address() as AddressInfo).port,
    cut:(direction: 'up' | 'down') => cut(direction),
    close:() => {
      for (const socket of sockets) socket.destroy();
      return new Promise<void>(resolve => server.close(() => resolve()));
    },
  };
}
