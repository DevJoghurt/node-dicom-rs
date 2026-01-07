import { viteCommonjs } from '@originjs/vite-plugin-commonjs'

export default defineNuxtConfig({
  compatibilityDate: '2024-12-16',
  
  future: {
    compatibilityVersion: 4,
  },
  
  devtools: { enabled: true },
  
  ssr: false, // SPA mode for client-side rendering

  nitro: {
    experimental: {
        database: true
    },
  },
  
  app: {
    head: {
      title: 'PACS SQLite - DICOM Viewer',
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      ],
    },
  },
  
  vite: {
    plugins: [viteCommonjs()],
    optimizeDeps: {
      include: [
        'dicom-parser',
        '@cornerstonejs/core',
        '@cornerstonejs/tools',
      ],
      exclude: [
        '@cornerstonejs/dicom-image-loader'
      ],
    },
    worker: {
      format: 'es'
    }
  },
})
