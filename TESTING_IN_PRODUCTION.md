# Cheezer Core — Production Testing Documentation

This document contains the exact guide, procedure, and script references for testing Cheezer Core in a live production or staging Kubernetes environment (`demo` namespace) with Floci AWS Cloud Emulation (`http://172.18.100.41:4566`).

---

## 1. System Architecture & Component Links

Cheezer Core operates as an autonomous incident response control plane for Kubernetes clusters and microservices.

```
+-------------------------------------------------------------------------------+
|                             Grafana / Alertmanager                            |
+---------------------------------------+---------------------------------------+
                                        | HTTP Webhook (JSON payload)
                                        v
+-------------------------------------------------------------------------------+
|                                  Cheezer Core                                 |
|                       (Port 9090: /api/grafana_webhook)                       |
+--------+------------------------------+-------------------------------+-------+
         |                              |                               |
         v                              v                               v
+------------------+          +-------------------+           +------------------+
|   OPA Engine     |          |  TOCTOU Engine    |           | RemediationGuard |
| (Fail-Closed Auth|          | (Re-validates state|          | (Rate limits max |
|   & Policy)      |          |  before actions)  |           | 3 actions / 10m) |
+--------+---------+          +---------+---------+           +--------+---------+
         |                              |                              |
         +------------------------------+------------------------------+
                                        |
                                        v
+-------------------------------------------------------------------------------+
|                               Remediation Action                              |
|          - Kubernetes Pod Restart / Scale / Rollout                           |
|          - Floci AWS S3 Audit Log (http://172.18.100.41:4566)                |
|          - Vercel / Render PaaS Gateway Webhooks                              |
|          - Real-Time Web Console Update (/dashboard)                          |
+-------------------------------------------------------------------------------+
```

---

## 2. Interactive Web Control Plane & REST API Testing

Cheezer Core features a modern dark glassmorphism dashboard built to `apex-ui-engineer` standards with vector icons (Lucide SVG) across 6 management views.

### Port Forwarding Access
To access the live Control Plane UI in production:
```bash
# Find the active Cheezer pod in namespace demo
POD_NAME=$(kubectl get pods -n demo -l app=cheezer-core -o jsonpath='{.items[0].metadata.name}')

# Start local port forward
kubectl port-forward pod/$POD_NAME 9090:9090 -n demo
```

Open your browser to: **`http://localhost:9090/dashboard`**

---

### REST API Endpoints Reference

All API calls accept either `Authorization: Bearer <API_KEY>` or `x-api-key: <API_KEY>`. Default production key: `hackathon2026`.

#### 1. Connections Manager
- **GET `/api/connections`**: Lists active connections (Kubernetes, Floci AWS S3/SQS, Vercel, Render, GitHub, Grafana).
- **POST `/api/connections/test`**: Triggers real-time connectivity & latency verification.
```bash
curl -s -X GET http://localhost:9090/api/connections \
  -H "Authorization: Bearer hackathon2026"
```

#### 2. Live Telemetry & Metrics
- **GET `/api/metrics`**: Returns incident statistics, OPA block count, fast path percentages, and TOCTOU revalidation metrics.
```bash
curl -s -X GET http://localhost:9090/api/metrics \
  -H "Authorization: Bearer hackathon2026"
```

#### 3. Real-Time Log Console
- **GET `/api/logs`**: Returns structured real-time system, triage, and execution log streams.
```bash
curl -s -X GET http://localhost:9090/api/logs \
  -H "Authorization: Bearer hackathon2026"
```

#### 4. Audit History & Incidents
- **GET `/api/history`**: Returns full incident audit trail stored in SQLite & synchronized to Floci AWS S3.
```bash
curl -s -X GET http://localhost:9090/api/history \
  -H "Authorization: Bearer hackathon2026"
```

#### 5. System & LLM Settings
- **GET `/api/settings`**: Retrieves current system configuration (LLM provider, OPA URL, RemediationGuard limits).
- **POST `/api/settings`**: Updates runtime configuration dynamically.
```bash
curl -s -X POST http://localhost:9090/api/settings \
  -H "Authorization: Bearer hackathon2026" \
  -H "Content-Type: application/json" \
  -d '{"llm_provider": "NVIDIA NIM", "llm_model": "meta/llama-3.2-11b-vision-instruct", "notification_webhook_url": "https://httpbin.org/post", "remediation_guard_max_actions": 3, "remediation_guard_window_seconds": 600}'
```

