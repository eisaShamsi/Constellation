//! MIG-047 Phase 0b Step H — Tool-use reliability benchmark.
//!
//! Runs a prompt suite through `LocalProvider` with the Phase 1 tool
//! palette (search_notes, read_note, find_similar, list_recent) and
//! scores three axes per prompt:
//!
//! 1. **Tool-call validity** — did the model emit a structurally-valid
//!    `ToolCall` with parseable JSON args for the right tool name?
//! 2. **Argument fidelity** — do the args match a graded "expected"
//!    args structure for the prompt?
//! 3. **Coherent reply after tool result** — does the round-2 response
//!    synthesize the (canned) tool result meaningfully?
//!
//! Output: markdown report with per-prompt scores + aggregate pass-rate.
//!
//! ## Usage
//!
//! ```bash
//! cd src-tauri
//! cargo run --release --bin bench_tool_use -- \
//!   --model /path/to/fanar-1-9b-q4km.gguf \
//!   --model-id fanar-1-9b-q4km
//! ```
//!
//! ## Phase 0b scope vs Phase 2.5 scope
//!
//! Phase 0b ships **Fanar only** (per Eisa's §4 A lock A4). The bench
//! here measures Fanar's tool-call reliability against the read-tool
//! palette. Phase 2.5 (MIG-050) re-runs this bench against both
//! Fanar AND Jais to produce the comparative bench Plan §1 Decision #2
//! originally promised.
//!
//! ## Prompt set
//!
//! Ships with **10 starter prompts** demonstrating the methodology
//! across three categories (search, read, multi-step). Architect §5
//! Step H called for 50 prompts total; Eisa expands the set before
//! running the full bench. The PROMPTS array below is the source of
//! truth — adding more entries is the only edit needed.
//!
//! The third-axis score ("coherent reply") is heuristic in this
//! standalone bench (we don't have a real read_note backend to dispatch
//! against; the bench injects a CANNED tool result and judges whether
//! the model's round-2 reply mentions content from that canned result).
//! Phase 1 (MIG-048) re-runs the bench with the real ToolDispatcher
//! once it lands.

use std::path::PathBuf;

use constellation_lib::mind::events::StreamEvent;
use constellation_lib::mind::provider::{
    ChatMessage, ChatRole, FinishReason, GenParams, InferenceProvider, ToolChoice, ToolSchema,
};
use constellation_lib::mind::providers::LocalProvider;
use serde_json::json;

struct Prompt {
    id: &'static str,
    category: &'static str,
    user_message: &'static str,
    /// The tool name we EXPECT the model to call.
    expected_tool: &'static str,
    /// A keyword from `expected_args` we expect to find in the model's
    /// emitted JSON args (e.g. for a "search for X" prompt, expect "X"
    /// in the query arg).
    expected_arg_keyword: &'static str,
    /// Canned tool result to inject for round-2. Must contain
    /// `canned_keyword` so the coherent-reply heuristic can detect
    /// whether the model's round-2 reply used the result.
    canned_tool_result: &'static str,
    canned_keyword: &'static str,
}

