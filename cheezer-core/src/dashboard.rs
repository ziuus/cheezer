use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use crate::action::Action;
use crate::executor;
use crate::ingest::Alert;
use crate::policy;
use crate::store;
use serde_json::json;
use std::collections::HashMap;

pub async fn serve_dashboard() -> impl IntoResponse {
    Html(DASHBOARD_HTML)
}

pub async fn get_incidents_json() -> impl IntoResponse {
    let incidents = store::get_incidents().unwrap_or_default();
    let remediations = store::get_remediations().unwrap_or_default();
    Json(json!({
        "incidents": incidents,
        "remediations": remediations
    }))
}

pub async fn approve_incident(
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    log::info!("Received human approval request for incident ID: {}", id);

    let incident = match store::get_incident_by_id(id) {
        Ok(Some(inc)) => inc,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"error": format!("Incident {} not found", id)})),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ));
        }
    };

    if incident.status != "requires_human_intervention" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Incident {} is in status '{}', expected 'requires_human_intervention'", id, incident.status)
            })),
        ));
    }

    // Parse proposed action from SQLite string
    let action = Action::parse_from_string(&incident.action);
    log::info!("Re-evaluating human-approved action against OPA policy gate: {:?}", action);

    // NON-NEGOTIABLE CONSTRAINT: Human approval MUST still pass OPA policy check
    let is_opa_allowed = policy::check_action(&action).await;
    if !is_opa_allowed {
        log::warn!("Human approval rejected by OPA policy gate for action: {}", incident.action);
        let _ = store::update_incident_status(id, "blocked_by_opa");
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "status": "blocked_by_opa",
                "error": "OPA policy engine rejected human-approved action"
            })),
        ));
    }

    // Execute via executor
    let dummy_alert = Alert {
        status: "firing".to_string(),
        labels: HashMap::new(),
        annotations: HashMap::new(),
    };

    match executor::apply_action(&action, &dummy_alert).await {
        Ok(_) => {
            log::info!("Human approved action executed successfully for incident {}", id);
            let _ = store::update_incident_status(id, "human_approved_and_executed");

            // Reset/release the RemediationGuard lock for the target resource
            let target_resource = action.target_resource();
            if !target_resource.is_empty() {
                let _ = store::reset_resource_remediations(&target_resource);
                let _ = store::log_remediation(id, &target_resource, &incident.action);
            }

            Ok(Json(json!({
                "status": "human_approved_and_executed",
                "incident_id": id,
                "action": incident.action
            })))
        }
        Err(e) => {
            log::error!("Execution failed for human approved action: {}", e);
            let _ = store::update_incident_status(id, "approval_execution_failed");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Execution failed: {}", e)
                })),
            ))
        }
    }
}

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Cheezer Autonomous Remediation Engine</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/htmx.org@2.0.4"></script>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;600;700&family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap');
        body { font-family: 'Plus Jakarta Sans', sans-serif; }
        code, .font-mono { font-family: 'JetBrains Mono', monospace; }
    </style>
