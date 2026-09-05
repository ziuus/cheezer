# Cheezer Architecture & Technical Specification

Cheezer is engineered to run **out-of-band** (outside the target Kubernetes cluster) to guarantee operational continuity and emergency remediation capability even during total control plane or network degradation.

---

## 🏗️ System Component Topology

```text
                               ┌─────────────────────────────┐
                               │   Grafana / Alertmanager    │
                               └──────────────┬──────────────┘
                                              │ POST /api/grafana_webhook
                                              │ (x-api-key: secret)
                                              ▼
                               ┌─────────────────────────────┐
                               │     Ingest (`ingest.rs`)    │
                               └──────────────┬──────────────┘
                                              │ Alert struct
                                              ▼
                               ┌─────────────────────────────┐
                               │     Triage (`triage.rs`)    │
                               └──────┬────────────────┬──────┘
                                      │                │
            (Known Rule Signature)    │                │ (Novel / Unknown Alert)
                      ┌───────────────┘                └───────────────┐
                      ▼                                                ▼
         ┌─────────────────────────┐                      ┌─────────────────────────┐
         │  Rule Matcher (`rule`)  │                      │   LLM API (`llm.rs`)    │
         └────────────┬────────────┘                      │ (OpenAI / Groq HTTP)    │
                      │                                   └────────────┬────────────┘
                      │                                                │ (Timeout / Err)
                      │                                                ▼
                      │                                   ┌─────────────────────────┐
                      │                                   │ Fallback (`fallback.rs`)│
                      │                                   └────────────┬────────────┘
                      │                                                │
                      └───────────────────────┬────────────────────────┘
                                              ▼
                               ┌─────────────────────────────┐
                               │ TOCTOU Check (`executor.rs`)│
                               │   (`revalidate_state`)      │
                               └──────────────┬──────────────┘
                                              │ (State Valid)
                                              ▼
                               ┌─────────────────────────────┐
                               │ RemediationGuard(`guard.rs`)│
                               │   (Rate limit / Budget)     │
                               └──────────────┬──────────────┘
                                              │ (Allowed)
                                              ▼
                               ┌─────────────────────────────┐
                               │  OPA Policy (`policy.rs`)   │
                               │  (Fail-Closed DENY Gate)    │
                               └──────────────┬──────────────┘
                                              │ (OPA Result == true)
                                              ▼
                               ┌─────────────────────────────┐
                               │   Executor (`executor.rs`)  │
                               │    (`kube-rs` Mutations)    │
                               └──────────────┬──────────────┘
                                              │
                                              ▼
                               ┌─────────────────────────────┐
                               │ Recovery Verification Check │
                               │     (`verify_recovery`)     │
                               └──────────────┬──────────────┘
                                              │
                                              ▼
                               ┌─────────────────────────────┐
                               │   SQLite WAL (`store.rs`)   │
                               └─────────────────────────────┘
```

---

## 🔒 Security Boundaries & Component Breakdown

### 1. Ingestion (`ingest.rs`)
- Mounts `/api/grafana_webhook` on Axum.
- Validates the `x-api-key` HTTP header against `CHEEZER_API_KEY` (default: `hackathon-secret`).
- Parses Alertmanager JSON payloads into typed `Alert` structs.

### 2. Rule Engine Triage (`triage.rs`)
- Performs deterministic matching against known patterns: `CrashLoopBackOff`, `OOMKilled`, `DNSResolutionFailure`, `NodeDiskPressure`, `ContainerCannotStart`.
- Matched alerts trigger actions directly from the rule matcher for zero AI cost and sub-millisecond execution.
- Evaluates novel or unrecognised alerts using a heuristic severity scorer to decide if AI escalation is required.

### 3. LLM Escalation & Action Allowlist (`llm.rs` & `action.rs`)
- Makes an OpenAI-compatible HTTP POST call to `LLM_API_URL` (OpenAI / Groq) with `LLM_API_KEY`.
- Enforces structured JSON output via `response_format: { "type": "json_object" }`.
- Deserializes network content directly into `LlmResponse` and converts to the `Action` enum:
  - `RestartPod { pod, namespace }`
  - `ScaleDeployment { deployment, target_replicas, namespace }`
  - `CordonNode { node }`
  - `DeleteNamespace { namespace }`
  - `ExecCommand { pod, command }`
  - `ModifyRbac { resource }`
  - `LogReviewNeeded { reason }`
  - `None`
