import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'
import { visualizer } from 'rollup-plugin-visualizer'
// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss(), visualizer({
    open: true,
    gzipSize: true,
    brotliSize: true,
  })],
  build: {
    cssMinify: true,
  },
})
