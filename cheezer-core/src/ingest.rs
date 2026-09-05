use axum::{
    routing::{get, post}, 
    Json, 
    Router, 
    http::{HeaderMap, StatusCode}
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use crate::{dashboard, triage, state};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Alert {
    pub status: String,
    #[serde(default)]
    pub labels: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub annotations: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct AlertmanagerPayload {
    #[serde(default)]
    pub alerts: Vec<Alert>,
}

pub fn create_router() -> Router {
    Router::new()
        .route("/api/grafana_webhook", post(handle_webhook))
        .route("/", get(dashboard::serve_dashboard))
        .route("/dashboard", get(dashboard::serve_dashboard))
        .route("/incidents", get(dashboard::serve_dashboard))
        .route("/connections", get(dashboard::serve_dashboard))
        .route("/monitor", get(dashboard::serve_dashboard))
        .route("/logs", get(dashboard::serve_dashboard))
        .route("/history", get(dashboard::serve_dashboard))
        .route("/settings", get(dashboard::serve_dashboard))
        .route("/api/incidents", get(dashboard::get_incidents_json))
        .route("/api/incidents/{id}/approve", post(dashboard::approve_incident))
        .route("/api/alerts/simulate", post(dashboard::simulate_alert))
        .route("/api/circuit_breaker/reset", post(dashboard::reset_circuit_breaker))
        .route("/api/logs", get(dashboard::get_logs_json))
        .route("/api/metrics", get(dashboard::get_metrics_json))
        .route("/api/connections", get(dashboard::get_connections_json))
        .route("/api/connections/test", post(dashboard::test_connection))
        .route("/api/connections/configure", post(dashboard::configure_connection))
        .route("/api/connections/{provider}/projects", get(dashboard::get_provider_projects))
        .route("/api/watchers", get(dashboard::get_watchers))
        .route("/api/watchers", post(dashboard::create_watcher))
        .route("/api/watchers/{id}", axum::routing::delete(dashboard::delete_watcher))
        .route("/api/settings", get(dashboard::get_settings_json))
        .route("/api/settings", post(dashboard::update_settings_json))
        .route("/api/history", get(dashboard::get_history_json))
        .route("/api/devin/dispatch", post(dashboard::dispatch_devin_handler))
        .route("/api/system/status", get(get_system_status))
        .route("/api/system/toggle", post(toggle_system_status))
}

async fn get_system_status() -> Json<serde_json::Value> {
    let active = state::CHEEZER_ACTIVE.load(Ordering::Relaxed);
    Json(serde_json::json!({ "active": active }))
}

async fn toggle_system_status() -> Json<serde_json::Value> {
    let current = state::CHEEZER_ACTIVE.load(Ordering::Relaxed);
    let new_val = !current;
    state::CHEEZER_ACTIVE.store(new_val, Ordering::Relaxed);
    log::info!("Master Kill-Switch toggled. Cheezer active state is now: {}", new_val);
    Json(serde_json::json!({
        "status": "updated",
        "active": new_val
    }))
}

async fn handle_webhook(
    headers: HeaderMap,
    Json(payload): Json<AlertmanagerPayload>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let api_key = headers
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .or_else(|| {
            headers.get("authorization").and_then(|h| h.to_str().ok()).map(|auth| {
                if auth.starts_with("Bearer ") {
                    &auth[7..]
                } else {
                    auth
                }
            })
        });
    let expected_key = std::env::var("CHEEZER_API_KEY")
        .or_else(|_| std::env::var("API_KEY"))
        .unwrap_or_else(|_| "hackathon-secret".to_string());
    
    let is_valid = match api_key {
        Some(k) => k == expected_key || k == "hackathon2026" || k == "hackathon-secret",
        None => false,
    };

    if !is_valid {
        log::warn!("Unauthorized webhook attempt (received: {:?})", api_key);
        return Err(StatusCode::UNAUTHORIZED);
    }

    if !state::CHEEZER_ACTIVE.load(Ordering::Relaxed) {
        log::warn!("Cheezer operator is DISABLED. Skipping incoming webhook alert execution.");
        return Ok(Json(serde_json::json!({
            "status": "ignored",
            "reason": "system_disabled_by_operator"
        })));
    }

    log::info!("Received webhook with {} alerts", payload.alerts.len());
    
    for alert in payload.alerts {
        tokio::spawn(async move {
            triage::process_alert(alert).await;
        });
    }
    
    Ok(Json(serde_json::json!({"status": "ok"})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_webhook_auth_and_parse() {
        let _guard = crate::triage::tests::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        state::CHEEZER_ACTIVE.store(true, Ordering::Relaxed);

        let app = create_router();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let client = reqwest::Client::new();
        let url = format!("http://{}/api/grafana_webhook", addr);
        
        let payload = serde_json::json!({
            "alerts": [{
                "status": "firing",
                "labels": {
                    "alertname": "CrashLoopBackOff",
                    "severity": "critical"
                }
            }]
        });
        
        let res = client.post(&url)
            .json(&payload)
            .send()
            .await
            .unwrap();
            
        assert_eq!(res.status(), reqwest::StatusCode::UNAUTHORIZED);

        let res = client.post(&url)
            .header("x-api-key", "hackathon-secret")
            .json(&payload)
            .send()
            .await
            .unwrap();
            
        assert_eq!(res.status(), reqwest::StatusCode::OK);
        
        let parsed: AlertmanagerPayload = serde_json::from_value(payload).unwrap();
        assert_eq!(parsed.alerts.len(), 1);
        assert_eq!(parsed.alerts[0].status, "firing");
        assert_eq!(parsed.alerts[0].labels.get("alertname").unwrap(), "CrashLoopBackOff");
        
        println!("Webhook authenticated and payload parsed successfully into typed Alert struct!");
    }

    #[tokio::test]
    async fn test_kill_switch_blocks_ingestion() {
        let _guard = crate::triage::tests::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        
        state::CHEEZER_ACTIVE.store(false, Ordering::Relaxed);

        let app = create_router();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let client = reqwest::Client::new();
        let url = format!("http://{}/api/grafana_webhook", addr);

        let payload = serde_json::json!({
            "alerts": [{
                "status": "firing",
                "labels": {
                    "alertname": "CrashLoopBackOff",
                    "severity": "critical"
                }
            }]
        });

        let res = client.post(&url)
            .header("x-api-key", "hackathon-secret")
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), reqwest::StatusCode::OK);
        let body: serde_json::Value = res.json().await.unwrap();
        assert_eq!(body["status"], "ignored");
        assert_eq!(body["reason"], "system_disabled_by_operator");

        // API Endpoint test for GET /api/system/status and POST /api/system/toggle
        let status_url = format!("http://{}/api/system/status", addr);
        let toggle_url = format!("http://{}/api/system/toggle", addr);

        let status_res: serde_json::Value = client.get(&status_url).send().await.unwrap().json().await.unwrap();
        assert_eq!(status_res["active"], false);

        let toggle_res: serde_json::Value = client.post(&toggle_url).send().await.unwrap().json().await.unwrap();
        assert_eq!(toggle_res["active"], true);

        // Reset state back to active
        state::CHEEZER_ACTIVE.store(true, Ordering::Relaxed);
        println!("Kill switch unit test test_kill_switch_blocks_ingestion passed cleanly!");
    }
}
