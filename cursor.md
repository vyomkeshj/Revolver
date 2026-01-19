## Revolver TUI Architecture

- **Runtime**: async `tokio` loop driving TUI render + scheduler updates.
- **UI**: `ratatui` + `crossterm` with a header, task list, task detail, input, and controls.
- **Navigation model**: `ScreenId` + `FragmentId`, `Screen` trait, and event queue in `AppState`.
- **Events**: screen handlers enqueue `AppEvent`s; main loop drains and applies.
- **Scheduler**: background task runner with cancel support via `watch` channels.
- **LLM**: `RigLlm` mock by default; real LLM behind `real-llm` feature flag.
- **Reports**: markdown generated per task in `reports/`.

## Key Files

- `src/main.rs`: terminal setup, splash, event loop, key handling.
- `src/screens/`: screen modules and fragment folders with key bindings.
- `src/ui.rs`: shared UI helpers (splash, borders, formatting).
- `src/app.rs`: app state, selection, focus.
- `src/scheduler.rs`: task lifecycle, evaluation loop, logging.
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
