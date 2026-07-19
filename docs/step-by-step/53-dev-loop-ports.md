# Step 53 — Ports, a hang, and a warning that cannot be fixed

*(three rough edges in the build loop — one papercut, one 39-minute CI hang of my own making, and one dead end worth proving dead.)*

## The bind that failed quietly

`dev-tools/dev` ended with this, and it had been there since step 01:

```
Error: Address already in use (os error 48)
```

The script's own comment called it fine: *"If :8280 is already held by a running server, the
fresh bind fails and Vite proxies to the existing one — fine for UI work."*

It is not fine, and the reason is what the failure looks like rather than what it costs. The
server dies; **Vite carries on and proxies to the old binary**. The page loads, the API answers,
everything appears healthy — and it is serving pre-edit code. The one signal that something is
wrong is a single line that scrolled past under the wasm build output.

I lost time to exactly this during step 52: a stale `target/debug/synapse-server` from an earlier
run was still holding :8280, and the first thing the new port-reclaim caught when I tested it was
that very process.

Both ports are fixed by design, which is what makes reclaiming them safe rather than rude. Vite
runs `--strictPort` because the Keycloak dev realm whitelists specific origins — a silent bump to
:5274 is what 403'd the silent-SSO iframe back in step 39. So "in use" here never means a
legitimate neighbour to tiptoe around; it always means a leftover.

`reclaim` lives in `dev-tools/lib-ports.sh`, sourced by both `dev` and `e2e` (which had the same
hole — `dev-tools/e2e` starts a server on the same port). SIGTERM, wait up to five seconds for
the socket, then SIGKILL. It **names each process it kills**:

```
→ :8280 (synapse-server) is held — reclaiming
    kill 51954  target/debug/synapse-server
```

That line is the point. Reaping a stale server of ours is routine; reaping something unexpected
must be visible, and a silent `kill` would make the tool worse than the problem.

## The CI job that hung for 39 minutes

Step 52's `e2e` job went green through nine steps and then sat in `smoke` for 39 minutes while
every other job in the graph finished. I cancelled it. Two defects, and I had shipped both.

**The health poll raced a compile.** The script ran `cargo run -p synapse-server --quiet &` and
then polled `/api/health` for 60 seconds. Locally the binary is always warm, so it starts
instantly; on a cold CI cache `cargo run` has to *build* the server first, which takes minutes.
The poll was counting down against a compile it could never outlast. It now runs
`cargo build` in the foreground and starts the built binary afterwards.

**The step could not end.** That is what turned a failure into a hang. CI runs
`dev-tools/e2e | tee /tmp/e2e.log`, and a background process that inherits the pipe holds its
write end open — so even after the script exits, `tee` never sees EOF and the step waits
forever. `cargo run` makes this worse by spawning children that survive a kill of cargo itself.
The server's output now goes to a file, and the script execs the binary directly so there is
exactly one PID to clean up.

Verified in the shape that actually matters — through a pipe, as CI invokes it:

```
$ dev-tools/e2e 2>&1 | tee /tmp/e2e-ci.log
exit=0  elapsed=8s   7 passed
terminated cleanly
```

**And the reason a defect could burn 39 minutes at all: not one job in `ci.yml` had
`timeout-minutes`.** A hang would have run to GitHub's six-hour default. Every job now carries a
budget — 5 minutes for the grep-only gates, 15 for cargo-deny, 30 for the toolchain jobs. The
`release` job is deliberately without one: it calls a reusable workflow, and GitHub does not
accept `timeout-minutes` on a `uses:` job.

## The warning that goes nowhere

Every build prints:

```
warning: the following packages contain code that will be rejected by a future version of Rust:
  proc-macro-error2 v2.0.1
```

The detail is `E0365`: the crate does `pub use proc_macro;` where the compiler now requires
`pub extern crate proc_macro;` (rust-lang/rust#127909).

I went looking for the fix and there isn't one. Each avenue was checked, not assumed:

- **Upgrade the crate** — `2.0.1` *is* the latest published version.
- **Patch to an unreleased fix** — the upstream repo (`GnomedDev/proc-macro-error-2`) is
  **archived**, last pushed 2024-09-06, and its HEAD still carries the offending line. There is
  nothing to `[patch]` to.
- **Upgrade past it** — it arrives four separate ways, all transitive:
  `leptos_macro`, `leptos_router_macro`, `reactive_stores_macro`, and `syn_derive` via `rstml`.
  Leptos 0.8 is the newest stable; 0.9 is a beta, which is not a thing to put under a production
  reader to silence a build warning. `cargo update` offers nothing.

The remaining option is forking and self-hosting a one-line patch of a build-time proc-macro
crate — taking on a fork to maintain forever, in exchange for a quieter build log. That trade is
not worth making.

So it is recorded instead, in **`rust-toolchain.toml`** — because that is where it will actually
bite. The channel is pinned to `1.97.0` (step 45), which means the build cannot break on its own:
it breaks the day someone raises that number. The note sits next to the version, says the fix is
a Leptos upgrade rather than anything in this repository, and lists what was already ruled out so
nobody re-runs this investigation. `deny.toml` gains a cross-reference, next to the existing note
that scopes `unmaintained` for the same crate and the same reasons.

## What this deliberately does not do

**No `--cap-lints` or suppression flag.** Silencing a future-incompat warning is how a pinned
toolchain becomes a surprise two years later. The warning is correct and should stay loud; what
was missing was an answer to "yes, and?", which is now written down.

**No fork.** See above.

**No process-name matching in `reclaim`.** It could refuse to kill anything that is not
`synapse-server` or `vite` — but on these two fixed ports, anything else squatting is a problem
the developer wants to know about and clear, not something to protect. Printing what it kills is
the safer half of that trade, and it is the half that keeps working when a process name changes.

## Verified

```
occupied :8280 → reclaim → free, and the process it named was a real leftover:
    kill 51954  target/debug/synapse-server

dev-tools/dev, full run:  server :8280 → 200   vite :5373 → 200
                          no "Address already in use" anywhere in the log
```

And the CI hang, through a pipe exactly as the runner invokes it:

```
dev-tools/e2e 2>&1 | tee  →  exit=0, 8s, 7 passed, pipeline terminated on its own
```

`bash -n` on all three scripts. `cargo build` still clean. No source or test changes, so the
suite is unmoved: 433 rust + 74 vitest + 7 e2e.

## The lesson

**A job with no timeout cannot fail — it can only hang.** The e2e script had two real bugs, and
either would have been a two-minute fix if the step had died at 30 minutes with a log. Instead
the absence of a budget turned a bad script into an open-ended one, and the job that was supposed
to gate the release became the thing blocking it. A timeout is not a safety net for bugs you have
thought of; it is the one that catches the shape you have not.

And the older one, which this step also fixes:

**"Harmless" in a comment is a claim, and this one had been wrong since step 01.** The bind error
really was survivable — Vite really did keep working. What the comment missed is that the
survival mode serves *stale code from a binary you thought you had replaced*, which is a far more
expensive failure than a crash would have been. A loud failure costs a restart; a quiet one costs
you the afternoon and your trust in what you are looking at.
