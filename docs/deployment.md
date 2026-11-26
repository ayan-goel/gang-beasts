# Deploy from GitHub

Use the same GitHub repository and branch. Railway builds from the repository root. Vercel builds the Vite project in `web`. One public Railway service runs the Rust game. Vercel builds the browser game and hosts the invite links.

1. In Railway, select **New Project → Deploy from GitHub repo** and choose this repository. In the service's **Settings → Networking**, select **Generate Domain**. Copy the public HTTPS URL. Railway reads the root Dockerfile and `railway.toml`, supplies `PORT`, and checks `/healthz`.
2. In Vercel, select **Add New → Project**, import the **web** Vite application and keep **Root Directory** set to **web**. Leave the build and output overrides at their defaults. Add **`VITE_BACKEND_URL`** to Production with the Railway HTTPS URL, for example `https://my-game.up.railway.app`. Click **Deploy**. The checked-in configuration installs the web dependencies, builds the pinned Rust/WASM engine with the shared repository sources and publishes all four game pages. New Vercel projects include sources outside the Root Directory by default.
3. In Railway's **Variables**, set **`BRAWLER_ORIGIN`** to the stable production Vercel URL, for example `https://my-game.vercel.app`. Apply the change and deploy. Share that Vercel URL. Use origins without paths. The server accepts the configured frontend origin, so use the stable production URL when inviting players.

| Project | Required variable | Value |
| --- | --- | --- |
| Vercel | `VITE_BACKEND_URL` | Railway public HTTPS origin |
| Railway | `BRAWLER_ORIGIN` | Vercel production HTTPS origin |

The first deployment is ready for friends when two devices can open the same Vercel invite, ready up, start a match and see the same round. The Railway URL at `/healthz` should return `{"status":"ok"}`. Vercel embeds the backend URL at build time, so redeploy Vercel after changing it. The default one-worker, one-replica Railway service supports one active match of up to eight players. Restarting the server ends its in-memory rooms.

## How the connections work

The browser posts room requests to its own Vercel origin. The build generates a Vercel Build Output API route from `VITE_BACKEND_URL` after import. This route forwards those requests to Railway, leaving the anonymous HttpOnly session cookie on the frontend hostname. After joining, the browser requests a one-use upgrade ticket with that cookie, then connects directly to Railway over WSS. Only the non-secret protocol name is echoed in the handshake. Invites always use the frontend hostname. Local same-origin play keeps its original socket authentication. [Protocol](protocol.md#hosted-socket-upgrades)

## Verification

With the project's web dependencies, Rust/WASM tools and Playwright Chromium installed, run from the repository root:

~~~sh
cargo test --locked -p brawler-server
cd web
node --test scripts/vercel-routes.test.mjs
npx playwright test -c playwright.deployment.config.ts
~~~

The split-host test builds the production frontend and runs two browser profiles with different frontend and backend site names on loopback. It checks room creation, reload, invite joining, direct sockets, first-party-only cookies, and a scored round on both clients. Local Chromium checks passed over HTTP/WS and HTTPS/WSS. The production frontend build, server release build and existing same-origin room checks also passed. Cloud builds and provider routing still require the first GitHub deployment and hosted two-device check.

[Railway GitHub services](https://docs.railway.com/services) · [Railway healthchecks](https://docs.railway.com/deployments/healthchecks) · [Vercel build output](https://vercel.com/docs/build-output-api/configuration) · [Vercel monorepo settings](https://vercel.com/docs/monorepos/monorepo-faq) · [Vercel external rewrites](https://vercel.com/docs/routing/rewrites)
