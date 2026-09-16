import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'
import { compression } from 'vite-plugin-compression2';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss(), compression({
    algorithms: ['gzip'],
    threshold: 10240,
    deleteOriginalAssets: true,
    include: /assets\/.*\.(js|css)$/,
  }),],
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    cssMinify: true,
  },
})
