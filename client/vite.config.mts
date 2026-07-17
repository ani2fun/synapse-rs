import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// ─────────────────────────────────────────────────────────────────────────────
// VITE — dev server :5373, /api proxied to the server :8280. synapse-rs owns
// its OWN port pair (the Scala oracle holds 5273/8180) so both dev loops run
// side by side; strictPort keeps the origin STABLE — a silent bump to :5374
// would fall outside the Keycloak dev client's registered origins and break
// the silent-SSO iframe with a 403. The @alias map points at the TS islands;
// the wasm-bindgen glue's `import … from "@markdown/loader"` resolves through
// it, and each loader's dynamic import gives the heavy renderer its own
// chunk. vitest runs the island suites.
// ─────────────────────────────────────────────────────────────────────────────

export default defineConfig({
  resolve: {
    alias: {
      "@markdown": fileURLToPath(new URL("./islands/markdown", import.meta.url)),
      "@editor": fileURLToPath(new URL("./islands/editor", import.meta.url)),
      "@auth": fileURLToPath(new URL("./islands/auth", import.meta.url)),
      "@tracer": fileURLToPath(new URL("./islands/tracer", import.meta.url)),
      "@diagram": fileURLToPath(new URL("./islands/diagram", import.meta.url)),
    },
  },
  server: {
    port: 5373,
    strictPort: true,
    proxy: {
      "/api": "http://localhost:8280",
      // LikeC4 lesson embeds (<iframe src="/c4/view/…">) ride the server's proxy.
      "/c4": "http://localhost:8280",
      "/media": "http://localhost:8280",
    },
  },
  build: {
    target: "esnext",
  },
  test: {
    include: ["islands/**/*.test.ts"],
  },
});
