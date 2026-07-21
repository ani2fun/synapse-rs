# ──────────────────────────────────────────────────────────────────────────────
# THE PRODUCTION IMAGE (step 35 → A13; oracle: step-34/ADR-S033) — two stages,
# CONTENT-FREE: SYNAPSE_ROOT points at the git-sync sidecar's volume, so prose
# publishing is a `git push` to synapse-content and the image rebuilds only
# when CODE changes. Since A13 it ships the ASTRO topology (axum + the SSR
# sidecar, two processes) with the old Leptos client kept in-image as an
# env-flip rollback (SYNAPSE_ASTRO_URL — see /app/start.sh). The old client's
# 700 KiB budget gate still runs INSIDE the build — an over-budget rollback
# bundle fails the image; the Astro per-page gate lives in CI's e2e job (A12),
# which needs a live stack the build stage does not have.
# ──────────────────────────────────────────────────────────────────────────────

FROM rust:1-bookworm AS builder

# Node 22 for the Vite build (matches the dev toolchain).
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# binaryen from upstream, PINNED: Debian bookworm ships binaryen 108 (2022), whose
# `wasm-opt -Oz` leaves ~270 KiB gz on the table vs a current release — enough to
# blow the 700 KiB budget gate below. (Found by the gate itself, working as built.)
ARG BINARYEN_VERSION=123
RUN arch="$(uname -m)" \
    && curl -fsSL "https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-${arch}-linux.tar.gz" \
       | tar -xz -C /usr/local --strip-components=1 \
    && wasm-opt --version

RUN rustup target add wasm32-unknown-unknown
# Pinned to Cargo.lock's wasm-bindgen (build-wasm.sh refuses a mismatch).
RUN cargo install wasm-bindgen-cli --version 0.2.126 --locked

# The shipped version, baked into the wasm bundle so the footer names the build the reader is
# actually running. `.dockerignore` excludes `.git`, so build.rs's git fallback CANNOT reach
# this image — the arg is the only path that works here, and the default makes an un-argued
# build say so rather than lie.
ARG SYNAPSE_VERSION=unknown
ENV SYNAPSE_VERSION=$SYNAPSE_VERSION

WORKDIR /build
COPY . .

# The server binary (release), shared by BOTH topologies.
RUN cargo build --release -p synapse-server

# ── THE ASTRO APP (A13) — the default serve, built FIRST ──────────────────────
# Ordered before the old client on purpose: since A03 the old client single-sources
# @markdown/@editor/@auth/@tracer/@diagram from web/src, so its vite build reaches
# into web/ and esbuild must resolve web/tsconfig.json's `extends: astro/tsconfigs/…`
# — which only exists once web/node_modules is installed here. (astro is a prod dep,
# so it survives the prune below and the client build still resolves.)
#
# The viz wasm bundle is gitignored build output the web build imports, so it comes
# first (release profile: the artifact CI's e2e budget caps). binaryen + wasm-bindgen
# are already installed above.
RUN bash dev-tools/build-viz-wasm.sh release

# `npm ci` (full — the astro build needs its devDeps), the SSR + client build, then
# prune to a PROD node_modules IN PLACE. The @astrojs/node standalone server is NOT
# self-contained: its bundle externalises framework deps (preact, unified, shiki, …)
# as bare specifiers that only resolve against node_modules — VERIFIED empirically
# (entry.mjs crashes ERR_MODULE_NOT_FOUND without it). We prune-and-copy rather than
# re-`npm ci` in the runtime so that stage stays offline and the lockfile install is
# the single source of truth.
RUN cd web && npm ci --no-audit --no-fund && npm run build && npm prune --omit=dev

# The OLD Leptos client — the rollback artifact. `npm run build` chains the release
# wasm pipeline (cargo → wasm-bindgen → wasm-opt -Oz) and `vite build`.
RUN cd client && npm ci --no-audit --no-fund && npm run build

# The rollback bundle's gate: gzipped critical path (entry + modulepreloads + app
# wasm) ≤ 700 KiB. Kept because the old client is a shipped, reachable serve.
RUN bash dev-tools/check-bundle-budget.sh client/dist

# Drop the three heaviest deps (~238 MB) from the PROD node_modules that ships: monaco-editor,
# mermaid, @terrastruct/d2 are CLIENT-only lazy islands — already bundled into dist/client's
# hashed chunks and loaded in the browser, never at SSR time. The standalone server's
# externalised imports were enumerated (grep of dist/server's bare specifiers) and name NONE
# of them; the Astro-mode boot check (a lesson page exercises shiki/remark) is what catches a
# future Astro version that starts externalizing one, turning silent bloat into a loud
# ERR_MODULE_NOT_FOUND. MUST run AFTER the client build: it single-sources the @editor/@diagram
# islands from web/src, which import monaco/mermaid/d2 as bare specifiers resolved right here.
RUN rm -rf web/node_modules/monaco-editor web/node_modules/mermaid web/node_modules/@terrastruct

# ──────────────────────────────────────────────────────────────────────────────

FROM debian:bookworm-slim AS runtime

# CA roots for the outbound reqwest clients (Keycloak · go-judge · Ollama), then
# Node 22 for the SSR sidecar (nodesource, same channel as the builder). curl is
# nodesource's own prerequisite. bash is already present in bookworm-slim (start.sh
# needs it for `wait -n`).
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && apt-get purge -y curl && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/synapse-server /app/synapse-server
COPY --from=builder /build/client/dist /app/static

# The Astro app: its SSR dist under /app/web, and the PROD node_modules the
# standalone server resolves its externalised imports against (see the builder note).
# node walks up from /app/web/server/entry.mjs and finds /app/web/node_modules.
COPY --from=builder /build/web/dist /app/web
COPY --from=builder /build/web/node_modules /app/web/node_modules
COPY dev-tools/start.sh /app/start.sh

# The pod runs as a NON-ROOT uid (65532, matching the deployment's runAsUser);
# a+rX makes everything world-readable/traversable and keeps execute only where it
# already was (the binary), then the entrypoint is made explicitly executable — a+rX
# would not add +x to a plain COPY'd file.
RUN chmod -R a+rX /app && chmod 0755 /app/start.sh

ENV STATIC_ROOT=/app/static \
    SYNAPSE_ROOT=/content \
    SYNAPSE_AUTO_RELOAD=false \
    SYNAPSE_PORT=8080 \
    SYNAPSE_ASTRO_URL=http://127.0.0.1:4321

EXPOSE 8080
USER 65532:65532
CMD ["/app/start.sh"]