const PROMPTS: &[Prompt] = &[
    // ── Category: search ──────────────────────────────────────────
    Prompt {
        id: "s1",
        category: "search",
        user_message: "Search my notes for everything about Canopus.",
        expected_tool: "search_notes",
        expected_arg_keyword: "canopus",
        canned_tool_result: r#"[{"title":"Suhail / Canopus","path":"astronomy/canopus.md","snippet":"Canopus rises in late August across Arabia"}]"#,
        canned_keyword: "Canopus",
    },
    Prompt {
        id: "s2",
        category: "search",
        user_message: "ابحث في ملاحظاتي عن سهيل.",
        expected_tool: "search_notes",
        expected_arg_keyword: "سهيل",
        canned_tool_result: r#"[{"title":"نجم سهيل","path":"falak/suhail.md","snippet":"يطلع سهيل في أواخر أغسطس"}]"#,
        canned_keyword: "سهيل",
    },
    Prompt {
        id: "s3",
        category: "search",
        user_message: "Find recent notes about PKF.",
        expected_tool: "search_notes",
        expected_arg_keyword: "pkf",
        canned_tool_result: r#"[{"title":"Personal Knowledge Formulation","path":"pkf.md","snippet":"PKF is the verb, not management"}]"#,
        canned_keyword: "PKF",
    },
    // ── Category: read ────────────────────────────────────────────
    Prompt {
        id: "r1",
        category: "read",
        user_message: "Read the note titled 'PKF Overview' and summarize it.",
        expected_tool: "read_note",
        expected_arg_keyword: "pkf",
        canned_tool_result: r#"{"title":"PKF Overview","body":"PKF means Personal Knowledge Formulation — the verb of creating knowledge, not the noun of storing it."}"#,
        canned_keyword: "Formulation",
    },
    Prompt {
        id: "r2",
        category: "read",
        user_message: "Open the note at astronomy/durur.md.",
        expected_tool: "read_note",
        expected_arg_keyword: "durur",
        canned_tool_result: r#"{"title":"Durur Calendar","body":"The Khaleeji Durur calendar divides the year into five seasons starting with the rise of Suhail."}"#,
        canned_keyword: "Durur",
    },
    // ── Category: find_similar ────────────────────────────────────
    Prompt {
        id: "f1",
        category: "find_similar",
        user_message: "What notes are similar to my 'Suhail and the Bedouin calendar' note?",
        expected_tool: "find_similar",
        expected_arg_keyword: "suhail",
        canned_tool_result: r#"[{"title":"Anwa stars","sim":0.82},{"title":"Pleiades myths","sim":0.74}]"#,
        canned_keyword: "Anwa",
    },
    // ── Category: list_recent ─────────────────────────────────────
    Prompt {
        id: "lr1",
        category: "list_recent",
        user_message: "What did I write in the last 7 days?",
        expected_tool: "list_recent",
        expected_arg_keyword: "7",
        canned_tool_result: r#"[{"title":"Coffee culture draft","modified":"2026-05-22"}]"#,
        canned_keyword: "Coffee",
    },
    // ── Category: multi-step (single-turn observable) ─────────────
    Prompt {
        id: "m1",
        category: "multi-step",
        user_message: "Search for 'Suhail' then read the top result.",
        expected_tool: "search_notes",
        expected_arg_keyword: "suhail",
        canned_tool_result: r#"[{"title":"Suhail / Canopus","path":"falak/suhail.md"}]"#,
        canned_keyword: "Suhail",
    },
    Prompt {
        id: "m2",
        category: "multi-step",
        user_message: "أحضر لي أحدث الملاحظات ثم لخص أحدثها.",
        expected_tool: "list_recent",
        expected_arg_keyword: "ملاحظات",
        canned_tool_result: r#"[{"title":"ثقافة القهوة","modified":"2026-05-22"}]"#,
        canned_keyword: "القهوة",
    },
    Prompt {
        id: "m3",
        category: "multi-step",
        user_message: "Find notes similar to my pkf.md note and tell me what they have in common.",
        expected_tool: "find_similar",
        expected_arg_keyword: "pkf",
        canned_tool_result: r#"[{"title":"Knowledge formulation","sim":0.85}]"#,
        canned_keyword: "formulation",
    },
];

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (model_path, model_id) = parse_args()?;

    println!("# MIG-047 Phase 0b — Tool-Use Reliability Bench");
    println!();
    println!("- **Model:** `{model_id}`");
    println!("- **Path:** `{}`", model_path.display());
    println!(
        "- **Date:** {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("- **Prompt count:** {}", PROMPTS.len());
    println!();
    println!(
        "Scoring per prompt: (1) tool-call validity, (2) argument keyword \
         present, (3) coherent reply after canned tool result. Each axis \
         is pass/fail."
    );
    println!();

    let provider = LocalProvider::new(model_path.clone(), model_id.clone());
    let tools = build_tool_palette();

    let mut results: Vec<(String, bool, bool, bool, String)> = Vec::new();

    println!(
        "| ID | Category | Tool call valid | Arg keyword | Coherent reply | Reply preview |"
    );
    println!("|---|---|---|---|---|---|");

    for p in PROMPTS {
        let outcome = score_prompt(&provider, p, &tools).await?;
        println!(
            "| {} | {} | {} | {} | {} | `{}` |",
            p.id,
            p.category,
            if outcome.tool_call_valid { "✅" } else { "❌" },
            if outcome.arg_keyword_present {
                "✅"
            } else {
                "❌"
            },
            if outcome.coherent_reply { "✅" } else { "❌" },
            outcome.preview
        );
        results.push((
            p.id.to_string(),
            outcome.tool_call_valid,
            outcome.arg_keyword_present,
            outcome.coherent_reply,
            outcome.preview,
        ));
    }
    println!();

    let total = results.len();
    let valid = results.iter().filter(|(_, v, _, _, _)| *v).count();
    let arg = results.iter().filter(|(_, _, a, _, _)| *a).count();
    let coh = results.iter().filter(|(_, _, _, c, _)| *c).count();

    println!("## Aggregate scores");
    println!(
        "- **Tool-call validity rate:** {valid}/{total} ({:.0}%)",
        100.0 * valid as f64 / total as f64
    );
    println!(
        "- **Argument keyword rate:** {arg}/{total} ({:.0}%)",
        100.0 * arg as f64 / total as f64
    );
    println!(
        "- **Coherent reply rate:** {coh}/{total} ({:.0}%)",
        100.0 * coh as f64 / total as f64
    );
    println!();
    println!(
        "**Bundled-default recommendation:** {} for Phase 0b (Fanar is \
         the only model in v1 per Eisa's §4 A lock A4; the comparative \
         Fanar-vs-Jais bench moves to Phase 2.5 / MIG-050).",
        model_id
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
                    "Usage: cargo run --release --bin bench_tool_use -- \\\n  \
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

fn build_tool_palette() -> Vec<ToolSchema> {
    vec![
        ToolSchema {
            name: "search_notes".into(),
            description: "Search the user's notes by a query string. Returns top matches with title, path, and snippet.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Free-text search query" },
                    "limit": { "type": "integer", "description": "Max results", "default": 10 }
                },
                "required": ["query"]
            }),
        },
        ToolSchema {
            name: "read_note".into(),
            description: "Read the full body of a single note by title or path. Returns title + body.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "identifier": { "type": "string", "description": "Note title OR path" }
                },
                "required": ["identifier"]
            }),
        },
        ToolSchema {
            name: "find_similar".into(),
            description: "Find notes semantically similar to a reference note. Returns ranked list with similarity scores.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Title or path of the reference note" },
                    "limit": { "type": "integer", "default": 5 }
                },
                "required": ["reference"]
            }),
        },
        ToolSchema {
            name: "list_recent".into(),
            description: "List the user's most-recently-modified notes. Optional time window (days).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "days": { "type": "integer", "description": "Number of days back", "default": 7 }
                }
            }),
        },
    ]
}

