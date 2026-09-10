# P01 — Desktop shell

## Goal

Turn the Phase 0 launch surface into the first usable application shell.

## Delivered

1. Application layout and accessible panel landmarks
2. Activity bar, sidebar, agent panel, bottom panel, and bounded resizing
3. Versioned local settings persistence with validation
4. Native project picker and eight-item recent-project history
5. Dark, light, and operating-system themes
6. Shell, settings, recent-project, frontend-build, and native-build tests

The selected project path is remembered locally, but its files are not accessed in Phase 1.

## Validation

`python3 scripts/verify.py`
