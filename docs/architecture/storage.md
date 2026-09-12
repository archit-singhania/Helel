# Storage architecture

## Ownership

Storage owns local SQLite schemas, migrations, transactions, and repositories for settings, indexes, and agent state. It does not decide context relevance or product behavior.

It depends on SQLite and versioned domain records. Core and agent repositories may depend on storage abstractions; UI and model training may not access tables directly. Migration, round-trip, concurrency, and recovery tests will use temporary databases.

Agent sessions remain a versioned atomic JSON ledger under `.helel`. Compiler-grade repository data now uses a bundled SQLite database with indexed symbol/reference names and FTS5 content. Security events use append-only JSON lines with a verified hash chain, MCP server definitions use an atomic local registry, and task rollback data records exact pre-mutation bytes for declared paths.
