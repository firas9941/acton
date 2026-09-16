import path from "node:path"

import react from "@vitejs/plugin-react"
import {defineConfig, loadEnv} from "vite"

import {gzipEmbeddedAssets} from "../ui/vite/embeddedAssets.ts"
import {themeBootstrap} from "../ui/vite/themeBootstrap.ts"

const outputDirectory = path.resolve(import.meta.dirname, "dist")

export default defineConfig(({mode}) => {
  const proxyTarget =
    loadEnv(mode, import.meta.dirname).VITE_BACKEND_PROXY_TARGET?.trim() || "http://127.0.0.1:18007"

  return {
    plugins: [
      themeBootstrap({storageKey: "localton-observability-theme"}),
      react(),
      gzipEmbeddedAssets(outputDirectory),
    ],
    build: {
      outDir: outputDirectory,
      emptyOutDir: true,
    },
    server: {
      host: "127.0.0.1",
      port: 3017,
      proxy: {
        "/api": proxyTarget,
        "/healthz": proxyTarget,
      },
    },
    preview: {
      port: 3017,
    },
  }
})
