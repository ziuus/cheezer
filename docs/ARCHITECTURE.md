# Cheezer Core Architecture & Technical Specification

> **Positioning:** The Autonomous, Vendor-Neutral, Safety-First Recovery Control Plane  
> **Operational Mandate:** *"Cheap enough to run continuously, fast enough to remediate in milliseconds, and predictive enough to act before failure."*

Cheezer is engineered to run **out-of-band** (outside target Kubernetes/cloud infrastructure) to guarantee operational continuity and emergency self-healing capability even during total target control plane or network degradation.

---

## 🏗️ System Component Topology

```text
                               ┌─────────────────────────────┐
                               │   Grafana / Alertmanager    │
                               │   Datadog / Sentry / OTel   │
                               └──────────────┬──────────────┘
                                              │ POST /api/webhooks/alert
                                              ▼
                               ┌─────────────────────────────┐
                               │     Ingest (`ingest.rs`)    │
                               │  (Kill Switch Check & Auth) │
                               └──────────────┬──────────────┘
                                              │ Alert struct
                                              ▼
                               ┌─────────────────────────────┐
                               │  Predictive Risk Engine     │
                               │     (`predictive.rs`)       │
                               │  (Linear Trends & EWMA)     │
                               └──────────────┬──────────────┘
                                              │
                                              ▼
                               ┌─────────────────────────────┐
                               │     Triage (`triage.rs`)    │
                               │  6-Tier Escalation Ladder   │
                               └──────┬────────────────┬──────┘
                                      │                │
            (Tier 0/1 Fast Path)      │                │ (Tier 4 Cloud LLM / Tier 5 Devin AI)
                      ┌───────────────┘                └───────────────┐
                      ▼                                                ▼
         ┌─────────────────────────┐                      ┌─────────────────────────┐
         │ Fast-Path Rule Engine   │                      │  LLM Router (`llm.rs`)  │
         │  (`triage.rs` <1ms)     │                      │ (gpt-4o-mini / gpt-4o)  │
         └────────────┬────────────┘                      └────────────┬────────────┘
                      │                                                │ (Timeout / Low Conf)
                      │                                                ▼
                      │                                   ┌─────────────────────────┐
                      │                                   │ Fallback (`fallback.rs`)│
                      │                                   └────────────┬────────────┘
                      │                                                │
                      └───────────────────────┬────────────────────────┘
                                              ▼
                               ┌─────────────────────────────┐
                               │ TOCTOU Check (`guard.rs`)   │
                               │   (`revalidate_state`)      │
                               └──────────────┬──────────────┘
                                              │ (State Still Broken)
                                              ▼
                               ┌─────────────────────────────┐
                               │ RemediationGuard(`guard.rs`)│
                               │ (Disruption Budget: <=3/15m)│
                               └──────────────┬──────────────┘
                                              │ (Budget Allowed)
                                              ▼
                               ┌─────────────────────────────┐
                               │  OPA Policy (`policy.rs`)   │
                               │  (Fail-Closed DENY Gate)    │
                               └──────────────┬──────────────┘
                                              │ (OPA Result == ALLOW)
                                              ▼
                               ┌─────────────────────────────┐
                               │   Executor (`executor.rs`)  │
                               │   (19 Platform Connectors)  │
                               └──────────────┬──────────────┘
                                              │
                                              ▼
                               ┌─────────────────────────────┐
                               │ Recovery Verification Check │
                               │   (Proves System Recovery)  │
                               └──────────────┬──────────────┘
                                              │
                                ┌─────────────┴─────────────┐
                                ▼                           ▼
                             RESOLVED                  ESCALATE → GITOPS PR
                                                          (Devin AI)
```

---

## 🔒 Security Boundaries & Component Breakdown

### 1. Ingestion (`ingest.rs`)
- Mounts `POST /api/webhooks/alert` on Axum.
- Validates HTTP token headers against `CHEEZER_API_KEY`.
- Verifies global kill switch state (`ENABLE_AUTONOMOUS_REMEDIATION=true`).
- Normalizes telemetry payloads into a unified typed `Alert` struct.

