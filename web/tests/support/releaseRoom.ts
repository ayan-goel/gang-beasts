import {expect, type BrowserContext} from '@playwright/test';

export async function releaseRoom(invite: string | undefined, contexts: BrowserContext[]) {
  if (!invite) return;
  const room = new URL(invite);
  const code = room.searchParams.get('code');
  // An abruptly closed page keeps its seat for 20 seconds and can block the next test's worker.
  const responses = await Promise.all(contexts.map(async context => {
    const cookies = await context.cookies();
    if (!cookies.some(cookie => cookie.name === '__Host-skyline')) return;
    const response = await context.request.post(room.origin + '/api/rooms/' + code + '/leave', {
      headers:{Origin:room.origin}, data:{},
    });
    return response.status();
  }));
  for (const status of responses) {
    if (status !== undefined) expect(status).toBe(204);
  }
}
