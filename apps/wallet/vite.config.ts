import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// Tauri (DEC-31): port tetap untuk `devUrl`, hasil build di `dist` (`frontendDist` di src-tauri/tauri.conf.json).
// Tanpa sumber luar: semua kode, gaya, dan ikon dibundel.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    host: '127.0.0.1',
    port: 1420,
    strictPort: true,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'es2022',
  },
});
