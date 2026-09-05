# Cheezer Build, Configuration & Deployment Guide

This guide details environment setup, configuration variables, compilation, test execution, and deployment steps for the Cheezer incident-remediation engine.

---

## 📋 Prerequisites

- **Rust toolchain** (`rustc` & `cargo` 1.80+)
- **Open Policy Agent** (`opa` CLI) for running the local Rego policy daemon
- **Kubernetes cluster** (minikube, k3s, kind, or EKS/GKE) with local `~/.kube/config` or in-cluster ServiceAccount
- *(Optional)* **OpenAI / Groq API Key** for live LLM escalation testing

---

## ⚙️ Environment Variables Reference

Cheezer is configured dynamically at runtime using environment variables:

| Variable | Description | Default Value | Required? |
| :--- | :--- | :--- | :--- |
| `CHEEZER_API_KEY` | Secret token expected in `x-api-key` header for Grafana/Alertmanager webhooks | `hackathon-secret` | Optional |
| `LLM_API_URL` | OpenAI-compatible chat completions API URL | `https://api.openai.com/v1/chat/completions` | Optional |
| `LLM_API_KEY` | Bearer API key for OpenAI / Groq LLM service | `""` | Optional for live AI |
| `LLM_MODEL` | Target LLM model name | `gpt-4o-mini` | Optional |
| `OPA_URL` | Open Policy Agent authorization evaluation endpoint | `http://localhost:8181/v1/data/cheezer/authz/allow` | Optional |
| `MOCK_EXECUTOR` | Set `"true"` to simulate Kubernetes API mutations during dry-run testing | `"false"` | Optional |
| `MOCK_OPA_ENABLED` | Set `"true"` to use fast embedded Rego logic instead of HTTP OPA during offline tests | `"false"` | Optional |
| `MOCK_STALE_STATE` | Set `"true"` to simulate self-healing pod TOCTOU revalidation aborts in tests | `"false"` | Optional |
| `IGNORE_COOLDOWN` | Set `"true"` to bypass 60s resource cooldown during rapid test runs | `"false"` | Optional |

---

## 🚀 Building Cheezer

### 1. Compile Debug / Release Binary

```bash
cd cheezer-core

# Compile debug profile
cargo build

# Compile optimized release binary
cargo build --release
```

The resulting binary will be placed at `cheezer-core/target/release/cheezer-core`.

---

## 🧪 Running the Test Suite

Cheezer includes 17 offline-capable unit and integration tests using `wiremock` and `kube-rs` dry-run mocks:

```bash
cd cheezer-core
cargo test -- --nocapture
```

---

## 🏃 Running Cheezer Production Services

### Step 1: Start Open Policy Agent Daemon
Run OPA in server mode loading the security policy:

```bash
opa run --server policies/cheezer.rego
```

OPA will listen on `http://localhost:8181`.

### Step 2: Start Primary Engine Instance
Set environment variables and launch Cheezer in `primary` role:

```bash
export LLM_API_KEY="your-actual-groq-or-openai-key"
export LLM_API_URL="https://api.groq.com/openai/v1/chat/completions"
export LLM_MODEL="llama3-70b-8192"
export OPA_URL="http://localhost:8181/v1/data/cheezer/authz/allow"
export CHEEZER_API_KEY="production-webhook-secret"

./target/release/cheezer-core --role=primary
```

Cheezer will listen for webhooks on `http://0.0.0.0:9090` and start the watchdog heartbeat listener on port `9000`.

### Step 3: Start Backup Watchdog Instance (HA Failover)
In a separate terminal or backup node, start the backup process monitoring the primary:

```bash
./target/release/cheezer-core --role=backup --peer=127.0.0.1:9000
```

If the primary process crashes or loses network connectivity, the backup automatically assumes the `primary` role and opens webhook ingestion.

---

## 🖥️ Accessing the Web Dashboard

Open your web browser and navigate to:

```text
http://localhost:9090/dashboard
```

Features available in the dashboard:
- Real-time KPI summary (Total Incidents, Self-Remediated, Requires Approval, OPA Denials).
- Live incident stream auto-refreshing via HTMX polling every 2s.
- **"Approve & Execute"** button for incidents locked by RemediationGuard (`requires_human_intervention`).
- Complete remediation audit history table.

---

## 📡 Sending Test Webhook Alerts

Simulate an Alertmanager firing alert using `curl`:

```bash
curl -X POST http://localhost:9090/api/grafana_webhook \
  -H "Content-Type: application/json" \
  -H "x-api-key: production-webhook-secret" \
  -d '{
    "alerts": [
      {
        "status": "firing",
        "labels": {
          "alertname": "CrashLoopBackOff",
          "severity": "critical",
          "pod": "payment-service-pod-0",
          "namespace": "production"
        }
      }
    ]
  }'
```
