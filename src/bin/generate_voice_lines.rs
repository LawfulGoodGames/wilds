use color_eyre::eyre::{Context, eyre};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::json;
use std::env;
use std::path::PathBuf;
use tokio::fs;

#[path = "../town_dialogue.rs"]
mod town_dialogue;

const DEEPGRAM_SPEAK_URL: &str = "https://api.deepgram.com/v1/speak";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _ = dotenvy::dotenv();

    let api_key =
        env::var("DEEPGRAM_API_KEY").wrap_err("missing DEEPGRAM_API_KEY environment variable")?;
    let args = env::args().skip(1).collect::<Vec<_>>();
    let force = args.iter().any(|arg| arg == "--force");
    let missing_only = args.iter().any(|arg| arg == "--missing-only");

    if force && missing_only {
        return Err(eyre!("use either --force or --missing-only, not both"));
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Token {api_key}"))
            .wrap_err("invalid DEEPGRAM_API_KEY for authorization header")?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .wrap_err("failed to build HTTP client")?;

    let mut created = 0usize;
    let mut skipped = 0usize;

    for line in town_dialogue::all_voice_lines() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/audio")
            .join(line.filename);
        if (missing_only || !force) && path.exists() {
            skipped += 1;
            println!("skip {} -> {}", line.id, path.display());
            continue;
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let response = client
            .post(DEEPGRAM_SPEAK_URL)
            .query(&[
                ("model", line.model),
                ("encoding", "linear16"),
                ("container", "wav"),
                ("sample_rate", "24000"),
            ])
            .json(&json!({ "text": line.text }))
            .send()
            .await
            .with_context(|| format!("deepgram request failed for {}", line.id))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable error body>".to_string());
            return Err(eyre!(
                "deepgram request failed for {} with status {}: {}",
                line.id,
                status,
                body
            ));
        }

        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("failed to read audio bytes for {}", line.id))?;

        fs::write(&path, &bytes)
            .await
            .with_context(|| format!("failed to write {}", path.display()))?;

        created += 1;
        println!("saved {} -> {}", line.id, path.display());
    }

    println!(
        "done: created {} file(s), skipped {} existing file(s)",
        created, skipped
    );

    Ok(())
}
