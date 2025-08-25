# quack [![status: WIP](https://img.shields.io/badge/status-WIP-orange)]()

> [!WARNING]
> **Under construction.** This package is not ready for use. Expect breaking changes and missing features.

A tiny Rust CLI that uses **llama.cpp** (via the `llama_cpp` Rust crate) to turn a git **diff** into a **Conventional Commit** message.

## Build

You need a local build of `llama.cpp` available to the `llama_cpp` crate (it links to the C library under the hood).
Make sure you have a GGUF model (e.g., a Q4_K variant) on disk.

```bash
cargo build --release
```

## Usage

Read a diff from stdin:

```bash
git diff | LLAMA_MODEL=./models/7B/your-model.gguf     target/release/llama-commit-bot --max-tokens 96
```

Or from a file:

```bash
LLAMA_MODEL=./models/7B/your-model.gguf     target/release/llama-commit-bot --input ./changes.diff --show-prompt
```

### Tips

- Increase `--context` if diffs are large (and your RAM/VRAM allows).
- For stricter output, set `--temperature 0.0` (greedy). For more creative summaries, raise it a bit (e.g., 0.3–0.7).
- Ensure your GGUF has a chat template or adjust the `build_prompt` function.
