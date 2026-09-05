# Cheezer Core — Hackathon & SRE Keynote Pitch Script

> **Product Strategy:** **⚡ FAST** (Sub-ms Rust) · **💰 CHEAP** (~99% LLM Cost Saved) · **🔮 PREDICTIVE** (Failure Forecasting Engine)  
> **Core Identity:** *"Cheap enough to run continuously, fast enough to remediate in milliseconds, and predictive enough to act before failure."*  
> **Technical Tagline:** **Predict → Decide → Revalidate → Authorize → Remediate → Verify.**

---

## 🎙️ 2-Minute Pitch Script

### **[0:00 - 0:25] The Hook: Predictive vs. Reactive Observability**
"Hi everyone, we’re team Cheezer. 

Every day, engineering teams pay millions to platforms like Dynatrace, Datadog, or PagerDuty to watch their systems break. But traditional observability is purely **reactive**—it alerts you *after* a pod has crashed, *after* memory has filled, or *after* customers experience downtime.

We built **Cheezer Core**. Cheezer isn't another monitoring dashboard or a cheaper Dynatrace. **Cheezer is an autonomous recovery control plane built on three metrics: cheap enough to run continuously, fast enough to remediate in milliseconds, and predictive enough to act before failure.**"

---

### **[0:25 - 1:15] The Predictive Loop & 4-Tier Decision Matrix**
"*(Point to live Cheezer Material 3 dashboard on screen)*

Look at how Cheezer handles an incident. Instead of waiting for a crash:

1. **Predictive Failure Engine (`predictive.rs`):** Calculates linear memory growth rates and EWMA baseline deviations. E.g., *"Pod `api-gateway` has an 87% probability of an OOMKilled breach in 18 minutes."* Cheezer initiates **preventive remediation** before the outage happens!
2. **Sub-Millisecond Rust Fast-Path (<1ms):** Known failure patterns (`CrashLoopBackOff`, `OOMKilled`) execute in microseconds in memory with zero LLM API costs.
3. **Adaptive 4-Tier LLM Router:** Calls heavy LLMs (`gpt-4o`) ONLY for critical multi-service cascades. Lightweight novel alerts use fast models (`gpt-4o-mini`), saving **~99% of LLM compute costs**.
4. **TOCTOU Revalidation (12ms):** Re-queries live health right before execution. If the pod self-resolved, Cheezer aborts instantly.
5. **OPA Policy Gate (Rego):** Validates the proposed fix against fail-closed security policies. No unauthorized root execution, no deleting system namespaces.
6. **RemediationGuard Budget:** Enforces strict disruption budgets—max 3 actions per 15 minutes per target.
7. **First-Class Verification:** Cheezer doesn't consider an action successful because a command returned exit code 0. It queries 5xx error rates, pod health, and latency to **prove system health actually recovered**.
8. **Infra-to-Code Escalation:** If 3 infra remediations fail, Cheezer dispatches Devin AI to open a declarative GitOps Pull Request on GitHub!"

---

### **[1:15 - 1:45] Control Plane Self-Resilience (Cheezer HA Pair)**
"Now, every SRE asks: *'What happens if the Cheezer server itself dies?'*

We built Cheezer on a fundamental principle: **Cheezer recovers customer infrastructure; an independent Watchdog quorum recovers Cheezer.**

Cheezer operates out-of-band as a **Dual-Node HA Pair (Primary + Standby)** monitored by an independent TCP proof-of-life Watchdog (`src/watchdog.rs`). If Server A goes down, Server B detects 3 missed heartbeats and promotes itself to Primary instantly. Cheezer is not just self-healing infrastructure—it is a **self-preserving recovery control plane**."

---

### **[1:45 - 2:00] The Closing & Positioning**
"Existing tools can perform pieces of automation inside their vendor lock-in silos. **Cheezer's product is the recovery loop itself.**

AI recommends. Policy decides. Cheezer executes. Cheezer verifies.

Thank you!"

---

## 🛡️ Competitive Positioning & Objection Handling Guide for Judges

| Question / Objection | Traditional Tool Answer | Cheezer Core Answer |
| :--- | :--- | :--- |
| *"Dynatrace & PagerDuty already have remediation runbooks. Why Cheezer?"* | "They lock you into their ecosystem and run scripts when told." | "Dynatrace sells observability; PagerDuty sells incident routing. **Cheezer's sole identity is the vendor-neutral recovery loop.** We operate across K8s, AWS, Vercel, and Docker with TOCTOU safety, OPA policy gates, and verification." |
| *"What if AI makes a dangerous command on my cluster?"* | "Hope the script author was careful." | "**AI recommends. Policy decides.** All LLM outputs must pass embedded OPA Rego policies (`policy.rs`) and TOCTOU state revalidation before execution. Irresponsible actions are blocked fail-closed." |
| *"How do you know the fix worked?"* | "The bash script returned exit code 0." | "**Command success != System recovery.** Cheezer performs active post-remediation health probes (HTTP 200, 5xx error rate drop, pod readiness) before marking an incident resolved." |
| *"What if the problem is in the application code, not the server?"* | "Alert an engineer on call." | "If 3 infrastructure remediations fail, Cheezer bridges the infra-to-code gap by dispatching Devin AI to create a declarative GitOps Pull Request on GitHub." |
| *"What if Cheezer itself crashes?"* | "Your monitoring is dead." | "Cheezer runs out-of-band in a **Primary-Standby HA pair** (`watchdog.rs`). If the primary node dies, the standby node promotes automatically within 3 heartbeats." |
