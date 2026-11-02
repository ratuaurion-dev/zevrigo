import type { NextConfig } from 'next';

// Static export (DEC-26): `next build` menulis HTML, JS, CSS, dan data RSC (.txt) ke `out/`, yang disajikan program
// explorer lewat `--web-dir apps/explorer/web/out`. Tanpa server Node.js, tanpa optimasi gambar, tanpa sumber luar.
const nextConfig: NextConfig = {
  output: 'export',
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
  poweredByHeader: false,
  reactStrictMode: true,
};

export default nextConfig;