### 2. Predictive Risk & Forecasting Engine (`predictive.rs`)
- Calculates linear/exponential memory growth rates, disk volume fill rates, and EWMA Z-score baseline deviations.
- Computes Time-To-Failure (TTF in minutes) and Failure Probability (0–100%).
- Triggers **preventive self-healing** before an outage occurs if probability > 75% and TTF < 20 mins (`Predict → Decide → Revalidate → Authorize → Remediate → Verify`).

### 3. Rule Engine Triage (`triage.rs`)
- Performs deterministic matching against known patterns: `CrashLoopBackOff`, `OOMKilled`, `DNSResolutionFailure`, `NodeDiskPressure`, `ContainerCannotStart`, `DatabaseLatencySpike`.
- Matched alerts trigger actions directly from the Rust fast path (< 1ms) for zero AI API cost.

### 4. Adaptive 6-Tier LLM Router (`llm.rs` & `devin.rs`)
- **Tier 0:** Fast-path Rust regex rules ($0.00 / <1ms).
- **Tier 1:** Statistical trend models & EWMA ($0.00 / <2ms).
- **Tier 2:** CPU-based LightGBM / Isolation Forest inference ($0.00 marginal).
- **Tier 3:** Local quantized small LLM (3B–8B, Ollama) ($0.00 marginal).
- **Tier 4:** Cloud LLM (`gpt-4o-mini` for warnings, `gpt-4o` for critical multi-service cascades).
- **Tier 5:** Devin AI Autonomous Engineer for code-level GitOps Pull Requests when 3 infra fixes fail.

### 5. TOCTOU & RemediationGuard Gate (`guard.rs`)
- **TOCTOU Revalidation:** Re-queries live cluster/cloud health right before executing mutations. Aborts if target has self-resolved.
- **Disruption Budget:** Enforces a sliding-window rate limit (max 3 actions per 15 minutes per target workload), stopping cascading alert storms.

### 6. OPA Policy Gate (`policy.rs`)
- Embedded Open Policy Agent (Rego engine).
- Enforces fail-closed evaluation: blocks `DeleteNamespace` on protected namespaces (`kube-system`, `production`), blocks root shell commands, and caps scaling limits (`max_replicas = 20`).

### 7. Multi-Platform Execution Layer (`executor.rs`)
- Direct REST API mutations across 19 supported platforms (Kubernetes, AWS Lambda/App Runner, Google Cloud Run, Azure, Fly.io, Railway, Heroku, Netlify, Docker, Podman, Swarm, Nomad, GitHub, Devin AI).
- Bypasses traditional HPA 90–150s lag for known incidents.

### 8. Remediation Verification (`dashboard.rs` & `executor.rs`)
- Queries post-remediation health metrics (HTTP 200, 5xx error rate drop, pod readiness) to confirm recovery before marking incidents resolved.

### 9. Control Plane Resiliency & Leader Election (`watchdog.rs`)
- Operates in a **Dual-Node HA Pair (Primary + Standby)** connected by a proof-of-life TCP heartbeat watchdog.
- Standby promotes to Primary automatically within 3 missed heartbeats.
- Uses Kubernetes `Lease` primitives to ensure single-leader mutation authority.

---

## 📊 Summary Comparison: Reactive vs. Cheezer Core

| Metric / Dimension | Traditional AIOps / Scripts | Cheezer Core Control Plane |
| :--- | :--- | :--- |
| **Ingestion Model** | Fixed volume & cardinality billing | **Risk-Adaptive Sampling (85-90% data cut)** |
| **Execution Latency** | 90–150s HPA lag | **< 1ms Fast-Path Direct API Mutation** |
| **AI Expense** | Un-gated LLM calls on raw logs | **6-Tier Escalation (~99% LLM Cost Saved)** |
| **Safety Invariants** | Assume script success on `exit 0` | **TOCTOU + OPA Fail-Closed + Health Verification** |
| **Failure Detection** | Purely Reactive (after crash) | **Predictive Forecasting (Linear Trends & EWMA)** |
| **Self-Resilience** | SPOF / Cloud SaaS dependent | **Primary-Standby HA Pair + Watchdog (`watchdog.rs`)** |
