# Cheezer

Cheezer is an autonomous, out-of-band Kubernetes incident-remediation engine. It acts as an intelligent, policy-gated automation layer that triages alerts, resolves known issues deterministically (to save costs and ensure speed), and escalates novel or complex incidents to an LLM.

## Architecture & Attribution Note
The failure-pattern taxonomy (CrashLoopBackOff, OOMKilled, ImagePullBackOff, etc.) and the general "rule-first, AI-escalation-for-novel-cases" pattern are inspired by the public architecture of two Apache 2.0 licensed open-source projects: [K8sGPT](https://github.com/k8sgpt-ai/k8sgpt) and [HolmesGPT](https://github.com/robusta-dev/holmes). This project is an original implementation of those conceptual patterns built for the "Automation for Good — IT Security & Cyber Resilience" hackathon track.

## Core Features
1. **Rule-First Triage Pipeline:** Known signatures (OOMKilled, CrashLoopBackOff) are resolved automatically via code.
2. **AI Escalation:** Only novel, high-severity issues escalate to the LLM (with a local fallback if disconnected).
3. **OPA Policy Gate:** Every action (rule-based or AI-based) is vetted by an Open Policy Agent (OPA) server.
4. **Resilient Architecture:** Runs completely out-of-band with a simple primary/backup active-passive failover watchdog.
5. **SQLite WAL Storage:** All triages and actions are logged persistently.

## Getting Started

See `docs/BUILD_GUIDE.md` for instructions on compiling and running Cheezer.
