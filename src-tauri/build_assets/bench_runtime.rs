//! MIG-047 Phase 0b Step B — Runtime micro-bench for mistral.rs.
//!
//! Standalone binary (not a unit test) that measures the user-visible
//! latency + throughput characteristics of `LocalProvider` against a
//! real GGUF model on dev hardware. Output is a markdown report block
//! ready to paste into `lab/reports/MIG-047-bench-runtime-{date}.md`.
//!
//! ## Usage
//!
//! ```bash
//! cd src-tauri
//! cargo run --release --bin bench_runtime -- \
//!   --model /path/to/fanar-1-9b-q4km.gguf \
//!   --model-id fanar-1-9b-q4km
//! ```
//!
//! ## What gets measured
//!
//! Three prompts of increasing length (cold-warm-sustained):
//! - **Run 1 — cold**: short Arabic greeting; pays the model-load
//!   cost (~5s warm SSD, mmap-backed). First-token latency includes
//!   the load.
//! - **Run 2 — warm**: a short follow-up; first-token latency now
//!   reflects only the prompt-processing cost.
//! - **Run 3 — sustained**: 200-word essay request; reveals
//!   sustained tokens-per-second.
//!
//! Plus: resident memory after three turns (RSS on Linux; 0 on
//! macOS/Windows until platform-specific code lands).
//!
//! ## Why not a unit test
//!
//! Loading a multi-GiB GGUF is not unit-test friendly (not in git;
//! load time exceeds default test timeout; results need human
//! interpretation). The Architect §5 Step B verify clause is
//! explicitly "Run on dev hardware; record results in
//! lab/reports/MIG-047-bench-runtime-{date}.md."

use std::path::PathBuf;
use std::time::Instant;

use constellation_lib::mind::events::StreamEvent;
use constellation_lib::mind::provider::{ChatMessage, ChatRole, GenParams, InferenceProvider};
use constellation_lib::mind::providers::LocalProvider;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (model_path, model_id) = parse_args()?;

    println!("# MIG-047 Phase 0b — Runtime Micro-Bench");
    println!();
    println!("- **Model:** `{model_id}`");
    println!("- **Path:** `{}`", model_path.display());
    println!(
        "- **Runtime:** mistral.rs {} (CPU-only, no PagedAttention)",
        env!("CARGO_PKG_VERSION")
    );
    println!("- **Date:** {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    println!();

    let provider = LocalProvider::new(model_path.clone(), model_id.clone());

    println!("## Run 1 — first-token latency (COLD, includes model load)");
    let outcome1 = run_bench(
        &provider,
        "مرحبا، كيف حالك؟",
        128,
    )
    .await?;
    print_outcome(&outcome1);

    println!("## Run 2 — first-token latency (warm)");
    let outcome2 = run_bench(
        &provider,
        "Tell me a short three-sentence story about an astronomer.",
        128,
    )
    .await?;
    print_outcome(&outcome2);

    println!("## Run 3 — sustained throughput");
    let outcome3 = run_bench(
        &provider,
        "Write a 200-word essay about Arabic coffee culture.",
        256,
    )
    .await?;
    print_outcome(&outcome3);

    println!("## Memory");
    let rss_mb = get_rss_mb();
    if rss_mb > 0 {
        println!("- **Resident set size (after 3 turns):** {rss_mb} MiB");
    } else {
        println!(
            "- Resident set size: not reported on this platform \
             (Linux only via /proc/self/status). Use Task Manager / Activity Monitor."
        );
    }
    println!();

    println!("## Summary");
    println!(
        "| Metric | Run 1 (cold) | Run 2 (warm) | Run 3 (sustained) |"
    );
    println!("|---|---|---|---|");
    println!(
        "| First-token latency (ms) | {} | {} | {} |",
        outcome1.first_token_ms, outcome2.first_token_ms, outcome3.first_token_ms
    );
    println!(
        "| Total time (ms) | {} | {} | {} |",
        outcome1.total_ms, outcome2.total_ms, outcome3.total_ms
    );
    println!(
        "| Tokens emitted | {} | {} | {} |",
        outcome1.tokens, outcome2.tokens, outcome3.tokens
    );
    println!(
        "| Throughput (tok/s) | {:.1} | {:.1} | {:.1} |",
        throughput(&outcome1),
        throughput(&outcome2),
        throughput(&outcome3)
    );
    println!();

    // Phase 0b Boss-test Stage 0 verify clause: "coherent Arabic response
    // within 5s on standard laptop." Surface a verdict.
    let stage_0_pass = outcome1.first_token_ms <= 5000;
    println!(
        "**Boss-test Stage 0 verify (Arabic greeting → response within 5s):** {}",
        if stage_0_pass {
            "✅ PASS"
        } else {
            "⚠️ FAIL — first-token >5s; investigate model-load + warmup"
        }
    );

    Ok(())
}

