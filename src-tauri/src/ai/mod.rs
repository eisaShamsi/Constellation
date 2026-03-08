use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub model: String,
    #[serde(rename = "tokensUsed")]
    pub tokens_used: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

/// Send a message to an AI provider and return the response.
#[tauri::command]
pub async fn ai_send_message(
    provider: String,
    api_key: String,
    model: String,
    base_url: String,
    messages: Vec<AIMessage>,
    max_tokens: u32,
    temperature: f32,
) -> Result<AIResponse, String> {
    match provider.as_str() {
        "openai" => send_openai(&api_key, &model, &messages, max_tokens, temperature).await,
        "anthropic" => send_anthropic(&api_key, &model, &messages, max_tokens, temperature).await,
        "gemini" => send_gemini(&api_key, &model, &messages, max_tokens, temperature).await,
        "ollama" => send_ollama(&base_url, &model, &messages, max_tokens, temperature).await,
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

/// Validate connection to an AI provider.
#[tauri::command]
pub async fn ai_validate_connection(
    provider: String,
    api_key: String,
    model: String,
    base_url: String,
) -> Result<bool, String> {
    let test_messages = vec![AIMessage {
        role: "user".to_string(),
        content: "Say 'ok' and nothing else.".to_string(),
    }];

    match ai_send_message(provider, api_key, model, base_url, test_messages, 10, 0.0).await {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

/// List available models for a provider.
#[tauri::command]
pub async fn ai_list_models(
    provider: String,
    api_key: String,
    base_url: String,
) -> Result<Vec<ModelInfo>, String> {
    match provider.as_str() {
        "openai" => list_openai_models(&api_key).await,
        "anthropic" => Ok(list_anthropic_models()),
        "gemini" => Ok(list_gemini_models()),
        "ollama" => list_ollama_models(&base_url).await,
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

// ─── OpenAI ──────────────────────────────────────────────

async fn send_openai(
    api_key: &str,
    model: &str,
    messages: &[AIMessage],
    max_tokens: u32,
    temperature: f32,
) -> Result<AIResponse, String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": model,
        "messages": messages.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content
        })).collect::<Vec<_>>(),
        "max_tokens": max_tokens,
        "temperature": temperature
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("OpenAI API error ({}): {}", status, text));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let tokens_used = json["usage"]["total_tokens"].as_u64().map(|t| t as u32);

    Ok(AIResponse {
        content,
        model: model.to_string(),
        tokens_used,
    })
}

async fn list_openai_models(api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        return Ok(vec![
            ModelInfo { id: "gpt-4o".into(), name: "GPT-4o".into() },
            ModelInfo { id: "gpt-4o-mini".into(), name: "GPT-4o Mini".into() },
        ]);
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let models: Vec<ModelInfo> = json["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m["id"].as_str()?.to_string();
                    if id.starts_with("gpt-") {
                        Some(ModelInfo { name: id.clone(), id })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}

// ─── Anthropic (Claude) ─────────────────────────────────

async fn send_anthropic(
    api_key: &str,
    model: &str,
    messages: &[AIMessage],
    max_tokens: u32,
    temperature: f32,
) -> Result<AIResponse, String> {
    let client = reqwest::Client::new();

    // Anthropic uses a separate system parameter
    let system_msg = messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone());

    let user_messages: Vec<serde_json::Value> = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
        .collect();

    let mut body = serde_json::json!({
        "model": model,
        "messages": user_messages,
        "max_tokens": max_tokens,
        "temperature": temperature
    });

    if let Some(sys) = system_msg {
        body["system"] = serde_json::Value::String(sys);
    }

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Anthropic API error ({}): {}", status, text));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let content = json["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let input_tokens = json["usage"]["input_tokens"].as_u64().unwrap_or(0);
    let output_tokens = json["usage"]["output_tokens"].as_u64().unwrap_or(0);

    Ok(AIResponse {
        content,
        model: model.to_string(),
        tokens_used: Some((input_tokens + output_tokens) as u32),
    })
}

fn list_anthropic_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo { id: "claude-opus-4-20250514".into(), name: "Claude Opus 4".into() },
        ModelInfo { id: "claude-sonnet-4-20250514".into(), name: "Claude Sonnet 4".into() },
        ModelInfo { id: "claude-haiku-4-20250506".into(), name: "Claude Haiku 4".into() },
    ]
}

// ─── Google Gemini ──────────────────────────────────────

async fn send_gemini(
    api_key: &str,
    model: &str,
    messages: &[AIMessage],
    max_tokens: u32,
    temperature: f32,
) -> Result<AIResponse, String> {
    let client = reqwest::Client::new();

    let contents: Vec<serde_json::Value> = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| {
            let role = if m.role == "assistant" { "model" } else { "user" };
            serde_json::json!({
                "role": role,
                "parts": [{ "text": m.content }]
            })
        })
        .collect();

    let mut body = serde_json::json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": max_tokens,
            "temperature": temperature
        }
    });

    // Add system instruction if present
    if let Some(sys) = messages.iter().find(|m| m.role == "system") {
        body["systemInstruction"] = serde_json::json!({
            "parts": [{ "text": sys.content }]
        });
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Gemini API error ({}): {}", status, text));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let content = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let tokens_used = json["usageMetadata"]["totalTokenCount"].as_u64().map(|t| t as u32);

    Ok(AIResponse {
        content,
        model: model.to_string(),
        tokens_used,
    })
}

fn list_gemini_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo { id: "gemini-pro".into(), name: "Gemini Pro".into() },
        ModelInfo { id: "gemini-1.5-pro".into(), name: "Gemini 1.5 Pro".into() },
        ModelInfo { id: "gemini-1.5-flash".into(), name: "Gemini 1.5 Flash".into() },
    ]
}

// ─── Ollama (Local) ─────────────────────────────────────

async fn send_ollama(
    base_url: &str,
    model: &str,
    messages: &[AIMessage],
    _max_tokens: u32,
    temperature: f32,
) -> Result<AIResponse, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": messages.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content
        })).collect::<Vec<_>>(),
        "stream": false,
        "options": {
            "temperature": temperature
        }
    });

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}. Is Ollama running?", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Ollama error ({}): {}", status, text));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let content = json["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(AIResponse {
        content,
        model: model.to_string(),
        tokens_used: None,
    })
}

async fn list_ollama_models(base_url: &str) -> Result<Vec<ModelInfo>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}. Is Ollama running?", e))?;

    if !resp.status().is_success() {
        return Ok(vec![]);
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let models: Vec<ModelInfo> = json["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let name = m["name"].as_str()?.to_string();
                    Some(ModelInfo { id: name.clone(), name })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}
