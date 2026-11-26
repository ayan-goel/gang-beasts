import { execFileSync, spawn } from 'node:child_process';
import { once } from 'node:events';
import { createServer } from 'node:net';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../../',import.meta.url));

export async function startServer() {
  const reservation = createServer();
  reservation.listen(0,'127.0.0.1');
  await once(reservation,'listening');
  const address = '127.0.0.1:' + reservation.address().port;
  const origin = 'http://' + address;
  await new Promise(resolve => reservation.close(resolve));
  const child = spawn(root + 'target/release/brawler-server',[],{cwd:root,
    env:{...process.env,BRAWLER_LISTEN:address,BRAWLER_ORIGIN:origin,BRAWLER_WORKERS:'1'},stdio:['ignore','pipe','inherit']});
  const finished = once(child,'exit');
  const close = async () => {
    if (child.exitCode === null && child.signalCode === null) child.kill('SIGINT');
    await finished;
  };
  try {
    await Promise.race([once(child.stdout,'data'),finished.then(() => { throw new Error('Server exited during startup'); })]);
    const response = await fetch(origin+'/api/maps');
    if (!response.ok) throw new Error('Map catalog unavailable');
    return {origin, identity:await response.json(),close, rssKiB:() => {
      const rss = Number(execFileSync('ps',['-o','rss=','-p',String(child.pid)],{encoding:'utf8'}).trim());
      if (!Number.isFinite(rss) || rss <= 0) throw new Error('Missing server RSS');
      return rss;
    }};
  } catch (error) { await close(); throw error; }
}
