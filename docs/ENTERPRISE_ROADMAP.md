# Cheezer Enterprise Roadmap

While Cheezer's v1 engine proves the viability of autonomous, AI-driven incident remediation via LLMs, deploying it into a true enterprise SRE environment requires mature defensive mechanisms.

Our immediate engineering roadmap focuses heavily on **control, visibility, and safety**:

## 1. Blast Radius Control (Disruption Budgets)
**The Challenge:** A misconfigured deployment triggering an alert storm could theoretically cause Cheezer to process all alerts and mutate a large segment of the cluster simultaneously, causing a self-inflicted outage.
**The Roadmap:** Implement strict **Disruption Budgets** and concurrency limits. The engine will enforce hard algorithmic limits (e.g., "never mutate more than 10% of total nodes simultaneously" or "maximum 3 pod restarts per namespace within 10 minutes").

## 2. Operator Telemetry (Self-Monitoring)
**The Challenge:** Cheezer consumes extensive monitoring data (Grafana/Prometheus) but currently does not produce its own. We need to monitor Cheezer's internal health.
**The Roadmap:** Expose a dedicated `/metrics` endpoint for Prometheus to scrape. This will provide SLIs (Service Level Indicators) for Cheezer's own operational health, tracking metrics like LLM router latency, OPA policy evaluation times, policy rejection rates, and successful versus failed mutations.

## 3. Native Auditability (Kubernetes Events)
**The Challenge:** Terminal stdout logs and database entries are useful, but Kubernetes operators and engineers primarily rely on the native Kubernetes event log.
**The Roadmap:** Whenever Cheezer mutates a resource (e.g., cordons a node, restarts a pod), it will emit standard Kubernetes `Event` objects tied directly to that resource. For example: `Reason: CheezerRemediation`, `Message: Restarted due to OOMKilled alert`. This bridges the gap between AI actions and native K8s observability.

## 4. Stateful Throttling (Flap Detection & Cool-Downs)
**The Challenge:** We currently lack granular resource-specific cool-downs. If a pod is fundamentally broken (e.g., `CrashLoopBackOff`), Cheezer could get stuck in an infinite loop attempting to remediate it.
**The Roadmap:** Introduce an internal cache with strict backoff timers per resource. If a specific resource requires intervention three times in five minutes, Cheezer will halt mutations on that target, fire a `LogReviewNeeded` alert, and safely escalate the issue to a human on-call engineer.
