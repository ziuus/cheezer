# Cheezer Core — Hackathon & SRE Keynote Pitch Script

> **Product Identity:** The Autonomous, Vendor-Neutral, Safety-First Recovery Control Plane  
> **Core Value:** *"Cheezer doesn't just observe failures or execute runbooks. It autonomously decides whether recovery is safe, performs the recovery, and proves that the system actually recovered."*

---

## 🎙️ 2-Minute Pitch Script

### **[0:00 - 0:25] The Hook: Beyond Observability**
"Hi everyone, we’re team Cheezer. 

Every day, engineering teams pay millions to platforms like Dynatrace, Datadog, or PagerDuty to watch their systems break. But when an incident hits at 3 AM, observability tools just send an alert. A human still has to wake up, re-verify if the issue is real, check policy, run a script, and manually verify if the system actually recovered.

We built **Cheezer Core**. Cheezer isn't another monitoring dashboard or a cheaper Dynatrace. **Cheezer is an autonomous recovery control plane that sits outside your infrastructure and safely takes systems from incident to verified recovery.**"

---

### **[0:25 - 1:15] The Demo & The Closed-Loop Core Engine**
"*(Point to live Cheezer Material 3 dashboard on screen)*

Look at how Cheezer handles an incident. When a Grafana or Prometheus alert fires, Cheezer doesn't blindly trigger a script. It executes a strict 7-stage closed recovery loop:

1. **Rule-First Diagnosis (<1ms):** Instantly matches known patterns (`CrashLoopBackOff`, `OOMKilled`) via Rust fast-paths, or escalates novel anomalies to an LLM signal classifier.
2. **TOCTOU Revalidation (12ms):** Before touching anything, Cheezer re-queries live cluster health. If the pod self-resolved, it aborts instantly with zero race conditions.
3. **OPA Policy Gate (Rego):** Validates the proposed fix against fail-closed security rules. No unauthorized root execution, no deleting system namespaces.
4. **RemediationGuard Budget:** Enforces strict disruption budgets—max 3 actions per 15 minutes per workload—stopping cascading alert storms.
5. **Multi-Platform Remediation:** Executes the fix natively across 19 platforms—Kubernetes, AWS, Vercel, GCP, or Docker.
6. **First-Class Verification:** **This is our biggest moat.** Cheezer doesn't consider an action successful because a command returned exit code 0. It queries 5xx error rates, pod health, and latency to **prove the system actually recovered**.
7. **Infra-to-Code Escalation:** If infrastructure remediation fails, Cheezer recognizes an application-level defect and automatically dispatches a task to Devin AI to open a GitOps Pull Request on GitHub!"

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
