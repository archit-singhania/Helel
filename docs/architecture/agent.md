# Agent architecture

## Ownership

The agent owns task state, context selection, action validation, tool sequencing, observations, retries, and completion criteria. It does not directly access the operating system, render UI, train models, or bypass permission policy.

It depends on core tool interfaces, repository context, model runtime interfaces, and persisted state. Desktop may observe and control it; core tools must not depend on it. Public interfaces will be typed commands, events, and an auditable state machine. Scripted models and fake tools test behavior before a learned model is connected.
