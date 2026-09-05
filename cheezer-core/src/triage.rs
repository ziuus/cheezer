use crate::ingest::Alert;
use crate::{store, fallback, llm, policy, executor};

pub async fn process_alert(alert: Alert) {
    let signature = alert.labels.get("alertname").map(|s| s.as_str()).unwrap_or("Unknown");
    let severity = alert.labels.get("severity").map(|s| s.as_str()).unwrap_or("info");
    
    let alert_id = match store::log_alert(signature, severity) {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to log alert: {}", e);
            return;
        }
    };

    if let Some(action) = fallback::match_rule(&alert) {
        log::info!("Matched known pattern for {}: {:?}", signature, action);
        if action.starts_with("log") {
             let _ = store::log_incident(signature, severity, "rule", &action, "logged");
             let _ = store::log_action(alert_id, "rule", &action);
             return;
        }
        execute_action(alert_id, signature, severity, "rule", &action, &alert).await;
        return;
    }

    let (occurrences, self_resolved) = store::get_signature_stats(signature).unwrap_or((1, 0));
    let resolution_rate = if occurrences > 0 { self_resolved as f32 / occurrences as f32 } else { 0.0 };

    if resolution_rate > 0.8 && severity == "low" {
        log::info!("Alert {} is low severity and often self-resolves. Logging only.", signature);
        let _ = store::log_incident(signature, severity, "none", "ignored (self-resolving)", "logged");
        let _ = store::log_action(alert_id, "none", "ignored (self-resolving)");
        return;
    }

    log::info!("Escalating novel alert {} to LLM", signature);
    let decision = llm::analyze(&alert).await;
    if decision.action.starts_with("log") || decision.action == "none" {
        let _ = store::log_incident(signature, severity, &decision.mode, &decision.action, "logged");
        let _ = store::log_action(alert_id, &decision.mode, &decision.action);
        return;
    }
    execute_action(alert_id, signature, severity, &decision.mode, &decision.action, &alert).await;
}

async fn execute_action(alert_id: i64, signature: &str, severity: &str, mode: &str, action: &str, alert: &Alert) {
    let (action_type, resource, target_replicas, cmd) = parse_action(action);
    let is_allowed = policy::check_action(action_type, resource, target_replicas, cmd).await;
    
    if !is_allowed {
        log::warn!("Action blocked by OPA policy: {}", action);
        let _ = store::log_incident(signature, severity, mode, action, "blocked");
        let _ = store::log_action(alert_id, mode, &format!("blocked: {}", action));
        return;
    }

    match executor::apply_action(action, alert).await {
        Ok(_) => {
            log::info!("Successfully executed action: {}", action);
            let _ = store::log_incident(signature, severity, mode, action, "executed");
            let _ = store::log_action(alert_id, mode, action);
        }
        Err(e) => {
            log::error!("Failed to execute action: {}", e);
            let _ = store::log_incident(signature, severity, mode, action, "failed");
            let _ = store::log_action(alert_id, mode, &format!("failed: {}", action));
        }
    }
}

