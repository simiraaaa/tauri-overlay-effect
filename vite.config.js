import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5173,
    strictPort: true
  },
  define: {
    isDev: process.env.NODE_ENV !== 'production'
  },
  clearScreen: false
});