fn parse_args() -> Result<(PathBuf, String), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut model_path: Option<PathBuf> = None;
    let mut model_id: String = "unknown-model".into();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--model" => {
                model_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--model-id" => {
                model_id = args[i + 1].clone();
                i += 2;
            }
            "--help" | "-h" => {
                eprintln!(
                    "Usage: cargo run --release --bin bench_runtime -- \\\n  \
                     --model /path/to/foo.gguf [--model-id name]"
                );
                std::process::exit(0);
            }
            other => return Err(format!("Unknown arg: {other}").into()),
        }
    }
    let model_path = model_path.ok_or("Need --model /path/to/foo.gguf (try --help)")?;
    if !model_path.exists() {
        return Err(format!("Model file not found: {}", model_path.display()).into());
    }
    Ok((model_path, model_id))
}

struct Outcome {
    first_token_ms: u128,
    total_ms: u128,
    tokens: u32,
    preview: String,
}

async fn run_bench(
    provider: &LocalProvider,
    prompt: &str,
    max_tokens: u32,
) -> Result<Outcome, Box<dyn std::error::Error>> {
    let messages = vec![ChatMessage {
        role: ChatRole::User,
        content: prompt.to_string(),
        tool_call_id: None,
        tool_name: None,
    }];
    let params = GenParams {
        max_tokens,
        ..GenParams::default()
    };

    let start = Instant::now();
    let mut rx: tokio::sync::mpsc::Receiver<StreamEvent> =
        provider.generate(&messages, &params).await?;
    let mut first_token_ms: Option<u128> = None;
    let mut text = String::new();
    let mut tokens = 0u32;

    while let Some(ev) = rx.recv().await {
        match ev {
            StreamEvent::Token { text: t } => {
                if first_token_ms.is_none() {
                    first_token_ms = Some(start.elapsed().as_millis());
                }
                text.push_str(&t);
                tokens += 1;
            }
            StreamEvent::Done { .. } => break,
            StreamEvent::Error { message } => {
                return Err(format!("provider error: {message}").into());
            }
            _ => {}
        }
    }

    let total_ms = start.elapsed().as_millis();
    let preview: String = text
        .chars()
        .take(80)
        .collect::<String>()
        .replace('\n', " ");
    Ok(Outcome {
        first_token_ms: first_token_ms.unwrap_or(0),
        total_ms,
        tokens,
        preview,
    })
}

fn throughput(o: &Outcome) -> f64 {
    if o.total_ms == 0 {
        return 0.0;
    }
    (o.tokens as f64 * 1000.0) / o.total_ms as f64
}

fn print_outcome(o: &Outcome) {
    println!("- **First-token latency:** {} ms", o.first_token_ms);
    println!("- **Total time:** {} ms ({} tokens)", o.total_ms, o.tokens);
    println!("- **Throughput:** {:.1} tok/s", throughput(o));
    println!("- **Preview:** `{}`", o.preview);
    println!();
}

#[cfg(target_os = "linux")]
fn get_rss_mb() -> u64 {
    if let Ok(s) = std::fs::read_to_string("/proc/self/status") {
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                if let Some(kb_str) = rest.split_whitespace().next() {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        return kb / 1024;
                    }
                }
            }
        }
    }
    0
}

#[cfg(not(target_os = "linux"))]
fn get_rss_mb() -> u64 {
    // macOS / Windows: would need platform-specific calls (Mach
    // task_info / GetProcessMemoryInfo). Out of scope for Phase 0b;
    // operator reads from Task Manager / Activity Monitor.
    0
}
