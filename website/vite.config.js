import { fileURLToPath, URL } from 'node:url';

import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';
import vueDevTools from 'vite-plugin-vue-devtools';
import { visualizer } from 'rollup-plugin-visualizer';

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
    vueDevTools(),
    visualizer({
      template: 'sunburst', // or sunburst
      open: true,
      gzipSize: true,
      brotliSize: true,
      filename: 'analyse.html', // will be saved in project's root
    }),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  define: { 'process.env': {} },
  server: {
    port: 3000,
  },
  preview: {
    port: 3000,
  },
  build: {
    // rollupOptions: {
    //   output: {
    //     manualChunks: {
    //       'group-user': [
    //         './src/views/HomeView.vue',
    //         './src/views/RegisterView.vue',
    //         './src/views/PhotosView.vue',
    //         './src/views/GameView.vue',
    //       ],
    //     },
    //   },
    // },
  },
  esbuild: {
    drop: ['console', 'debugger'],
  },
});
