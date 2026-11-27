/// <reference types="vitest/config" />
// Le paquet aria-interface ne se compile pas seul : chaque appli le compile avec son interface. Cette configuration
// ne sert qu'à ses tests (vitest) et à svelte-check.
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [svelte()],
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
  },
});