fn parse_action(action: &str) -> (&str, &str, i32, Vec<&str>) {
    if action.starts_with("restart pod") {
        ("restart", "pod", 0, vec![])
    } else if action.starts_with("delete namespace") {
        ("delete", "namespace", 0, vec![])
    } else if action.starts_with("scale") {
        let parts: Vec<&str> = action.split_whitespace().collect();
        let target_replicas = parts.last().and_then(|s| s.parse().ok()).unwrap_or(0);
        ("scale", "deployment", target_replicas, vec![])
    } else if action.starts_with("exec") {
        ("exec", "pod", 0, vec!["exec"])
    } else if action.starts_with("modify rbac") {
        ("modify", "rbac", 0, vec![])
    } else {
        ("unknown", "unknown", 0, vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn test_rule_first_triage_and_logging() {
        let _guard = TEST_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
        }
        store::init_db().unwrap();

        store::clear_db().unwrap();
        llm::reset_llm_call_count();

        // 1. CrashLoopBackOff alert
        let mut labels1 = HashMap::new();
        labels1.insert("alertname".to_string(), "CrashLoopBackOff".to_string());
        labels1.insert("severity".to_string(), "critical".to_string());
        labels1.insert("pod".to_string(), "test-pod-1".to_string());
        labels1.insert("namespace".to_string(), "default".to_string());

        let alert1 = Alert {
            status: "firing".to_string(),
            labels: labels1,
            annotations: HashMap::new(),
        };

        // 2. OOMKilled alert
        let mut labels2 = HashMap::new();
        labels2.insert("alertname".to_string(), "OOMKilled".to_string());
        labels2.insert("severity".to_string(), "warning".to_string());
        labels2.insert("pod".to_string(), "test-pod-2".to_string());
        labels2.insert("namespace".to_string(), "default".to_string());

        let alert2 = Alert {
            status: "firing".to_string(),
            labels: labels2,
            annotations: HashMap::new(),
        };

        // Process both alerts
        process_alert(alert1).await;
        process_alert(alert2).await;

        // Assert zero LLM calls were made
        assert_eq!(llm::get_llm_call_count(), 0, "LLM must NOT be called for known rule-matched alerts!");

        // Assert incidents logged in SQLite
        let incidents = store::get_incidents().unwrap();
        assert_eq!(incidents.len(), 2, "Expected 2 incidents in database");

        // Incident 1 assertions
        assert_eq!(incidents[0].signature, "CrashLoopBackOff");
        assert_eq!(incidents[0].mode, "rule");
        assert_eq!(incidents[0].action, "restart pod");
        assert_eq!(incidents[0].status, "executed");

        // Incident 2 assertions
        assert_eq!(incidents[1].signature, "OOMKilled");
        assert_eq!(incidents[1].mode, "rule");
        assert_eq!(incidents[1].action, "restart pod");
        assert_eq!(incidents[1].status, "executed");

        println!("SUCCESS: CrashLoopBackOff and OOMKilled triaged via rule path with zero LLM calls and logged to SQLite incidents table!");
    }

    #[tokio::test]
    async fn test_llm_escalation_for_novel_alert() {
        let _guard = TEST_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
            std::env::set_var("MOCK_LLM_RESPONSE", "restart pod");
        }
        store::init_db().unwrap();
        store::clear_db().unwrap();
        llm::reset_llm_call_count();

        // Novel high-severity alert that misses all 6 rule patterns
        let mut labels = HashMap::new();
        labels.insert("alertname".to_string(), "UnknownDatabaseLatencySpike".to_string());
        labels.insert("severity".to_string(), "critical".to_string());
        labels.insert("pod".to_string(), "db-pod-0".to_string());
        labels.insert("namespace".to_string(), "production".to_string());

        let alert = Alert {
            status: "firing".to_string(),
            labels,
            annotations: HashMap::new(),
        };

        // Process alert
        process_alert(alert).await;

        // 1. Assert rule matcher fell through and LLM was invoked exactly once
        assert_eq!(llm::get_llm_call_count(), 1, "LLM MUST be invoked for unrecognized novel alerts!");

        // 2. Assert incident is recorded in SQLite with mode='ai', evaluated action, and status='executed'
        let incidents = store::get_incidents().unwrap();
        assert_eq!(incidents.len(), 1, "Expected 1 incident in database");
        assert_eq!(incidents[0].signature, "UnknownDatabaseLatencySpike");
        assert_eq!(incidents[0].mode, "ai");
        assert_eq!(incidents[0].action, "restart pod");
        assert_eq!(incidents[0].status, "executed");

        println!("SUCCESS: Novel alert UnknownDatabaseLatencySpike escalated to LLM, evaluated by policy gate, and saved to SQLite with mode='ai'!");
    }

    #[tokio::test]
    async fn test_llm_timeout_triggers_fallback() {
        let _guard = TEST_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
            std::env::set_var("FORCE_LLM_TIMEOUT", "true");
            std::env::remove_var("MOCK_LLM_RESPONSE");
        }
        store::init_db().unwrap();
        store::clear_db().unwrap();
        llm::reset_llm_call_count();

        // Novel alert that times out during LLM resolution
        let mut labels = HashMap::new();
        labels.insert("alertname".to_string(), "NetworkPartitionDetected".to_string());
        labels.insert("severity".to_string(), "critical".to_string());
        labels.insert("pod".to_string(), "net-pod-0".to_string());
        labels.insert("namespace".to_string(), "production".to_string());

        let alert = Alert {
            status: "firing".to_string(),
            labels,
            annotations: HashMap::new(),
        };

        // Process alert
        process_alert(alert).await;

        // 1. Assert LLM attempt was registered before timing out
        assert_eq!(llm::get_llm_call_count(), 1, "LLM attempt should be registered before timing out!");

        // 2. Assert incident is recorded in SQLite with mode='fallback', safe action, and status='executed'
        let incidents = store::get_incidents().unwrap();
        assert_eq!(incidents.len(), 1, "Expected 1 incident in database");
        assert_eq!(incidents[0].signature, "NetworkPartitionDetected");
        assert_eq!(incidents[0].mode, "fallback");
        assert_eq!(incidents[0].action, "restart pod");
        assert_eq!(incidents[0].status, "executed");

        // Clean up environment variables
        unsafe {
            std::env::remove_var("FORCE_LLM_TIMEOUT");
        }

        println!("SUCCESS: LLM timeout triggered Local Fallback Mode within timeout, passed policy gate, and saved to SQLite with mode='fallback'!");
    }

    #[tokio::test]
    async fn test_opa_blocks_dangerous_actions() {
        let _guard = TEST_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
            std::env::set_var("MOCK_LLM_RESPONSE", "delete namespace default");
        }
        store::init_db().unwrap();
        store::clear_db().unwrap();
        llm::reset_llm_call_count();

        // Alert where LLM attempts dangerous action (namespace deletion)
        let mut labels = HashMap::new();
        labels.insert("alertname".to_string(), "UnsafeLLMRecommendation".to_string());
        labels.insert("severity".to_string(), "critical".to_string());
        labels.insert("namespace".to_string(), "default".to_string());

        let alert = Alert {
            status: "firing".to_string(),
            labels,
            annotations: HashMap::new(),
        };

        process_alert(alert).await;

        // Verify OPA policy gate blocked the execution and persisted status = 'blocked'
        let incidents = store::get_incidents().unwrap();
        assert_eq!(incidents.len(), 1, "Expected 1 incident in database");
        assert_eq!(incidents[0].signature, "UnsafeLLMRecommendation");
        assert_eq!(incidents[0].mode, "ai");
        assert_eq!(incidents[0].action, "delete namespace default");
        assert_eq!(incidents[0].status, "blocked");

        unsafe {
            std::env::remove_var("MOCK_LLM_RESPONSE");
        }

        println!("SUCCESS: OPA blocked dangerous namespace deletion proposed by LLM, recorded status='blocked' in SQLite!");
    }
}




