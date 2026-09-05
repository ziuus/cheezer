# 🧀 Cheezer

**Autonomous, Out-of-Band Kubernetes Incident-Remediation Engine in Rust**

Cheezer is an intelligent, policy-gated automation engine designed for out-of-band Kubernetes incident response. Built for high security, zero-trust execution, and zero-AI cost for common issues, Cheezer triages cluster alerts, executes deterministic remediations for known fault patterns, and safely escalates novel incidents to an LLM under strict security bounds.

---

## 📌 Attribution & Architectural Inspiration

The failure-pattern taxonomy (`CrashLoopBackOff`, `OOMKilled`, `ImagePullBackOff`, `DNSResolutionFailure`, etc.) and the general "rule-first, AI-escalation-for-novel-cases" architecture are inspired by the public architecture of two Apache 2.0 licensed open-source projects:
- [K8sGPT](https://github.com/k8sgpt-ai/k8sgpt)
- [HolmesGPT](https://github.com/robusta-dev/holmes)

Cheezer is an original Rust implementation of these conceptual patterns developed for the **"Automation for Good — IT Security & Cyber Resilience"** track. It does not copy their source code verbatim.

---

## 🛡️ Core Architecture & Security Boundaries

```text
[ Alertmanager / Grafana Webhook ]
               │ (x-api-key authenticated)
               ▼
   [ Rule Engine Triage ] ──── (Known pattern: CrashLoop/OOM) ───► [ Zero AI Cost Rule Action ]
               │                                                              │
   (Novel alert fallthrough)                                                  │
               ▼                                                              │
    [ OpenAI / Groq LLM API ]                                                │
   (JSON Mode -> Action Enum)                                                 │
               │                                                              │
               ├───────────────────────────────┬──────────────────────────────┘
               ▼                               ▼
 [ TOCTOU Revalidation ] ──(Self-resolved)──► [ Aborted_StaleState Logged ]
               │ (State valid)
               ▼
   [ Remediation Guard ] ──(Rate limit / Loop)─► [ Requires Human Intervention ]
               │ (Within budget)                            │
               ▼                                            ▼
   [ Fail-Closed OPA Gate ] ◄───────────────── [ Web Dashboard Override ]
 (HTTP 500 / Timeout = DENY)                          (/dashboard)
               │ (Approved by Rego)
               ▼
    [ Real Kube-rs Executor ]
 (RestartPod / Scale / Cordon)
               │
               ▼
  [ Recovery Verification ] ──(Health re-check)─► [ Logged as Recovered / Failed ]
```

### 1. Rule-First Triage Engine (`triage.rs`)
- Known fault patterns (`CrashLoopBackOff`, `OOMKilled`, `DNSResolutionFailure`, `NodeDiskPressure`, `ContainerCannotStart`) are resolved **deterministically** with zero LLM API calls and zero AI cost.
- Only novel or unrecognized high-severity alerts escalate to the LLM path.

### 2. Action Allowlist & Structured Intent (`action.rs` & `llm.rs`)
- The LLM has **zero shell or raw command execution authority**.
- System prompts enforce structured JSON output matching the `LlmResponse` schema, which deserializes directly into a strongly-typed Rust `Action` enum:
  - `RestartPod { pod, namespace }`
  - `ScaleDeployment { deployment, target_replicas, namespace }`
  - `CordonNode { node }`
  - `DeleteNamespace { namespace }`
  - `ExecCommand { pod, command }`
  - `ModifyRbac { resource }`
  - `LogReviewNeeded { reason }`
  - `None`
- Hallucinated actions, raw bash syntax (`kubectl delete`), or malformed JSON are immediately rejected and trigger **Local Fallback Mode**.

### 3. Remediation Guard & Operational Circuit Breaker (`guard.rs`)
- Sits before OPA and execution to stop looping remediations and malicious alert storms.
- Enforces 3 hardcoded circuit breakers:
  - **Per-Resource Limit**: Max 3 actions on the same resource within a 10-minute rolling window.
  - **Incident Budget**: Max 5 total actions per incident ID.
  - **Cooldown**: 60-second mandatory wait between actions on the same resource.
- Exceeding thresholds halts execution, marks the incident status as `requires_human_intervention`, and emits an outbound notification webhook (Slack/PagerDuty).

### 4. Fail-Closed OPA Policy Engine (`policy.rs`)
- Every proposed mutation (whether from rules, LLM, or human override) must pass an HTTP authorization query against Open Policy Agent (`OPA_URL`).
- **Fail-Closed Constraint**: If the OPA daemon is unreachable, times out (500ms), returns a non-200 HTTP status, or returns a response missing `"result": true`, Cheezer strictly defaults to **FAIL-CLOSED (DENY / false)**.

### 5. TOCTOU Revalidation & Post-Remediation Verification (`executor.rs`)
- **Time-of-Check to Time-of-Use (TOCTOU) Protection**: `revalidate_state` queries the Kubernetes API via `kube-rs` immediately before execution. If a pod has self-resolved (`Running` & `Ready`) or no longer exists, execution aborts with status `Aborted_StaleState` before touching OPA or the executor.
- **Recovery Verification**: `verify_recovery` fetches cluster health post-mutation to confirm whether the resource recovered (`Recovered` or `Failed`).

### 6. Real-Time Web Dashboard & Human Approval Gateway (`dashboard.rs`)
- Embedded single-binary web interface mounted at `/dashboard` (styled with Tailwind CSS CDN and HTMX).
- Live polls `/api/incidents` every 2 seconds to render active incidents, circuit breaker locks, and remediation history.
- Provides an **"Approve & Execute"** button for locked incidents (`POST /api/incidents/{id}/approve`). Human approvals **MUST still pass through OPA policy checks** before hitting `executor.rs`.

---

## ⚡ Quickstart & Testing

### Running Tests
Cheezer includes a 17-test suite using `wiremock` and `kube-rs` dry runs for fast, offline-capable verification:

```bash
cd cheezer-core
cargo test -- --nocapture
```

### Accessing the Web Dashboard
Build and run the release binary:

```bash
cargo run --release
```

Navigate to `http://localhost:9090/dashboard` in your browser.

---

## 📁 Repository Structure

- `cheezer-core/src/ingest.rs`: Webhook ingestion and `x-api-key` validation.
- `cheezer-core/src/triage.rs`: Rule matching, heuristic scoring, and triage state machine.
- `cheezer-core/src/llm.rs`: OpenAI/Groq API client with structured JSON parsing.
- `cheezer-core/src/fallback.rs`: Local fallback engine for offline resilience.
- `cheezer-core/src/policy.rs`: HTTP OPA policy gate with fail-closed enforcement.
- `cheezer-core/src/guard.rs`: Remediation Guard, circuit breakers, and rate limits.
- `cheezer-core/src/executor.rs`: Real `kube-rs` cluster mutations, TOCTOU check, and verification.
- `cheezer-core/src/store.rs`: SQLite WAL persistence for incidents and audit logs.
- `cheezer-core/src/dashboard.rs`: Axum web UI, incidents JSON API, and human approval gateway.
- `cheezer-core/src/watchdog.rs`: High-availability active-passive leader election loop.
- `docs/ARCHITECTURE.md`: In-depth architecture specification and state transitions.
- `docs/BUILD_GUIDE.md`: Build, configuration, and environment variables guide.
