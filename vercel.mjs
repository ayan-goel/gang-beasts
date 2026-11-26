const backend = new URL(process.env.VITE_BACKEND_URL || 'invalid:');
if (backend.protocol !== 'https:' || backend.href !== backend.origin + '/') {
  throw new Error('Set VITE_BACKEND_URL to the public Railway HTTPS origin, without a path.');
}

export const config = {
  framework: null,
  installCommand: 'npm ci --prefix web --include=dev',
  buildCommand: 'bash tools/vercel-build.sh',
  outputDirectory: 'web/dist',
  rewrites: [
    { source: '/:page(play|rooms|playground)', destination: '/:page/index.html' },
    { source: '/api/:path*', destination: backend.origin + '/api/:path*' },
  ],
};
