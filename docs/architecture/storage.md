# Storage architecture

## Ownership

Storage owns local SQLite schemas, migrations, transactions, and repositories for settings, indexes, and agent state. It does not decide context relevance or product behavior.

It depends on SQLite and versioned domain records. Core and agent repositories may depend on storage abstractions; UI and model training may not access tables directly. Migration, round-trip, concurrency, and recovery tests will use temporary databases.