---

## 3. Automated Chaos Testing Suite (`scripts/trigger-chaos-bug.sh`)

Use the provided chaos test trigger script to simulate 5 failure scenarios against the running production system:

### Usage Syntax:
```bash
./scripts/trigger-chaos-bug.sh [scenario] [endpoint]
```

Default endpoint: `http://localhost:9090/api/grafana_webhook`

---

### Chaos Scenario Scenarios:

#### Scenario 1: `crashloop`
Simulates a `flaky-order-service` pod crashing repeatedly in `CrashLoopBackOff`.
```bash
./scripts/trigger-chaos-bug.sh crashloop
```
*Expected Behavior:* Cheezer Core receives alert, evaluates pattern fast-path, performs TOCTOU check, restarts pod `flaky-order-service`, updates incident history, and archives log to Floci AWS S3.

---

#### Scenario 2: `oom`
Simulates an Out-Of-Memory (`OOMKilled`) container termination.
```bash
./scripts/trigger-chaos-bug.sh oom
```
*Expected Behavior:* Triage engine detects memory exhaustion alert, triggers memory ceiling adjustment / pod restart, and logs audit record.

---

#### Scenario 3: `dangerous` (OPA Safety Policy Enforcement)
Simulates a dangerous or unauthorized remediation command (`rm -rf /` or host namespace privilege escalation).
```bash
./scripts/trigger-chaos-bug.sh dangerous
```
*Expected Behavior:* Cheezer's OPA engine evaluates fail-closed authorization policy, **BLOCKS** execution completely (`OPA Status: BLOCKED`), increments `opa_blocked` counter, and logs a security violation audit event.

---

#### Scenario 4: `rate-limit` (RemediationGuard Enforcement)
Simulates a runaway alert loop firing 5 consecutive alerts within a 60-second window.
```bash
./scripts/trigger-chaos-bug.sh rate-limit
```
*Expected Behavior:* First 3 remediation actions execute successfully. The 4th and 5th alerts are rate-limited by `RemediationGuard` (`REMEDIATION_GUARD_LIMIT_EXCEEDED`), preventing cascading instability.

---

#### Scenario 5: `kill-switch` (Emergency Shutdown)
Simulates manual or automated activation of the emergency system kill-switch.
```bash
./scripts/trigger-chaos-bug.sh kill-switch
```
*Expected Behavior:* All incoming remediation webhook triggers are rejected until the kill switch is manually reset via the settings panel or API.

---

## 4. Telemetry & OpenTelemetry Pipeline Setup (`scripts/floci-otel-pipeline.sh`)

To test end-to-end OpenTelemetry (OTel) metrics & alert flow from Grafana Alertmanager to Cheezer Core:

```bash
# Execute the OTel pipeline setup script
./scripts/floci-otel-pipeline.sh
```

This script:
1. Deploys the Grafana Alertmanager & OTel Collector stack in namespace `demo`.
2. Connects Cheezer Core in-cluster DNS (`http://cheezer-core.demo.svc.cluster.local:9090/api/grafana_webhook`).
3. Connects Floci AWS Emulator for telemetry archival at `http://172.18.100.41:4566`.

---

## 5. Verification Checklist

To verify a successful production deployment:

1. **Kubernetes Pod Status**:
   `kubectl get pods -n demo` -> `cheezer-core` MUST show `STATUS: Running` and `READY: 1/1`.
2. **Unit & Integration Test Suite**:
   `cargo test` -> All 19 unit tests MUST pass cleanly.
3. **Control Plane Dashboard**:
   `curl -s http://localhost:9090/dashboard` -> Returns status `200 OK` with 6-tab HTML.
4. **Safety & Audit Integrity**:
   Verify OPA policy enforcement blocks unauthorized actions (`./scripts/trigger-chaos-bug.sh dangerous`).
5. **Floci AWS Synchronization**:
   Verify S3 audit bucket synchronization at `http://172.18.100.41:4566`.

---

*Documentation maintained by Cheezer Reliability Engineering.*
