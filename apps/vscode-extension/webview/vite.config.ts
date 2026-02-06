import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';
import { resolve } from 'path';

export default defineConfig({
  plugins: [solidPlugin()],
  resolve: {
    alias: { '@ext': resolve(__dirname, '../src') },
  },
  build: {
    outDir: resolve(__dirname, '../dist/webview'),
    emptyDirOnBuild: true,
    lib: {
      entry: resolve(__dirname, 'src/index.tsx'),
      formats: ['iife'],
      name: 'HeadwindWebview',
      fileName: () => 'index.js',
    },
    cssCodeSplit: false,
    target: 'esnext',
    minify: 'esbuild',
    rollupOptions: {
      output: {
        assetFileNames: 'index[extname]',
      },
    },
  },
});
