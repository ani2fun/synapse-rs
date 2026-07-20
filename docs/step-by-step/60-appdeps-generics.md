# Step 60 — AppDeps learns the ports it already had

*(DIP at the wiring boundary: the composition root depends on the three fakeable ports, and
three ITs stop assembling their own routers.)*

## The ask

Item 4 of the deepening loop. `AppDeps` pinned two concrete Postgres adapters
(`allowlist`, `views`) and one concrete tutor client — so the three ITs that fake those
ports (`admin_allowlist_it`, `admin_lesson_views_it`, `tutor_routes_it`) could not use
`app()` at all and each assembled its own sub-router. The route handlers were already
generic over the ports; the wiring struct was the one place the abstraction died early.
"The interface is the test surface" — and the test surface fell short of the composition
in exactly one spot.

## The shape

```rust
pub struct AppDeps<
    L = PostgresSubmissionAllowlist,
    V = PostgresLessonViews,
    C = OllamaTutorClient,
> where L: SubmissionAllowlist + 'static, V: LessonViewStore + 'static, C: TutorClient + 'static
{ … allowlist: Arc<L>, views: Arc<V>, tutor: TutorRoutesState<C>, … }

pub fn app<L, V, C>(deps: AppDeps<L, V, C>) -> Router where …
```

Static dispatch, no `dyn` (house rule); the **default type parameters are the production
adapters**, so `main.rs` and the common IT helper changed by ZERO lines — inference from
the field values covers construction, and a bare `AppDeps` still means the shipping graph.

`submit: Arc<LiveSubmitSolution>` stays concrete on purpose: its `List` parameter is the
Postgres allowlist, and threading the four-param service's generics through `AppDeps` would
be sprawl for no current test need — the admin router is what the fakes exercise; the submit
gate's own behaviour is pinned by its unit fakes and the gated Postgres IT.

## The fold

`tests/common/mod.rs` grew `app_with_stores(issuer, allowlist, views, tutor)` — the full
`app()` over caller-supplied stores — plus `lazy_allowlist()`/`lazy_views()`/`tutor_off()`
defaults (the lazy-pool constructor deduped into `lazy_pool()`). Each IT's local router
builder became a one-call delegation; **every test body and assertion stayed byte-identical,
and all 13 passed unchanged on the first run** — the full layer stack (security headers,
compression, limits, telemetry) disturbed nothing, which is itself evidence the layers are
transparent to API semantics. A side upgrade for the tutor IT: its structural-404 pin now
proves the disabled chat route is absent from the WHOLE router, not just a sub-router that
happened not to mount it.

## Verified

Conventions · fmt · clippy pedantic (clean) · `cargo test --workspace` 458 (13 of them now
crossing the real middleware) · vitest 83 (untouched). No browser pass — this step changes
test wiring and a type signature, nothing the preview can observe; `main.rs` compiling
unchanged against the defaults is the production-safety argument.
