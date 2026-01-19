# Revolver

A Rust TUI agent that iterates on hypotheses to solve image‑based tasks. Point it at an image
dataset, describe what you know (heuristics as text or images), and let it propose, test, and
refine solutions. The system keeps improving by synthesizing programs from the strongest ideas.

This approach can generalize beyond vision: the same loop can be applied to microcontroller
programming workflows where hypotheses and evaluations are grounded in hardware constraints.

## What It Does

- Accepts tasks with an image dataset and heuristics.
- Generates hypotheses with an LLM (Rig).
- Evaluates hypotheses against the dataset.
- Keeps successful hypotheses, discards weak ones.
- Synthesizes a program and iterates while scores improve.
- Produces a markdown report per task.

## TUI Workflow (v0.1)

- **Main screen** shows tasks, details, and controls.
- **Task input screen** lets you define:
  - Name
  - Dataset folder
  - Heuristics (titles + image paths)
- Editing uses a blinking cursor and list selection.

Key bindings (defaults):

- `n`: open task input
- `t/d/i`: focus tasks / detail / input fragments
- `j/k` or `↑/↓`: move selection (when applicable)
- `c`: cancel task
- `q`: quit

Task input:

- `F1/F2`: switch fragments
- `Tab`: switch fields in Task Description
- `+`: add heuristic or image (depending on focus)
- `←/→`: move between heuristic title and image list
- `Enter`: submit (or add image when image list is focused)
- `Esc`: cancel

## Architecture Overview

- **Event‑driven UI**: screens emit `AppEvent`s, the main loop applies them.
- **Protocol boundary**: `UiToEngine` / `EngineToUi` messages via a gateway.
- **Engine module**: task scheduler + runner isolated in `src/engine/`.
- **Serializable events**: event queue supports future replay and remote server decoupling.

See `cursor.md` for detailed architecture notes.

## Running

```bash
cargo run
```

## LLM Test (optional)

Requires `OPENAI_API_KEY` in `.env`:

```bash
cargo test rig_llm_generation_uses_env_key
```

## Reports

Generated in `reports/` (gitignored).
