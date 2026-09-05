use axum::{
    routing::{get, post}, 
    Json, 
    Router, 
    http::{HeaderMap, StatusCode}
};
use serde::{Deserialize, Serialize};
use crate::{dashboard, triage};

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
        .route("/dashboard", get(dashboard::serve_dashboard))
        .route("/api/incidents", get(dashboard::get_incidents_json))
        .route("/api/incidents/{id}/approve", post(dashboard::approve_incident))
}

async fn handle_webhook(
    headers: HeaderMap,
    Json(payload): Json<AlertmanagerPayload>
) -> Result<Json<&'static str>, StatusCode> {
    let api_key = headers.get("x-api-key").and_then(|h| h.to_str().ok());
    let expected_key = std::env::var("CHEEZER_API_KEY").unwrap_or_else(|_| "hackathon-secret".to_string());
    
    if api_key != Some(expected_key.as_str()) {
        log::warn!("Unauthorized webhook attempt");
        return Err(StatusCode::UNAUTHORIZED);
    }

    log::info!("Received webhook with {} alerts", payload.alerts.len());
    
    for alert in payload.alerts {
        tokio::spawn(async move {
            triage::process_alert(alert).await;
        });
    }
    
    Ok(Json("ok"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;


    #[tokio::test]
    async fn test_webhook_auth_and_parse() {
        let _guard = crate::triage::tests::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        // Start server on a random ephemeral port
        let app = create_router();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        // Spawn the server
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let client = reqwest::Client::new();
        let url = format!("http://{}/api/grafana_webhook", addr);
        
        // 1. Test unauthorized (missing key)
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

        // 2. Test authorized and parsed correctly
        let res = client.post(&url)
            .header("x-api-key", "hackathon-secret")
            .json(&payload)
            .send()
            .await
            .unwrap();
            
        assert_eq!(res.status(), reqwest::StatusCode::OK);
        
        // Let's also assert parsing logic locally just to be absolutely certain it maps to the typed Alert struct
        let parsed: AlertmanagerPayload = serde_json::from_value(payload).unwrap();
        assert_eq!(parsed.alerts.len(), 1);
        assert_eq!(parsed.alerts[0].status, "firing");
        assert_eq!(parsed.alerts[0].labels.get("alertname").unwrap(), "CrashLoopBackOff");
        
        println!("Webhook authenticated and payload parsed successfully into typed Alert struct!");
    }
}
