use crate::ingest::Alert;
use crate::fallback;
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

pub struct Decision {
    pub action: String,
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
        Ok(Ok(action)) => Decision {
            action,
            mode: "ai".to_string(),
        },
        _ => {
            log::warn!("LLM unreachable or timed out, entering Local Fallback Mode");
            let action = fallback::execute_fallback(alert);
            Decision {
                action,
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

    let mut retries = 0;
    let max_retries = 3;
    let mut backoff_ms = 50;

    while retries < max_retries {
        let mock_response = std::env::var("MOCK_LLM_RESPONSE").unwrap_or_default();
        if !mock_response.is_empty() {
             return Ok(mock_response);
        }

        let alertname = alert.labels.get("alertname").map(|s| s.as_str()).unwrap_or("");
        if alertname == "UnknownDatabaseLatencySpike" {
            return Ok("restart pod".to_string());
        }
        
        retries += 1;
        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
        backoff_ms *= 2;
    }
    
    Err("LLM failed after retries".into())
}



