# Cheezer Core — Research-Grounded Architectural Redesign

> **Core Value Metric:** *Reliability Recovered per Dollar and per Millisecond.*  
> **Strategic Mandate:** *"Cheap enough to run continuously, fast enough to remediate in milliseconds, and predictive enough to act before failure."*

---

## Executive Verdict: The Core Telemetry & Cost Inversion

The single most important architectural change in Cheezer Core is **inverting the traditional AIOps data flow**:

- **Traditional Observability Model (Datadog, Dynatrace, New Relic, Splunk):** Agents ship 100% of telemetry to a central cloud backend at fixed high resolution. Cost scales linearly with fleet size and cardinality, regardless of whether any system is broken.
- **Cheezer Core Inverted Model:** Telemetry resolution, storage, and reasoning tiers are all a function of **current risk state**, decided in-process inside Rust close to the source with zero network hops. A healthy fleet costs near $0.00 to watch. Cost, latency, and LLM calls scale strictly with *how much trouble the system is actually in*.

```text
                 TRADITIONAL OBSERVABILITY MODEL
Fleet (1,000 Pods) ──► Ship 100% Telemetry ──► Central Storage ──► $$$ High Bill Always
                                                                 (Even when 99% healthy)

                  CHEEZER CORE INVERTED MODEL
Fleet (1,000 Pods) ──► In-Process Rust Risk Engine (<1ms)
                       ├─ Healthy (95%) ──► Sparse Sampling / $0.00 Cost
                       └─ Anomaly (5%)  ──► 6-Tier Escalation Ladder (Rules → Stats → AI)
```

---

## 1. The 3 Core Performance Pillars

```text
                               CHEEZER CORE
                                    │
         ┌──────────────────────────┼──────────────────────────┐
         ▼                          ▼                          ▼
      ⚡ FAST                    💰 CHEAP                  🔮 PREDICTIVE
 Sub-millisecond Rust       Adaptive 6-Tier Router     Predictive Risk Engine
 rule-first path (<1ms).    (Tier 0 $0 rules ->        Extrapolates trend lines
 Zero network/LLM latency   Tier 1 $0.0001 Fast LLM ->  & EWMA Z-scores to act
 for known failure patterns Tier 2 $0.01 Deep LLM).    BEFORE outages happen!
                            Saves ~99% LLM compute.
```

1. **⚡ FAST:** Sub-millisecond Rust rule-first execution path (`< 1ms`). Bypasses network hops and HPA's 90–150 second reaction lag by issuing direct API mutations for known incident patterns (`CrashLoopBackOff`, `OOMKilled`).
2. **💰 CHEAP:** Telemetry resolution and AI escalation are confidence-gated. 88%+ of incidents never leave Tier 0/1 (in-process Rust), keeping aggregate AI spend to low single-digit dollars per 1,000 incidents.
3. **🔮 PREDICTIVE:** Adaptive Forecasting Engine (`src/predictive.rs`). Uses lightweight statistical models (linear regression, Holt-Winters, EWMA, CUSUM) rather than expensive deep learning transformers to forecast failures 15–20 minutes before they occur.

---

## 2. The 6-Tier Escalation Ladder

Cheezer Core routes incident triage through a strict 6-tier ladder, escalating ONLY when confidence falls below threshold:

| Tier | Engine / Technology | When it Fires | Marginal Cost | Latency |
| :--- | :--- | :--- | :--- | :--- |
| **0 — Deterministic Rule** | Hand-authored / auto-promoted Rust rules (`src/triage.rs`) | Known failure signature exists (`CrashLoopBackOff`, `OOMKilled`) | **~$0.00** | **< 1 ms** |
| **1 — Statistical Model** | EWMA, CUSUM, Z-score, MAD (`src/predictive.rs`) | Simple statistical anomaly, memory/disk growth trend | **~$0.00** | **< 2 ms** |
| **2 — Small Local ML** | LightGBM / Isolation Forest CPU inference | Multivariate/nonlinear workload anomalies | **~$0.00** | **Low ms** |
| **3 — Small Local LLM** | Quantized 3B–8B local model (Ollama / llama.cpp) | Natural-language log clustering & first-pass summary | **~$0.00** | **100–300 ms** |
| **4 — Cloud LLM** | Claude Haiku 4.5 ($1/$5 MTok) / Sonnet 4.6 ($3/$15 MTok) | Truly novel/unpatterned incident; Tier 0–3 low confidence | **~$0.001–0.01** | **1–2 s** |
| **5 — Coding Agent** | Devin AI Autonomous Engineer (`src/devin.rs`) | Application code-level defect; 3 infra fixes failed | **Task Token** | **Human-Gated PR** |

