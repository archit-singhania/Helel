# Storage architecture

## Ownership

Storage owns local SQLite schemas, migrations, transactions, and repositories for settings, indexes, and agent state. It does not decide context relevance or product behavior.

It depends on SQLite and versioned domain records. Core and agent repositories may depend on storage abstractions; UI and model training may not access tables directly. Migration, round-trip, concurrency, and recovery tests will use temporary databases.

Phases 4 and 5 establish versioned, atomic JSON repositories under `.helel` for the code index and agent ledger. This keeps the initial persistence layer dependency-free and portable. The planned SQLite adapter remains the target when query volume, transactional migrations, and concurrent writers require it; callers depend on domain APIs rather than the on-disk representation.