- If the LLM proposes an unallowed action, returns invalid JSON, or times out (10s limit), Cheezer immediately routes to `fallback::execute_fallback`.

### 4. TOCTOU Revalidation (`executor.rs::revalidate_state`)
- Prevents Time-of-Check to Time-of-Use race conditions.
- Immediately prior to execution, queries the Kubernetes API using `kube-rs`:
  - `RestartPod`: Checks if target pod phase is `Running` and all container statuses report `ready == true`. If so, aborts execution as `Aborted_StaleState`.
  - `ScaleDeployment`: Checks if current deployment replicas already match the target.

### 5. Remediation Guard & Circuit Breaker (`guard.rs`)
- Sits after TOCTOU check and before OPA policy check.
- Evaluates action history stored in SQLite:
  - **Per-Resource Limit**: Max 3 actions on the same resource in a 10-minute window.
  - **Incident Budget**: Max 5 total actions per incident.
  - **Cooldown**: Mandatory 60-second cooldown per resource.
- Exceeding thresholds locks autonomous execution, transitions incident status to `requires_human_intervention`, and emits an outbound notification webhook payload.

### 6. Fail-Closed OPA Policy Gate (`policy.rs`)
- Posts `OpaQuery` JSON payload to `OPA_URL` (`http://localhost:8181/v1/data/cheezer/authz/allow`).
- Evaluates actions against Rego security rules (`policies/cheezer.rego`).
- **Fail-Closed Constraint**: Any HTTP connection error, 5xx error, 500ms timeout, or missing `"result": true` field **MUST** return `false` (DENY).

### 7. Kubernetes Executor & Verification (`executor.rs`)
- Authenticates automatically via local `kubeconfig` or in-cluster `ServiceAccount` using `kube::Client::try_default()`.
- Implements real resource mutations:
  - `RestartPod`: Calls `Api::<Pod>::namespaced().delete()` so Kubernetes controllers recreate it.
  - `ScaleDeployment`: Issues `Patch::Merge` updating `spec.replicas`.
  - `CordonNode`: Issues `Patch::Merge` setting `spec.unschedulable: true`.
- Post-execution, `verify_recovery` checks resource health and logs `verification_result` as `Recovered` or `Failed`.

### 8. Web Dashboard & Human Approval Gateway (`dashboard.rs`)
- Axum routes:
  - `GET /dashboard`: Embedded HTML UI (Tailwind CSS + HTMX).
  - `GET /api/incidents`: JSON endpoint surfacing incidents and remediation history.
  - `POST /api/incidents/{id}/approve`: Human override endpoint for `requires_human_intervention` incidents. Re-evaluates action against OPA before executing.

### 9. High-Availability Watchdog (`watchdog.rs`)
- Runs active-passive failover between Primary and Backup processes.
- Backup monitors Primary over TCP heartbeat. If Primary dies, Backup takes over webhook ingestion automatically.

---

## 📊 Incident Status Lifecycle State Machine

```text
[ Incoming Alert ]
        │
        ▼
 (TOCTOU Check) ───(Self-Resolved)───► Aborted_StaleState
        │ (Valid)
        ▼
(RemediationGuard) ─(Limit Exceeded)─► requires_human_intervention
        │ (Allowed)                               │
        ▼                                         │ (Human Click "Approve")
   (OPA Gate) ────(Rego Denied)──────► blocked    │
        │ (Allowed)                               ▼
        ▼                           (Re-evaluate OPA Gate)
   (Executor) ────(Kube Error)───────► failed     │ (Denied) ──► blocked_by_opa
        │ (Success)                               │ (Allowed)
        ▼                                         ▼
(Verify Recovery) ─(Health Check)──► executed / human_approved_and_executed
                                     (verification_result: Recovered / Failed)
```
