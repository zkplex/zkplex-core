import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'
import path from 'path'

export default defineConfig({
  define: {
    // Pass BUILD_ID to the client for cache busting
    'import.meta.env.VITE_BUILD_ID': JSON.stringify(
      process.env.BUILD_ID || new Date().getTime().toString()
    )
  },
  plugins: [
    react(),
    wasm(),
    topLevelAwait(),
    // Custom plugin to prevent WASM caching in dev mode
    {
      name: 'wasm-no-cache',
      configureServer(server) {
        server.middlewares.use((req, res, next) => {
          if (req.url?.endsWith('.wasm') || req.url?.includes('zkplex_core')) {
            res.setHeader('Cache-Control', 'no-store, no-cache, must-revalidate, proxy-revalidate');
            res.setHeader('Pragma', 'no-cache');
            res.setHeader('Expires', '0');
          }
          next();
        });
      }
    }
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      // Support both local development (../pkg) and Docker (/app/pkg)
      'zkplex-wasm': path.resolve(__dirname, process.env.DOCKER ? './pkg/zkplex_core.js' : '../pkg/zkplex_core.js')
    }
  },
  server: {
    fs: {
      allow: [
        // Allow serving files from parent directory (for pkg)
        path.resolve(__dirname, '..'),
        // Default: project root
        path.resolve(__dirname, '.')
      ]
    },
    // Disable all caching
    hmr: {
      overlay: false
    }
  },
  // Force no caching of WASM files
  build: {
    rollupOptions: {
      output: {
        assetFileNames: '[name]-[hash][extname]'
      }
    }
  },
  optimizeDeps: {
    exclude: ['zkplex-wasm']
  }
})