# examples/python/ — Python Example Games

## Purpose

Python games demonstrating GoudEngine's Python SDK.

## Files

- `main.py` — Basic SDK demo: window creation, rendering, input handling
- `flappy_bird.py` — Flappy Bird clone mirroring the C# `flappy_goud` example
- `run_demo.sh` — Helper script to run Python demos
- `README.md` — Python examples documentation

## Running

```bash
./dev.sh --sdk python --game python_demo    # Run main.py demo
./dev.sh --sdk python --game flappy_bird    # Run Flappy Bird
./examples/python/run_demo.sh               # Direct run via helper
```

## Patterns

- Import from `goud_engine` package (sdks/python/goud_engine/)
- `flappy_bird.py` mirrors C# `flappy_goud` for SDK parity validation
- Examples use snake_case (Python convention)
- Keep examples readable — they serve as SDK documentation

## Anti-Patterns

- NEVER import from engine internals — use the public `goud_engine` package only
- NEVER add dependencies beyond the standard library and goud_engine
