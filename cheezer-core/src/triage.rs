use crate::action::Action;
use crate::ingest::Alert;
use crate::{store, fallback, guard, llm, policy, executor};

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
        if action.action_type() == "log" {
             let _ = store::log_incident(signature, severity, "rule", &action.to_action_string(), "logged");
             let _ = store::log_action(alert_id, "rule", &action.to_action_string());
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
    if decision.action.action_type() == "log" || decision.action == Action::None {
        let _ = store::log_incident(signature, severity, &decision.mode, &decision.action.to_action_string(), "logged");
        let _ = store::log_action(alert_id, &decision.mode, &decision.action.to_action_string());
        return;
    }
    execute_action(alert_id, signature, severity, &decision.mode, &decision.action, &alert).await;
}

async fn execute_action(alert_id: i64, signature: &str, severity: &str, mode: &str, action: &Action, alert: &Alert) {
    let action_str = action.to_action_string();
    let target_res = action.target_resource();
    let target_resource = if let Some(pod) = alert.labels.get("pod") {
        pod.as_str()
    } else if let Some(node) = alert.labels.get("node") {
        node.as_str()
    } else if !target_res.is_empty() {
        &target_res
    } else {
        action.resource_type()
    };

    // 0. TOCTOU Revalidation Check (runs BEFORE Remediation Guard & OPA)
    if let Err(executor::ExecutionError::StaleState(reason)) = executor::revalidate_state(action).await {
        log::warn!("Execution ABORTED due to TOCTOU stale state: {}", reason);
        let _ = store::log_incident_with_verification(
            signature,
            severity,
            mode,
            &action_str,
            "Aborted_StaleState",
            "Aborted_StaleState",
        );
        let _ = store::log_action(alert_id, mode, &format!("aborted_stale_state: {}", reason));
        return;
    }

    // 1. Remediation Guard Check (sits BEFORE OPA policy check)
    match guard::RemediationGuard::evaluate(alert_id, target_resource, action) {
        guard::GuardResult::Block(reason) => {
            log::warn!("Action blocked by Remediation Guard: {}", reason);
            guard::send_outbound_notification(target_resource, &action_str, &reason);
            let _ = store::log_incident(signature, severity, mode, &action_str, "requires_human_intervention");
            let _ = store::log_action(alert_id, mode, &format!("blocked_by_guard: {}", reason));
            return;
        }
        guard::GuardResult::Allow => {}
    }

    // 2. OPA Policy Check
    let is_allowed = policy::check_action(action).await;
    if !is_allowed {
        log::warn!("Action blocked by OPA policy: {}", action_str);
        let _ = store::log_incident(signature, severity, mode, &action_str, "blocked");
        let _ = store::log_action(alert_id, mode, &format!("blocked: {}", action_str));
        return;
    }

    // 3. Executor with Post-Execution Recovery Verification
    match executor::apply_action(action, alert).await {
        Ok(_) => {
            log::info!("Successfully executed action: {}", action_str);
            let _ = store::log_remediation(alert_id, target_resource, &action_str);

            let is_recovered = match executor::verify_recovery(action).await {
                Ok(true) => "Recovered",
                Ok(false) => "Failed",
                Err(_) => "Failed",
            };
            log::info!("Post-remediation verification status: {}", is_recovered);

            let _ = store::log_incident_with_verification(signature, severity, mode, &action_str, "executed", is_recovered);
            let _ = store::log_action(alert_id, mode, &action_str);
        }
        Err(e) => {
            log::error!("Failed to execute action: {}", e);
            let _ = store::log_incident_with_verification(signature, severity, mode, &action_str, "failed", "Failed");
            let _ = store::log_action(alert_id, mode, &format!("failed: {}", action_str));
        }
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

pub static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn test_rule_first_triage_and_logging() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
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
        assert_eq!(incidents[0].action, "restart pod test-pod-1");
        assert_eq!(incidents[0].status, "executed");

        // Incident 2 assertions
        assert_eq!(incidents[1].signature, "OOMKilled");
        assert_eq!(incidents[1].mode, "rule");
        assert_eq!(incidents[1].action, "restart pod test-pod-2");
        assert_eq!(incidents[1].status, "executed");

        println!("SUCCESS: CrashLoopBackOff and OOMKilled triaged via rule path with zero LLM calls and logged to SQLite incidents table!");
    }

    #[tokio::test]
    async fn test_llm_escalation_for_novel_alert() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
            std::env::remove_var("MOCK_LLM_RESPONSE");
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
        assert_eq!(incidents[0].action, "restart pod db-pod-0");
        assert_eq!(incidents[0].status, "executed");

        println!("SUCCESS: Novel alert UnknownDatabaseLatencySpike escalated to LLM, evaluated by policy gate, and saved to SQLite with mode='ai'!");
    }

    #[tokio::test]
    async fn test_llm_structured_parsing_success() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let valid_json = serde_json::json!({
            "incident_class": "DatabaseLatencySpike",
            "confidence": 0.95,
            "proposed_action": "RestartPod",
            "target": {
                "namespace": "production",
                "resource": "db-pod-0"
            },
            "reason": "Database latency spike detected"
        }).to_string();

        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
            std::env::set_var("MOCK_LLM_RESPONSE", &valid_json);
        }
        store::init_db().unwrap();
        store::clear_db().unwrap();
        llm::reset_llm_call_count();

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

        process_alert(alert).await;

        let incidents = store::get_incidents().unwrap();
        assert_eq!(incidents.len(), 1);
        assert_eq!(incidents[0].signature, "UnknownDatabaseLatencySpike");
        assert_eq!(incidents[0].mode, "ai");
        assert_eq!(incidents[0].action, "restart pod db-pod-0");
        assert_eq!(incidents[0].status, "executed");

        unsafe {
            std::env::remove_var("MOCK_LLM_RESPONSE");
        }

        println!("SUCCESS: LLM response parsed into structured Action enum and executed!");
    }

    #[tokio::test]
    async fn test_llm_invalid_action_triggers_fallback() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let invalid_json = serde_json::json!({
            "incident_class": "MaliciousAttempt",
            "confidence": 0.9,
            "proposed_action": "kubectl delete namespace production",
            "target": {
                "namespace": "production",
                "resource": "all"
            },
            "reason": "Malicious command execution proposal"
        }).to_string();

        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
            std::env::set_var("MOCK_LLM_RESPONSE", &invalid_json);
        }
        store::init_db().unwrap();
        store::clear_db().unwrap();
        llm::reset_llm_call_count();

        let mut labels = HashMap::new();
        labels.insert("alertname".to_string(), "UnknownAnomaly".to_string());
        labels.insert("severity".to_string(), "critical".to_string());
        labels.insert("pod".to_string(), "app-pod-1".to_string());
        labels.insert("namespace".to_string(), "production".to_string());

        let alert = Alert {
            status: "firing".to_string(),
            labels,
            annotations: HashMap::new(),
        };

        process_alert(alert).await;

        let incidents = store::get_incidents().unwrap();
        assert_eq!(incidents.len(), 1);
        assert_eq!(incidents[0].signature, "UnknownAnomaly");
        assert_eq!(incidents[0].mode, "fallback");
        assert_eq!(incidents[0].action, "restart pod app-pod-1");
        assert_eq!(incidents[0].status, "executed");

        unsafe {
            std::env::remove_var("MOCK_LLM_RESPONSE");
        }

        println!("SUCCESS: Unallowed raw LLM action was rejected and safely triggered Local Fallback Mode!");
    }

    #[tokio::test]
    async fn test_llm_timeout_triggers_fallback() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
            std::env::set_var("FORCE_LLM_TIMEOUT", "true");
            std::env::remove_var("MOCK_LLM_RESPONSE");
        }
        store::init_db().unwrap();
        store::clear_db().unwrap();
        llm::reset_llm_call_count();

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

        process_alert(alert).await;

        assert_eq!(llm::get_llm_call_count(), 1, "LLM attempt should be registered before timing out!");

        let incidents = store::get_incidents().unwrap();
        assert_eq!(incidents.len(), 1, "Expected 1 incident in database");
        assert_eq!(incidents[0].signature, "NetworkPartitionDetected");
        assert_eq!(incidents[0].mode, "fallback");
        assert_eq!(incidents[0].action, "restart pod net-pod-0");
        assert_eq!(incidents[0].status, "executed");

        unsafe {
            std::env::remove_var("FORCE_LLM_TIMEOUT");
        }

        println!("SUCCESS: LLM timeout triggered Local Fallback Mode within timeout, passed policy gate, and saved to SQLite with mode='fallback'!");
    }

    #[tokio::test]
    async fn test_opa_blocks_dangerous_actions() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let unsafe_json = serde_json::json!({
            "incident_class": "UnsafeRequest",
            "confidence": 0.95,
            "proposed_action": "DeleteNamespace",
            "target": {
                "namespace": "default",
                "resource": "default"
            },
            "reason": "Requesting namespace deletion"
        }).to_string();

        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
            std::env::set_var("MOCK_LLM_RESPONSE", &unsafe_json);
        }
        store::init_db().unwrap();
        store::clear_db().unwrap();
        llm::reset_llm_call_count();

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

    #[tokio::test]
    async fn test_remediation_loop_blocked() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
            std::env::set_var("IGNORE_COOLDOWN", "true");
        }
        store::init_db().unwrap();
        store::clear_db().unwrap();
        llm::reset_llm_call_count();
        policy::reset_policy_call_count();

        for _ in 1..=4 {
            let mut labels = HashMap::new();
            labels.insert("alertname".to_string(), "CrashLoopBackOff".to_string());
            labels.insert("severity".to_string(), "critical".to_string());
            labels.insert("pod".to_string(), "flapping-pod-x".to_string());
            labels.insert("namespace".to_string(), "default".to_string());

            let alert = Alert {
                status: "firing".to_string(),
                labels,
                annotations: HashMap::new(),
            };

            process_alert(alert).await;
        }

        let incidents = store::get_incidents().unwrap();
        assert_eq!(incidents.len(), 4, "Expected 4 incident entries");

        assert_eq!(incidents[0].status, "executed");
        assert_eq!(incidents[1].status, "executed");
        assert_eq!(incidents[2].status, "executed");

        assert_eq!(incidents[3].status, "requires_human_intervention");
        assert_eq!(incidents[3].action, "restart pod flapping-pod-x");

        assert_eq!(policy::get_policy_call_count(), 3, "OPA must only be called 3 times; 4th attempt must be blocked by RemediationGuard before OPA!");

        unsafe {
            std::env::remove_var("IGNORE_COOLDOWN");
        }

        println!("SUCCESS: RemediationGuard blocked 4th rapid pod restart before OPA was reached, marking status='requires_human_intervention'!");
    }

    #[tokio::test]
    async fn test_executor_aborts_on_stale_state() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
            std::env::set_var("MOCK_OPA_ENABLED", "true");
            std::env::set_var("MOCK_STALE_STATE", "true");
        }
        store::init_db().unwrap();
        store::clear_db().unwrap();

        let mut labels = HashMap::new();
        labels.insert("alertname".to_string(), "CrashLoopBackOff".to_string());
        labels.insert("severity".to_string(), "critical".to_string());
        labels.insert("pod".to_string(), "self-healing-pod".to_string());
        labels.insert("namespace".to_string(), "default".to_string());

        let alert = Alert {
            status: "firing".to_string(),
            labels,
            annotations: HashMap::new(),
        };

        process_alert(alert).await;

        let incidents = store::get_incidents().unwrap();
        assert_eq!(incidents.len(), 1, "Expected 1 incident in database");
        assert_eq!(incidents[0].status, "Aborted_StaleState");
        assert_eq!(incidents[0].verification_result, "Aborted_StaleState");

        unsafe {
            std::env::remove_var("MOCK_STALE_STATE");
        }

        println!("SUCCESS: TOCTOU revalidation aborted execution on self-healing pod and logged Aborted_StaleState in SQLite!");
    }
}






