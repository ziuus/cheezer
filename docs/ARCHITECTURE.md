# Architecture

Cheezer runs outside the cluster to ensure it remains operational even during massive cluster failures.

## Components

1. **Ingest (`ingest.rs`)**: An Axum HTTP server that receives webhooks from Grafana/Alertmanager.
2. **Triage (`triage.rs`)**: A rule engine that evaluates alerts against known signatures (e.g. `OOMKilled`, `CrashLoopBackOff`). It includes a heuristic severity/novelty scorer to decide if an alert needs AI escalation.
3. **LLM Escalation (`llm.rs`)**: For unrecognized or highly novel alerts, Cheezer queries an LLM to recommend an action.
4. **Fallback (`fallback.rs`)**: If the LLM times out or is unreachable, this guarantees we fall back to a deterministic rule approach.
5. **Policy Engine (`policy.rs`)**: Contacts a local OPA HTTP server (`opa run --server`) to validate the proposed action against `policies/cheezer.rego`.
6. **Executor (`executor.rs`)**: Uses `kube-rs` to execute approved actions against the cluster.
7. **Store (`store.rs`)**: Uses SQLite in WAL mode to log incidents, triage decisions, and actions taken.
8. **Watchdog (`watchdog.rs`)**: A simple TCP-bind-first-wins leader election system to run a primary and backup instance.
