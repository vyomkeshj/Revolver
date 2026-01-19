## Revolver TUI Architecture

### High-Level Flow

-- The app runs a single async render loop (`tokio`) that:
  - draws the current screen,
  - reads keyboard input,
  - enqueues `AppEvent`s,
  - drains the event queue to mutate `AppState`,
  - emits protocol messages to the engine via the `Gateway`.

### Screens & Fragments

- **Screens** are top-level views (see `ScreenId`), each implemented as a `Screen` trait with:
  - `draw(...)` for rendering,
  - `handle_key(...)` for mapping key presses to `AppEvent`s.
- **Fragments** are sub-sections within a screen (see `FragmentId`), used to:
  - scope focus and list selection,
  - drive conditional input behavior,
  - control which actions are valid at a time.
- Fragments do not call each other directly; they read shared state and respond to events.

### Event-Driven Core (Serializable)

- All interactions are expressed as **`AppEvent`** with screen-specific payloads:
  - Enqueued by screens/fragments on keypress.
  - Drained by the main loop via `AppState::apply_event`.
  - Designed to be serializable for future replay and remote event sources.
- **Text editing** reuses `TextEditEvent` for cursor movement, typing, and deletion.
-- `apply_event` returns `EventResult`:
  - `quit` flag for shutdown,
  - optional `UiToEngine` command for the engine.

### Protocol Boundary

- **Protocol** uses serializable messages:
  - `UiToEngine` for user intents,
  - `EngineToUi` for task updates.
- **Gateway** owns the channels and bridges UI ↔ engine.

### Data Ownership & Sharing

- **`AppState` is the single source of truth** for UI state:
  - current screen/fragment,
  - selection indices,
  - in-progress task draft,
  - cursor position + blink state,
  - task list snapshots and logs.
- Screens/fragments only mutate via `AppEvent`s, so state changes are replayable.

### Scheduler Boundary

- Background processing is isolated behind `SchedulerCommand` and `TaskUpdate`:
  - UI emits `SchedulerCommand` (e.g., add/cancel task).
  - Scheduler sends `TaskUpdate` to keep UI state in sync.
  - This boundary is designed to be swapped with a remote event server.

### Future Server Decoupling

- The design assumes a future external event server:
  - UI will consume serialized events (replayable).
  - Scheduler can move out-of-process and stream `TaskUpdate` back.
  - The UI remains event-driven with minimal changes.

## Key Files

- `src/main.rs`: terminal setup, splash, event loop, key handling.
- `src/screens/`: screen modules and fragment folders with key bindings.
- `src/ui.rs`: shared UI helpers (splash, borders, formatting).
- `src/app.rs`: `AppState`, `AppEvent`, event queue, state mutations.
- `src/lib.rs`: shared module exports for integration tests.
- `src/engine/scheduler.rs`: task lifecycle, evaluation loop, logging.
- `src/task.rs`: domain models for tasks/hypotheses.
- `src/llm.rs`: LLM interface (mock + optional Rig).
- `src/report.rs`: markdown report generation.

## Feature Notes

- **Splash**: startup shows "TheLastMachine" then enters UI.
- **Task navigation**: `T`/`D`/`I` focus main fragments, `N` opens task input screen.
- **Selection style**: highlighted items use green background with inverted text.
- **Fragment focus**: active fragment uses dashed green border.
- **Cancellation**: `c` cancels selected task.
- **Task input screen**: Task Description fragment on top with Name/Dataset/Heuristics boxes.
- **Heuristics**: list supports `+` to add; images are listed under a heuristic.
- **Heuristics edit**: titles are editable; Right/Left moves into image list for editing.
- **Heuristics add**: `+` adds an image when Images is focused, otherwise adds a heuristic.
- **Empty list UI**: lists show `[Empty List]` placeholder when empty.
- **Cursor**: blinking cursor shown in active text fields and lists; arrows move it.
- **Key dispatch**: screen handlers use non-blocking `try_send` for scheduler commands.
- **LLM test**: integration test loads `.env` for `OPENAI_API_KEY`.
- **Input behavior**: Task name accepts all characters; no H/I shortcuts.
- **Task input keymap**: `F1/F2` switch fragments, `Tab` switches fields.

## Update Policy

- When adding a feature or fixing a bug, update this file with a brief note.
