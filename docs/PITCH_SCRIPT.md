# Cheezer - 2 Minute Pitch Script

**[0:00 - 0:30] The Hook & The Problem**
"Hi everyone, we’re team Cheezer. Every day, SREs get woken up at 3 AM to handle mundane alerts: a pod is OOMKilled, a deployment gets stuck, or a node runs out of memory. They open a dashboard, look at the logs, restart the pod, and go back to sleep. It’s manual, it’s exhausting, and it doesn’t scale. 
What if you could give your Kubernetes cluster a brain?"

**[0:30 - 1:15] The Solution & Demo**
"Enter Cheezer: an autonomous, LLM-driven incident responder. Cheezer directly hooks into your Grafana alerts and OpenTelemetry traces. When an alert fires, Cheezer’s AI engine ingests the logs, identifies the root cause, and directly executes a remediation—like restarting a pod or scaling a deployment. 
*(Point to demo)* Here, you can see Cheezer instantly diagnosing a crash loop and successfully restoring the workload, all while adhering to strict Open Policy Agent (OPA) security guardrails, so the AI never runs wild."

**[1:15 - 1:45] The 'Enterprise Roadmap' (Self-Awareness)**
"Now, we know what the seasoned SREs in the room are thinking: *'An AI taking control of my cluster? What about blast radius? What if it loops?'* 
We completely agree. To make this production-ready for true enterprise environments, our immediate roadmap focuses purely on defense:
1. **Blast Radius Control:** Implementing strict Disruption Budgets so Cheezer can never mutate more than 10% of nodes at once.
2. **Stateful Throttling:** Adding flap-detection. If a pod crashes 3 times in 5 minutes, Cheezer stops touching it and escalates to a human.
3. **Native Auditability:** Cheezer will emit native Kubernetes `Events` detailing exactly *why* it took action, and we’re adding an internal `/metrics` endpoint so you can monitor the AI’s latency and success rates directly in Prometheus."

**[1:45 - 2:00] The Closing**
"Cheezer proves that AI can safely move beyond just answering questions, and actually *fix* your infrastructure while you sleep. We’ve built the engine, mapped out the defensive roadmap, and we’re ready to revolutionize incident response. Thank you."
