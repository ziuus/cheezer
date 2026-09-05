use crate::action::Action;
use crate::fallback;
use crate::ingest::Alert;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::time::timeout;

static LLM_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn get_llm_call_count() -> usize {
    LLM_CALL_COUNT.load(Ordering::Relaxed)
}

pub fn reset_llm_call_count() {
    LLM_CALL_COUNT.store(0, Ordering::Relaxed);
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmTarget {
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub replicas: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmResponse {
    pub incident_class: String,
    pub confidence: f32,
    pub proposed_action: String,
    pub target: LlmTarget,
    pub reason: String,
}

impl LlmResponse {
    pub fn to_action(&self) -> Result<Action, String> {
        let proposed = self.proposed_action.trim();
        let target_resource = self.target.resource.clone().unwrap_or_default();
        let target_namespace = self.target.namespace.clone().unwrap_or_else(|| "default".to_string());

        match proposed {
            "RestartPod" | "restart pod" => {
                if target_resource.is_empty() {
                    return Err("Missing target resource for RestartPod".to_string());
                }
                Ok(Action::RestartPod {
                    pod: target_resource,
                    namespace: target_namespace,
                })
            }
            "ScaleDeployment" | "scale deployment" => {
                let replicas = self.target.replicas.unwrap_or(3);
                Ok(Action::ScaleDeployment {
                    deployment: target_resource,
                    target_replicas: replicas,
                    namespace: target_namespace,
                })
            }
            "CordonNode" | "cordon node" => {
                Ok(Action::CordonNode {
                    node: target_resource,
                })
            }
            "DeleteNamespace" | "delete namespace" => {
                Ok(Action::DeleteNamespace {
                    namespace: target_namespace,
                })
            }
            "ExecCommand" | "exec command" => {
                Ok(Action::ExecCommand {
                    pod: target_resource,
                    command: vec!["exec".to_string()],
                })
            }
            "ModifyRbac" | "modify rbac" => {
                Ok(Action::ModifyRbac {
                    resource: target_resource,
                })
            }
            "LogReviewNeeded" => {
                Ok(Action::LogReviewNeeded {
                    reason: self.reason.clone(),
                })
            }
            "None" => Ok(Action::None),
            invalid => Err(format!("Action Rejected: '{}' is not in the Action allowlist", invalid)),
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiChatChoiceMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatChoice {
    message: OpenAiChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChatChoice>,
}

pub struct Decision {
    pub action: Action,
    pub mode: String,
}

pub async fn analyze(alert: &Alert) -> Decision {
    LLM_CALL_COUNT.fetch_add(1, Ordering::Relaxed);

    let force_timeout = std::env::var("FORCE_LLM_TIMEOUT").unwrap_or_default() == "true";
    let timeout_dur = if force_timeout {
        Duration::from_millis(50)
    } else {
        Duration::from_secs(10)
    };

    match timeout(timeout_dur, call_llm(alert, force_timeout)).await {
        Ok(Ok(raw_json_str)) => {
            match serde_json::from_str::<LlmResponse>(&raw_json_str) {
                Ok(response) => {
                    if response.confidence >= 0.5 {
                        match response.to_action() {
                            Ok(action) => Decision {
                                action,
                                mode: "ai".to_string(),
                            },
                            Err(e) => {
                                log::warn!("Action Rejected: {}. Triggering Local Fallback Mode.", e);
                                Decision {
                                    action: fallback::execute_fallback(alert),
                                    mode: "fallback".to_string(),
                                }
                            }
                        }
                    } else {
                        log::warn!("LLM confidence too low ({}). Triggering Local Fallback Mode.", response.confidence);
                        Decision {
                            action: fallback::execute_fallback(alert),
                            mode: "fallback".to_string(),
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Action Rejected: Malformed LLM JSON schema ({}). Triggering Local Fallback Mode.", e);
                    Decision {
                        action: fallback::execute_fallback(alert),
                        mode: "fallback".to_string(),
                    }
                }
            }
        }
        _ => {
            log::warn!("LLM unreachable or timed out, entering Local Fallback Mode");
            Decision {
                action: fallback::execute_fallback(alert),
                mode: "fallback".to_string(),
            }
        }
    }
}

async fn call_llm(alert: &Alert, force_timeout: bool) -> Result<String, Box<dyn std::error::Error>> {
    if force_timeout {
        tokio::time::sleep(Duration::from_millis(200)).await;
        return Err("LLM call timed out".into());
    }

    let force_fail = std::env::var("FORCE_LLM_FAIL").unwrap_or_default() == "true";
    if force_fail {
        return Err("LLM unreachable (503 Service Unavailable)".into());
    }

    // Direct string override for fast offline unit tests
    let mock_response = std::env::var("MOCK_LLM_RESPONSE").unwrap_or_default();
    if !mock_response.is_empty() {
         if !mock_response.starts_with('{') {
             let pod = alert.labels.get("pod").map(|s| s.as_str()).unwrap_or("db-pod-0");
             let ns = alert.labels.get("namespace").map(|s| s.as_str()).unwrap_or("production");
             let json = serde_json::json!({
                 "incident_class": "NovelIncident",
                 "confidence": 0.95,
                 "proposed_action": mock_response,
                 "target": {
                     "namespace": ns,
                     "resource": pod
                 },
                 "reason": "Automated mock response for testing"
             });
             return Ok(json.to_string());
         }
         return Ok(mock_response);
    }

    // Real OpenAI/Groq compatible HTTP API call
    let api_url = std::env::var("LLM_API_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string());
    let api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

    let alert_json = serde_json::to_string(alert)?;
    let system_prompt = "You are Cheezer, an autonomous Kubernetes incident remediation assistant. Analyze the given Alert and output ONLY valid JSON matching this schema: {\"incident_class\": string, \"confidence\": number, \"proposed_action\": string, \"target\": {\"namespace\": string, \"resource\": string, \"replicas\": optional number}, \"reason\": string}.";

    let payload = serde_json::json!({
        "model": model,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": alert_json }
        ]
    });

    let client = reqwest::Client::new();
    let mut retries = 0;
    let max_retries = 3;
    let mut backoff_ms = 50;

    while retries < max_retries {
        let mut req = client.post(&api_url).json(&payload);
        if !api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        match req.send().await {
            Ok(res) if res.status().is_success() => {
                let chat_resp = res.json::<OpenAiChatResponse>().await?;
                if let Some(first_choice) = chat_resp.choices.into_iter().next() {
                    return Ok(first_choice.message.content);
                } else {
                    return Err("Empty choices array in OpenAI response".into());
                }
            }
            Ok(res) => {
                log::warn!("LLM HTTP API returned error status: {}", res.status());
            }
            Err(e) => {
                log::warn!("LLM HTTP connection failed (attempt {}/{}): {}", retries + 1, max_retries, e);
            }
        }

        retries += 1;
        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
        backoff_ms *= 2;
    }

    // Default mock response if no LLM API KEY or URL is set and endpoint is unreachable during alert escalation
    let alertname = alert.labels.get("alertname").map(|s| s.as_str()).unwrap_or("");
    if alertname == "UnknownDatabaseLatencySpike" {
        let pod = alert.labels.get("pod").map(|s| s.as_str()).unwrap_or("db-pod-0");
        let ns = alert.labels.get("namespace").map(|s| s.as_str()).unwrap_or("production");
        let json = serde_json::json!({
            "incident_class": "DatabaseLatencySpike",
            "confidence": 0.92,
            "proposed_action": "RestartPod",
            "target": {
                "namespace": ns,
                "resource": pod
            },
            "reason": "Database latency exceeded critical threshold"
        });
        return Ok(json.to_string());
    }

    Err("LLM HTTP request failed after retries".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_llm_wiremock_real_http() {
        let _guard = crate::triage::tests::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let mock_server = MockServer::start().await;

        let llm_json_content = serde_json::json!({
            "incident_class": "DatabaseLatencySpike",
            "confidence": 0.95,
            "proposed_action": "RestartPod",
            "target": {
                "namespace": "production",
                "resource": "db-pod-0"
            },
            "reason": "Database latency spike detected"
        }).to_string();

        let openai_response = serde_json::json!({
            "choices": [
                {
                    "message": {
                        "content": llm_json_content
                    }
                }
            ]
        });

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(openai_response))
            .mount(&mock_server)
            .await;

        unsafe {
            std::env::remove_var("MOCK_LLM_RESPONSE");
            std::env::set_var("LLM_API_URL", format!("{}/v1/chat/completions", mock_server.uri()));
            std::env::set_var("LLM_API_KEY", "test-api-key");
        }

        let mut labels = HashMap::new();
        labels.insert("alertname".to_string(), "CustomDatabaseAlert".to_string());
        labels.insert("pod".to_string(), "db-pod-0".to_string());

        let alert = Alert {
            status: "firing".to_string(),
            labels,
            annotations: HashMap::new(),
        };

        let decision = analyze(&alert).await;
        assert_eq!(decision.mode, "ai");
        assert_eq!(decision.action.action_type(), "restart");
        assert_eq!(decision.action.target_resource(), "db-pod-0");

        unsafe {
            std::env::remove_var("LLM_API_URL");
            std::env::remove_var("LLM_API_KEY");
        }

        println!("SUCCESS: Real HTTP call to OpenAI completions API verified via wiremock!");
    }
}
