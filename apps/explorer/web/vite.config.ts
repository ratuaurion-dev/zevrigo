// Build dan server pengembangan antarmuka web explorer (Spesifikasi Explorer Web v0.1 Bagian 1 dan 6).
// Hasil build di dist/ disajikan program explorer lewat --web-dir; `npm run dev` meneruskan /api ke explorer lokal.
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: '/',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: false,
    assetsInlineLimit: 0,
    modulePreload: { polyfill: false },
  },
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
    proxy: { '/api': 'http://127.0.0.1:28802' },
  },
});