struct PromptOutcome {
    tool_call_valid: bool,
    arg_keyword_present: bool,
    coherent_reply: bool,
    preview: String,
}

/// Minimal tool-use system prompt for the bench. MIG-048 §D added GBNF
/// grammar constraint, but grammar only ensures tool-call JSON is
/// well-formed IF the model chooses to emit one — it doesn't force the
/// choice. The model needs to KNOW tools exist AND see an example.
/// Phase 1 §F builds the canonical Arabic-first system prompt; here
/// we use an aggressive few-shot bench stand-in to test whether
/// Fanar will follow tool-use instructions at all.
fn bench_system_prompt(tools: &[ToolSchema]) -> String {
    let tool_lines: Vec<String> = tools
        .iter()
        .map(|t| format!("- {}: {}", t.name, t.description))
        .collect();
    format!(
        "You are a knowledge assistant. You have access to these tools to read the user's notes:\n\
         {tool_lines}\n\n\
         IMPORTANT: When the user asks about their notes, you MUST call a tool. \
         Do not answer from your own knowledge. The user's notes contain information you cannot \
         access without using a tool.\n\n\
         To call a tool, respond with ONLY a single JSON object on its own, no prose around it. \
         Format: {{\"tool\":\"<name>\",\"args\":{{<arguments>}}}}\n\n\
         Example 1:\n\
         User: Show me what I wrote about machine learning.\n\
         Assistant: {{\"tool\":\"search_notes\",\"args\":{{\"query\":\"machine learning\"}}}}\n\n\
         Example 2:\n\
         User: What's in the file at /home/user/notes/project.md?\n\
         Assistant: {{\"tool\":\"read_note\",\"args\":{{\"path\":\"/home/user/notes/project.md\"}}}}\n\n\
         Example 3:\n\
         User: Find notes similar to /home/user/notes/topic.md\n\
         Assistant: {{\"tool\":\"find_similar\",\"args\":{{\"path\":\"/home/user/notes/topic.md\"}}}}",
        tool_lines = tool_lines.join("\n")
    )
}