</head>
<body class="bg-slate-950 text-slate-100 min-h-screen">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <!-- Header -->
        <header class="flex flex-col md:flex-row md:items-center md:justify-between pb-8 border-b border-slate-800 gap-4">
            <div class="flex items-center space-x-4">
                <div class="w-12 h-12 rounded-xl bg-gradient-to-tr from-amber-500 via-orange-500 to-yellow-400 flex items-center justify-center shadow-lg shadow-amber-500/20 font-black text-2xl text-slate-950">
                    🧀
                </div>
                <div>
                    <h1 class="text-2xl font-bold tracking-tight text-white flex items-center gap-3">
                        Cheezer Core <span class="text-xs px-2.5 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-mono font-semibold">v0.1.0 • Out-of-Band</span>
                    </h1>
                    <p class="text-sm text-slate-400 mt-0.5">Autonomous Kubernetes Remediation Engine & Human Approval Gateway</p>
                </div>
            </div>
            <div class="flex items-center space-x-3">
                <div class="flex items-center space-x-2 bg-slate-900 border border-slate-800 px-3 py-1.5 rounded-lg text-xs font-mono text-slate-300">
                    <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
                    <span>WATCHDOG ACTIVE</span>
                </div>
                <div class="flex items-center space-x-2 bg-slate-900 border border-slate-800 px-3 py-1.5 rounded-lg text-xs font-mono text-slate-300">
                    <span class="w-2 h-2 rounded-full bg-blue-400 animate-ping"></span>
                    <span>OPA FAIL-CLOSED ENFORCED</span>
                </div>
            </div>
        </header>

        <!-- KPI Cards Grid -->
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 my-8" id="kpi-grid">
            <div class="bg-slate-900/60 border border-slate-800 rounded-xl p-5 backdrop-blur">
                <div class="text-xs font-medium text-slate-400 uppercase tracking-wider">Total Incidents</div>
                <div class="text-3xl font-extrabold text-white mt-2" id="kpi-total">0</div>
            </div>
            <div class="bg-slate-900/60 border border-slate-800 rounded-xl p-5 backdrop-blur">
                <div class="text-xs font-medium text-slate-400 uppercase tracking-wider">Self-Remediated</div>
                <div class="text-3xl font-extrabold text-emerald-400 mt-2" id="kpi-executed">0</div>
            </div>
            <div class="bg-amber-950/30 border border-amber-500/30 rounded-xl p-5 backdrop-blur">
                <div class="text-xs font-medium text-amber-400 uppercase tracking-wider flex items-center justify-between">
                    Requires Approval
                    <span class="w-2 h-2 rounded-full bg-amber-400 animate-pulse"></span>
                </div>
                <div class="text-3xl font-extrabold text-amber-400 mt-2" id="kpi-approval">0</div>
            </div>
            <div class="bg-slate-900/60 border border-slate-800 rounded-xl p-5 backdrop-blur">
                <div class="text-xs font-medium text-slate-400 uppercase tracking-wider">OPA Denials / Blocked</div>
                <div class="text-3xl font-extrabold text-rose-400 mt-2" id="kpi-blocked">0</div>
            </div>
        </div>

        <!-- Main Incident Audit Table Section -->
        <main class="space-y-8">
            <section class="bg-slate-900/50 border border-slate-800 rounded-2xl p-6 backdrop-blur shadow-xl">
                <div class="flex items-center justify-between pb-4 mb-4 border-b border-slate-800">
                    <div>
                        <h2 class="text-lg font-bold text-white">Live Incident Stream & Circuit Breakers</h2>
                        <p class="text-xs text-slate-400 mt-0.5">Real-time audit log of rule evaluations, LLM escalations, and human intervention locks</p>
                    </div>
                    <button onclick="fetchIncidents()" class="text-xs font-mono bg-slate-800 hover:bg-slate-700 text-slate-300 px-3 py-1.5 rounded-lg border border-slate-700 transition">
                        ⚡ Refresh Now
                    </button>
                </div>

                <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="text-xs font-mono uppercase tracking-wider text-slate-400 border-b border-slate-800 bg-slate-950/40">
                                <th class="py-3 px-4">ID</th>
                                <th class="py-3 px-4">Timestamp</th>
                                <th class="py-3 px-4">Signature / Alert</th>
                                <th class="py-3 px-4">Severity</th>
                                <th class="py-3 px-4">Mode</th>
                                <th class="py-3 px-4">Proposed Action</th>
                                <th class="py-3 px-4">Status</th>
                                <th class="py-3 px-4 text-right">Human Override</th>
                            </tr>
                        </thead>
                        <tbody id="incidents-body" class="divide-y divide-slate-800/60 text-sm">
                            <tr>
                                <td colspan="8" class="text-center py-8 text-slate-500 font-mono">Loading incident stream...</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </section>

            <!-- Remediation Audit Trail Table -->
            <section class="bg-slate-900/50 border border-slate-800 rounded-2xl p-6 backdrop-blur shadow-xl">
                <div class="flex items-center justify-between pb-4 mb-4 border-b border-slate-800">
                    <div>
                        <h2 class="text-lg font-bold text-white">Remediation History Audit Log</h2>
                        <p class="text-xs text-slate-400 mt-0.5">Executed cluster mutations and resource rate-limit history</p>
                    </div>
                </div>

                <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="text-xs font-mono uppercase tracking-wider text-slate-400 border-b border-slate-800 bg-slate-950/40">
                                <th class="py-3 px-4">Remediation ID</th>
                                <th class="py-3 px-4">Incident ID</th>
                                <th class="py-3 px-4">Resource</th>
                                <th class="py-3 px-4">Action</th>
                                <th class="py-3 px-4">Timestamp</th>
                            </tr>
                        </thead>
                        <tbody id="remediations-body" class="divide-y divide-slate-800/60 text-sm">
                            <tr>
                                <td colspan="5" class="text-center py-6 text-slate-500 font-mono">No remediation history records yet</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </section>
        </main>
    </div>

    <script>
        async function approveIncident(id) {
            if (!confirm(`Are you sure you want to approve and execute incident #${id}? Action will be validated by OPA before execution.`)) return;
            try {
                const res = await fetch(`/api/incidents/${id}/approve`, { method: 'POST' });
                const data = await res.json();
                if (res.ok) {
                    alert(`✅ Incident #${id} successfully approved and executed!\nAction: ${data.action}`);
                    fetchIncidents();
                } else {
                    alert(`❌ Approval rejected/failed:\n${data.error || JSON.stringify(data)}`);
                    fetchIncidents();
                }
            } catch (err) {
                alert(`Error communicating with server: ${err}`);
            }
        }

        async function fetchIncidents() {
            try {
                const res = await fetch('/api/incidents');
                const data = await res.json();
                renderIncidents(data.incidents || []);
                renderRemediations(data.remediations || []);
            } catch (err) {
                console.error("Failed to fetch incidents:", err);
            }
        }

        function renderIncidents(list) {
            let total = list.length;
            let executed = 0;
            let approval = 0;
            let blocked = 0;

            const body = document.getElementById('incidents-body');
            if (list.length === 0) {
                body.innerHTML = `<tr><td colspan="8" class="text-center py-8 text-slate-500 font-mono">No incidents recorded yet</td></tr>`;
                return;
            }

            let html = '';
            for (const inc of list) {
                if (inc.status === 'executed' || inc.status === 'human_approved_and_executed') executed++;
                else if (inc.status === 'requires_human_intervention') approval++;
                else if (inc.status === 'blocked' || inc.status === 'blocked_by_opa') blocked++;

                let statusBadge = '';
                if (inc.status === 'requires_human_intervention') {
                    statusBadge = `<span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-amber-500/10 text-amber-400 border border-amber-500/30">⚠️ Circuit Breaker Locked</span>`;
                } else if (inc.status === 'executed' || inc.status === 'human_approved_and_executed') {
                    statusBadge = `<span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">✅ ${inc.status}</span>`;
                } else if (inc.status === 'blocked' || inc.status === 'blocked_by_opa') {
                    statusBadge = `<span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-rose-500/10 text-rose-400 border border-rose-500/30">🚫 ${inc.status}</span>`;
                } else {
                    statusBadge = `<span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-slate-800 text-slate-300">${inc.status}</span>`;
                }

                let modeBadge = `<span class="font-mono text-xs text-slate-400 uppercase">${inc.mode}</span>`;
                if (inc.mode === 'rule') modeBadge = `<span class="font-mono text-xs text-sky-400 font-semibold uppercase">⚡ RULE</span>`;
                else if (inc.mode === 'ai') modeBadge = `<span class="font-mono text-xs text-purple-400 font-semibold uppercase">🤖 AI</span>`;
                else if (inc.mode === 'fallback') modeBadge = `<span class="font-mono text-xs text-amber-400 font-semibold uppercase">🛡️ FALLBACK</span>`;

                let actionBtn = '-';
                if (inc.status === 'requires_human_intervention') {
                    actionBtn = `<button onclick="approveIncident(${inc.id})" class="bg-amber-500 hover:bg-amber-400 text-slate-950 font-bold px-3 py-1 rounded text-xs transition shadow shadow-amber-500/20">Approve & Execute</button>`;
                }

                html += `
                    <tr class="hover:bg-slate-900/40 transition">
                        <td class="py-3 px-4 font-mono text-slate-400">#${inc.id}</td>
                        <td class="py-3 px-4 font-mono text-xs text-slate-400">${inc.timestamp || '-'}</td>
                        <td class="py-3 px-4 font-semibold text-white">${inc.signature}</td>
                        <td class="py-3 px-4"><span class="text-xs px-2 py-0.5 rounded bg-slate-800 text-slate-300 font-mono">${inc.severity}</span></td>
                        <td class="py-3 px-4">${modeBadge}</td>
                        <td class="py-3 px-4 font-mono text-xs text-slate-300">${inc.action}</td>
                        <td class="py-3 px-4">${statusBadge}</td>
                        <td class="py-3 px-4 text-right">${actionBtn}</td>
                    </tr>
                `;
            }

            body.innerHTML = html;
            document.getElementById('kpi-total').innerText = total;
            document.getElementById('kpi-executed').innerText = executed;
            document.getElementById('kpi-approval').innerText = approval;
            document.getElementById('kpi-blocked').innerText = blocked;
        }

        function renderRemediations(list) {
            const body = document.getElementById('remediations-body');
            if (list.length === 0) {
                body.innerHTML = `<tr><td colspan="5" class="text-center py-6 text-slate-500 font-mono">No remediation history records yet</td></tr>`;
                return;
            }

            let html = '';
            for (const rem of list) {
                html += `
                    <tr class="hover:bg-slate-900/40 transition">
                        <td class="py-2.5 px-4 font-mono text-slate-400">#${rem.id}</td>
                        <td class="py-2.5 px-4 font-mono text-slate-400">#${rem.incident_id}</td>
                        <td class="py-2.5 px-4 font-mono text-sky-400 font-semibold">${rem.resource}</td>
                        <td class="py-2.5 px-4 font-mono text-xs text-slate-300">${rem.action}</td>
                        <td class="py-2.5 px-4 font-mono text-xs text-slate-400">${rem.timestamp}</td>
                    </tr>
                `;
            }
            body.innerHTML = html;
        }

        setInterval(fetchIncidents, 2000);
        fetchIncidents();
    </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_human_approval_flow() {
        let _guard = crate::triage::tests::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("MOCK_EXECUTOR", "true");
        }
        store::init_db().unwrap();
        store::clear_db().unwrap();

        // 1. Trigger a guard block by exceeding resource budget
        let resource = "test-pod-human-approval";
        let mut labels = HashMap::new();
        labels.insert("alertname".to_string(), "CrashLoopBackOff".to_string());
        labels.insert("severity".to_string(), "critical".to_string());
        labels.insert("pod".to_string(), resource.to_string());
        labels.insert("namespace".to_string(), "default".to_string());

        let alert = Alert {
            status: "firing".to_string(),
            labels,
            annotations: HashMap::new(),
        };

        // Log 3 initial remediations to exceed per-resource budget (max 3 per 10 mins)
        let action_str = format!("restart pod {}", resource);
        store::log_remediation(1, resource, &action_str).unwrap();
        store::log_remediation(1, resource, &action_str).unwrap();
        store::log_remediation(1, resource, &action_str).unwrap();

        // Process 4th alert - Remediation Guard will BLOCK and transition incident to 'requires_human_intervention'
        crate::triage::process_alert(alert).await;

        let incidents = store::get_incidents().unwrap();
        assert!(!incidents.is_empty(), "Expected an incident to be recorded");
        let blocked_inc = incidents
            .iter()
            .find(|i| i.status == "requires_human_intervention")
            .expect("Expected incident with status 'requires_human_intervention'");

        let blocked_id = blocked_inc.id;
        println!(
            "Verified incident #{} blocked by Remediation Guard in status 'requires_human_intervention'",
            blocked_id
        );

        // 2. Spawn Axum server and programmatically post to /api/incidents/:id/approve
        let app = crate::ingest::create_router();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let client = reqwest::Client::new();
        let url = format!("http://{}/api/incidents/{}/approve", addr, blocked_id);

        let res = client.post(&url).send().await.unwrap();
        assert_eq!(res.status(), reqwest::StatusCode::OK);

        // 3. Assert status updated to 'human_approved_and_executed'
        let updated_inc = store::get_incident_by_id(blocked_id).unwrap().unwrap();
        assert_eq!(updated_inc.status, "human_approved_and_executed");

        // 4. Verify OPA enforcement constraint: Attempt approving an action that violates OPA (delete namespace)
        let opa_denied_id = store::log_incident(
            "DangerousDeleteNamespace",
            "critical",
            "rule",
            "delete namespace production",
            "requires_human_intervention",
        )
        .unwrap();

        let deny_url = format!("http://{}/api/incidents/{}/approve", addr, opa_denied_id);
        let deny_res = client.post(&deny_url).send().await.unwrap();
        assert_eq!(deny_res.status(), reqwest::StatusCode::FORBIDDEN);

        let denied_inc = store::get_incident_by_id(opa_denied_id).unwrap().unwrap();
        assert_eq!(denied_inc.status, "blocked_by_opa");

        println!("SUCCESS: Human approval flow verified! Valid approvals executed, OPA violations blocked cleanly!");
    }
}

