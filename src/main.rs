use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use llama_cpp::standard_sampler::StandardSampler;
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};

/// Generate a Conventional Commit message from a git diff using a local GGUF
/// model via llama.cpp (Rust bindings: `llama_cpp` 0.3).
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Generate Conventional Commit messages from a git diff using llama.cpp"
)]
struct Args {
    /// Path to the GGUF model (or set LLAMA_MODEL environment variable)
    #[arg(short, long, env = "LLAMA_MODEL")]
    model: PathBuf,

    /// Maximum tokens to generate
    #[arg(short = 'n', long, default_value_t = 96)]
    max_tokens: usize,

    /// Temperature (kept for future use; default sampler here is deterministic)
    #[arg(short = 't', long, default_value_t = 0.2)]
    temperature: f32,

    /// Top-p nucleus sampling (kept for future use)
    #[arg(long, default_value_t = 0.95)]
    top_p: f32,

    /// Top-k sampling (kept for future use; 0 = disabled)
    #[arg(long, default_value_t = 40)]
    top_k: i32,

    /// Context tokens (increase for longer diffs if you have RAM/VRAM)
    #[arg(short = 'c', long, default_value_t = 4096)]
    context: i32,

    /// Optional input file with a unified git diff. If omitted, reads STDIN.
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// If set, print the prompt that is sent to the model
    #[arg(long)]
    show_prompt: bool,
}

const SYSTEM_INSTRUCTIONS: &str = r#"You are CommitBot, an expert at crafting precise Conventional Commit messages.
Follow these rules strictly:
- Output ONLY the commit message, no preamble or explanation.
- Use Conventional Commits format: type(scope): summary
- Add a brief body if helpful.
- Include "BREAKING CHANGE: ..." if applicable.
- Keep summary to <= 72 chars, present-tense, imperative mood.
- Derive scope from paths in the diff when possible.
"#;

fn read_diff(args: &Args) -> Result<String> {
    let diff = if let Some(path) = &args.input {
        fs::read_to_string(path).with_context(|| format!("reading diff from {}", path.display()))?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };
    Ok(diff)
}

fn build_prompt(diff: &str) -> String {
    format!(
        r#"<|system|>
{system}
<|user|>
Given the following unified git diff, write a single Conventional Commit message.
Do not include code fences. Do not include "Message:" or any explanation.
If the change is trivial (whitespace/comments), reply with "chore: minor housekeeping".

Diff:
```diff
{diff}
            <|assistant|>
"#,
        system = SYSTEM_INSTRUCTIONS,
        diff = diff
    )
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Load the model
    let model = LlamaModel::load_from_file(
        &args.model,
        LlamaParams {
            ..Default::default()
        },
    )
    .context("loading model")?;

    // Create a session (context). Note: n_ctx is u32 in 0.3.x.
    let mut session = model
        .create_session(SessionParams {
            n_ctx: args.context as u32,
            ..Default::default()
        })
        .context("creating session")?;

    // Build prompt
    let diff = read_diff(&args)?;
    let prompt = build_prompt(&diff);
    if args.show_prompt {
        eprintln!("--- PROMPT START ---\n{}\n--- PROMPT END ---", prompt);
    }

    // Feed the prompt to the session
    session.advance_context(prompt).context("feeding prompt")?;

    // Sampler: 0.3.x exposes a default sampler; builder methods vary by patch level.
    // If your crate exposes them, you can apply temperature/top-p/top-k here.
    let sampler = StandardSampler::default();

    // Start completion and stream strings. `start_completing_with` returns Result<CompletionHandle, _>.
    let start = Instant::now();
    let stream_iter = session
        .start_completing_with(sampler, args.max_tokens)?
        .into_strings();

    // Iterate items as String to avoid unsized `str` inference.
    let mut out = String::new();
    for chunk in stream_iter {
        let chunk: String = chunk;
        print!("{}", chunk);
        let _ = io::stdout().flush();
        out.push_str(&chunk);
    }

    eprintln!("\n(generated in {:.2?})", start.elapsed());
    Ok(())
}
