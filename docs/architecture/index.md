# Architecture

Helel is split into a React desktop interface, a Rust core and agent runtime, Python model tooling, and versioned contracts between them. It is local and offline by construction.

| Area | Owns | Document |
| --- | --- | --- |
| Desktop | Presentation and user interaction | [desktop](desktop.md) |
| Core | Local application services | [core](core.md) |
| Agent | Deterministic action loop | [agent](agent.md) |
| Model | Tokenization, training, inference | [model](model.md) |
| Storage | Local durable data | [storage](storage.md) |
| Security | Workspace and command boundaries | [security](security.md) |

Dependencies point inward through explicit contracts: desktop to core, agent to core tools and model runtime, and both to storage abstractions. The model never receives direct filesystem or process authority.
