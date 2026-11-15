import type { NextConfig } from 'next';

// Static export (TSK-M64): `next build` menulis HTML, CSS, JS, dan data RSC (.txt) ke `out/`, disajikan program
// `zevrigo-demo web --web-dir apps/blog/out`. Tanpa server Node.js, tanpa optimasi gambar, tanpa sumber luar.
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