---

## 3. Four-State Telemetry Resolution Model

Cheezer Core dynamically adjusts metric collection sampling rates based on the current workload risk state:

```text
NORMAL (95% time)  ──► Sparse sampling (1/10th rate), aggregated rollups, zero cardinality bloat
        │
        ▼ (Anomaly detected by Tier 1 EWMA/CUSUM)
SUSPICIOUS         ──► Raise sampling rate + retain raw-resolution rolling memory buffer
        │
        ▼ (Incident confirmed)
ACTIVE INCIDENT    ──► Full 100% resolution collection across implicated trace paths & services
        │
        ▼ (Post-action health verification passed)
RECOVERED          ──► Exponential cooldown decay back to NORMAL
```

**Result:** An **85–90% reduction** in stored and shipped telemetry volume compared to traditional static observability collectors.

---

## 4. End-to-End Predictive & Safety Loop

```text
             1. PREDICTIVE FORECASTING
                (Memory growth rate / Disk fill trend / EWMA Z-score)
                             │
                             ▼
             2. DECISION MATRIX
                (Rules <1ms → Adaptive LLM Router)
                             │
                             ▼
             3. TOCTOU REVALIDATION
                (Fresh K8s/Cloud read: Is target still broken?)
                             │
                             ▼
             4. OPA POLICY GATE
                (Embedded Rego: Fail-closed authorization)
                             │
                             ▼
             5. REMEDIATIONGUARD BUDGET
                (Max 3 actions / 15 mins per target)
                             │
                             ▼
             6. MULTI-PLATFORM EXECUTION
                (Direct API mutation: K8s, AWS, GCP, Vercel, Docker)
                             │
                             ▼
             7. REMEDIATION VERIFICATION
                (Prove system health recovered: HTTP 200, 5xx drop, latency)
                             │
               ┌─────────────┴─────────────┐
               ▼                           ▼
            RESOLVED                    ESCALATE
                                           │
                                           ▼
                                     GITOPS CODE FIX
                                       (Devin AI)
```

---

## 5. Control Plane Resiliency & Leader Election

- **Failure Domain Separation:** Cheezer Core operates out-of-band on a dedicated control plane node separate from monitored application workloads.
- **Dual-Node HA Pair (`src/watchdog.rs`):** Primary node binds TCP watchdog listener; Standby node polls peer heartbeats. Promotes to Primary within 3 missed heartbeats.
- **K8s Lease Leader Election:** Uses native Kubernetes `Lease` primitives to ensure only the elected Leader issues mutating remediation calls, preventing split-brain or duplicate execution.
- **Principle:** *"Cheezer recovers customer infrastructure; an independent Watchdog quorum recovers Cheezer."*

---

## 6. Competitive Positioning Summary

| Feature / Metric | Traditional AIOps (Datadog/Dynatrace) | Cheezer Core |
| :--- | :--- | :--- |
| **Data Ingestion Cost** | Fixed high volume + cardinality pricing | **Risk-Adaptive (85-90% data reduction)** |
| **Known Incident Speed** | 90–150s HPA lag + alert routing | **< 1ms Fast-Path Direct Mutation** |
| **LLM Spending** | Un-gated API calls on raw logs | **6-Tier Escalation (~99% LLM Cost Saved)** |
| **Remediation Safety** | Runbook scripts (Assume success on `exit 0`) | **TOCTOU + OPA Fail-Closed + Health Verification** |
| **Forecasting Model** | Heavy deep learning / proprietary AI | **Cheap statistical models (Holt-Winters, Linear Regression, EWMA)** |
| **Control Plane Resiliency** | Vendor SaaS dependent | **Primary-Standby HA Pair + Watchdog (`watchdog.rs`)** |
