import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  define: {
    isDev: process.env.NODE_ENV !== 'production'
  },
  clearScreen: false
});
