# Wilds — Project Context for Claude Code

## What is Wilds?

**Wilds** is an open-source, terminal-based (TUI) RPG built in Rust. It is a fully story-driven RPG with character creation, an overarching narrative, a spell system, and turn-based combat — all rendered in the terminal using Ratatui.

## Tech Stack

| Crate | Purpose |
|---|---|
| `ratatui` | TUI rendering (widgets, layout, frames) |
| `crossterm` | Terminal input, color, mouse events |
| `tokio` | Async runtime for the event loop |
| `sqlx` + SQLite | Local save file — character state, inventory, progress persistence |
| `serde` | Serializing/deserializing game state |
| `color-eyre` | Error handling |

## Current State

The project is bootstrapped from the Ratatui event-driven async template. The core loop (`app.rs`, `event.rs`, `ui.rs`) is a minimal counter demo — this is the starting point. Nothing game-specific has been built yet.

## Planned Game Systems

- **Character creation** — name, class, stat allocation
- **Overarching story** — narrative campaign with acts/chapters
- **Spell system** — mana, spell types, effects
- **Combat** — turn-based encounters with enemies
- **Inventory** — items, equipment, consumables
- **Persistence** — save/load via SQLite (SQLx)

## Architecture Guidelines

- Keep game systems in separate modules under `src/` (e.g. `src/combat/`, `src/character/`, `src/story/`, `src/spells/`)
- Game state should be serializable with `serde` and persistable via `sqlx`
- UI rendering logic lives in `src/ui/` — keep it separate from game logic
- The `App` struct in `app.rs` is the top-level state container; game state hangs off it
- Async event handling via `tokio` + crossterm event stream; avoid blocking the main loop

## Code Style

- Rust 2024 edition
- Prefer `thiserror` for domain errors if added later
- No `unwrap()` in game logic — propagate errors with `?` and `color-eyre`
- Modules should be cohesive — one system per module tree
