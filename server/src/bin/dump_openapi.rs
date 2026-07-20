//! Dumps the code-first OpenAPI document (`synapse_server::ApiDoc`) as pretty JSON to stdout —
//! the source `dev-tools/gen-api-types.sh` feeds into `openapi-typescript` to produce
//! `web/src/lib/api/schema.gen.ts`. A dedicated binary rather than a server flag: generating the
//! document needs no bound port, no config, no Postgres — just the `#[derive(OpenApi)]` macro
//! output.

use utoipa::OpenApi;

fn main() -> anyhow::Result<()> {
    println!("{}", synapse_server::ApiDoc::openapi().to_pretty_json()?);
    Ok(())
}
