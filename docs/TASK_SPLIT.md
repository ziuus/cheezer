# Task Split

## Triage Core
- `ingest.rs`: Axum webhook receiver, JSON deserialization.
- `triage.rs`: Match known signatures (`CrashLoopBackOff`, etc).
- `store.rs`: SQLite WAL persistence.

## AI & Policy
- `llm.rs`: LLM invocation, retry, parsing.
- `fallback.rs`: Deterministic rule fallback on LLM failure.
- `policy.rs`: OPA validation logic (`opa run --server`).

## Execution & Resilience
- `executor.rs`: `kube-rs` logic.
- `watchdog.rs`: Primary/backup active-passive failover.
