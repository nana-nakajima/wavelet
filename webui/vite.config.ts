import { vitePlugin as remix } from "@remix-run/dev";
import { defineConfig, type Plugin } from "vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import path from "path";

// ---------------------------------------------------------------------------
// Custom plugin to inject COOP/COEP headers on EVERY response (including
// the HTML served by Remix's dev middleware). Vite's `server.headers` only
// applies to static assets, so we need middleware for the HTML document.
// ---------------------------------------------------------------------------
function crossOriginIsolation(): Plugin {
  return {
    name: "cross-origin-isolation",
    configureServer(server) {
      server.middlewares.use((_req, res, next) => {
        res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
        res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
        next();
      });
    },
    configurePreviewServer(server) {
      server.middlewares.use((_req, res, next) => {
        res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
        res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
        next();
      });
    },
  };
}

export default defineConfig({
  plugins: [
    // COOP/COEP headers — must be first so they apply before Remix serves HTML
    crossOriginIsolation(),

    // WASM support — allows direct import of .wasm modules
    wasm(),
    topLevelAwait(),

    remix({
      ssr: false, // SPA mode — all audio/PixiJS is client-only
    }),
  ],

  server: {
    fs: {
      // Allow serving WASM artifacts from the Rust build output
      allow: [".."],
    },
  },

  // ---------------------------------------------------------------------------
  // Optimisation — exclude WASM from Vite's dependency pre-bundling so the
  // raw .wasm binary is served as-is to the AudioWorklet.
  // ---------------------------------------------------------------------------
  optimizeDeps: {
    exclude: ["wavelet-wasm"],
  },

  build: {
    target: "esnext",
  },

  resolve: {
    alias: {
      "~": path.resolve(__dirname, "app"),
    },
  },

  // ---------------------------------------------------------------------------
  // Worker configuration — AudioWorklet scripts are loaded as ES modules.
  // Vite needs to know they should be bundled as workers.
  // ---------------------------------------------------------------------------
  worker: {
    format: "es",
    plugins: () => [wasm(), topLevelAwait()],
  },
});
