use serde::{Deserialize, Serialize};
use std::error::Error;
use crate::store;

#[derive(Debug, Serialize, Deserialize)]
pub struct DevinDispatchPayload {
    pub repo: Option<String>,
    pub incident_id: Option<i64>,
    pub prompt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevinSessionResponse {
    pub session_id: String,
    pub url: String,
    pub status: String,
    pub message: String,
}

pub async fn dispatch_devin_agent(
    repo: &str,
    incident_signature: &str,
    action: &str,
    logs: &str,
) -> Result<DevinSessionResponse, Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("DEVIN_API_KEY")
        .or_else(|_| std::env::var("DEVIN_KEY"))
        .ok()
        .or_else(|| {
            store::get_credential("devin")
                .ok()
                .flatten()
                .map(|(t, _, _)| t)
        })
        .filter(|k| !k.trim().is_empty());

    let key = match api_key {
        Some(k) => k,
        None => return Err("DEVIN_API_KEY is unconfigured. Please enter your Devin API Token in the Connections tab under 'Devin AI Autonomous Engineer API' or set the DEVIN_API_KEY environment variable to dispatch live Devin AI agents.".into()),
    };

    let target_repo = if repo.trim().is_empty() {
        "ziuus/cheezer".to_string()
    } else {
        repo.trim().to_string()
    };

    let prompt = format!(
        "High-priority incident detected by Cheezer AI Operator on repository '{target_repo}'.\n\
         Alert Signature: '{incident_signature}'\n\
         Target Action: '{action}'\n\
         Log Trace:\n{logs}\n\n\
         Please inspect the repository source code, identify the root cause, fix the bug, run tests, and open a Pull Request with the remediation."
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client
        .post("https://api.devin.ai/v1/sessions")
        .header("Authorization", format!("Bearer {}", key.trim()))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "prompt": prompt,
        }))
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        let body: serde_json::Value = response.json().await?;
        let session_id = body["session_id"]
            .as_str()
            .unwrap_or("devin-session-active")
            .to_string();
        let url = body["url"]
            .as_str()
            .unwrap_or(&format!("https://preview.devin.ai/devin/session/{}", session_id))
            .to_string();

        Ok(DevinSessionResponse {
            session_id,
            url: url.clone(),
            status: "dispatched".to_string(),
            message: format!("🤖 Devin AI successfully dispatched to repository '{}'! Live Session URL: {}", target_repo, url),
        })
    } else {
        let err_text = response.text().await.unwrap_or_default();
        Err(format!("Devin AI API error (HTTP {}): {}", status, err_text).into())
    }
}