async fn score_prompt(
    provider: &LocalProvider,
    p: &Prompt,
    tools: &[ToolSchema],
) -> Result<PromptOutcome, Box<dyn std::error::Error>> {
    // Round 1: ask the model with tools available, primed by a system prompt.
    let messages_r1 = vec![
        ChatMessage {
            role: ChatRole::System,
            content: bench_system_prompt(tools),
            tool_call_id: None,
            tool_name: None,
        },
        ChatMessage {
            role: ChatRole::User,
            content: p.user_message.into(),
            tool_call_id: None,
            tool_name: None,
        },
    ];
    let params = GenParams {
        max_tokens: 256,
        tools: tools.to_vec(),
        tool_choice: ToolChoice::Auto,
        ..GenParams::default()
    };

    let mut rx: tokio::sync::mpsc::Receiver<StreamEvent> =
        provider.generate(&messages_r1, &params).await?;
    let mut tool_call: Option<(String, String, serde_json::Value)> = None;
    let mut round1_finish: Option<FinishReason> = None;
    let mut round1_text = String::new();
    while let Some(ev) = rx.recv().await {
        match ev {
            StreamEvent::Token { text } => {
                round1_text.push_str(&text);
            }
            StreamEvent::ToolCall { id, name, args } => {
                tool_call = Some((id, name, args));
            }
            StreamEvent::Done { finish_reason, .. } => {
                round1_finish = Some(finish_reason);
                break;
            }
            StreamEvent::Error { message } => {
                return Ok(PromptOutcome {
                    tool_call_valid: false,
                    arg_keyword_present: false,
                    coherent_reply: false,
                    preview: format!("ERR: {message}"),
                });
            }
        }
    }

    let (tool_call_valid, arg_keyword_present, call_id, call_name) = match tool_call {
        Some((id, name, args)) => {
            let valid = name == p.expected_tool;
            let kw_present = json_contains_keyword(&args, p.expected_arg_keyword);
            (valid, kw_present, id, name)
        }
        None => (false, false, String::new(), String::new()),
    };

    // If round 1 didn't tool-call, skip round 2 but capture the
    // prose preview so we can see what Fanar said instead.
    if !tool_call_valid || round1_finish != Some(FinishReason::ToolCall) {
        let preview = if round1_text.trim().is_empty() {
            "(no tool call; empty)".into()
        } else {
            let trimmed = round1_text.trim();
            let n = trimmed.chars().take(80).collect::<String>();
            format!("(prose) {}{}", n, if trimmed.chars().count() > 80 { "…" } else { "" })
        };
        return Ok(PromptOutcome {
            tool_call_valid,
            arg_keyword_present,
            coherent_reply: false,
            preview,
        });
    }

    // Round 2: inject canned tool result, ask for follow-up.
    let messages_r2 = vec![
        ChatMessage {
            role: ChatRole::System,
            content: bench_system_prompt(tools),
            tool_call_id: None,
            tool_name: None,
        },
        ChatMessage {
            role: ChatRole::User,
            content: p.user_message.into(),
            tool_call_id: None,
            tool_name: None,
        },
        ChatMessage {
            role: ChatRole::Tool,
            content: p.canned_tool_result.into(),
            tool_call_id: Some(call_id),
            tool_name: Some(call_name),
        },
    ];

    let mut rx2: tokio::sync::mpsc::Receiver<StreamEvent> =
        provider.generate(&messages_r2, &params).await?;
    let mut round2_text = String::new();
    while let Some(ev) = rx2.recv().await {
        match ev {
            StreamEvent::Token { text } => round2_text.push_str(&text),
            StreamEvent::Done { .. } => break,
            StreamEvent::Error { .. } => break,
            _ => {}
        }
    }

    // Heuristic: did the round-2 reply mention the canned keyword?
    let coherent_reply = round2_text
        .to_lowercase()
        .contains(&p.canned_keyword.to_lowercase());

    let preview: String = round2_text
        .chars()
        .take(60)
        .collect::<String>()
        .replace('\n', " ");

    Ok(PromptOutcome {
        tool_call_valid,
        arg_keyword_present,
        coherent_reply,
        preview,
    })
}

/// Recursively scan a JSON Value's string fields for a substring match
/// (case-insensitive). Used to detect whether the model put the right
/// keyword in the right place without enforcing exact JSON shape.
fn json_contains_keyword(v: &serde_json::Value, keyword: &str) -> bool {
    let needle = keyword.to_lowercase();
    match v {
        serde_json::Value::String(s) => s.to_lowercase().contains(&needle),
        serde_json::Value::Array(arr) => arr.iter().any(|x| json_contains_keyword(x, keyword)),
        serde_json::Value::Object(map) => {
            map.values().any(|x| json_contains_keyword(x, keyword))
        }
        _ => false,
    }
}
