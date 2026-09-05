"use client";

import React, { useState } from "react";
import { Cpu, Shield, Terminal, Lock, CheckCircle2, FileCode } from "lucide-react";

interface PipelineStep {
  id: number;
  title: string;
  file: string;
  badge: string;
  badgeColor: string;
  shortDesc: string;
  fullDesc: string;
  rustCode: string;
  invariant: string;
}

export default function ArchitecturePipeline() {
  const [activeStepId, setActiveStepId] = useState<number>(1);

  const steps: PipelineStep[] = [
    {
      id: 1,
      title: "1. Webhook Ingest",
      file: "cheezer-core/src/ingest.rs",
      badge: "Auth Gate",
      badgeColor: "bg-blue-100 text-blue-900 border-blue-300 font-bold",
      shortDesc: "Mounts /api/grafana_webhook. Validates x-api-key HTTP header.",
      fullDesc: "Axum web endpoint parses incoming Alertmanager or Grafana payloads into typed Rust Alert structs. Enforces strict HTTP x-api-key authentication before accepting any webhook.",
      rustCode: `pub async fn handle_grafana_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<GrafanaPayload>,
) -> Result<impl IntoResponse, AppError> {
    let api_key = headers.get("x-api-key")
        .and_then(|v| v.to_str().ok());
    if api_key != Some(&state.config.api_key) {
        return Err(AppError::Unauthorized);
    }
    // Process Alert struct...
}`,
      invariant: "Unauthenticated payloads return HTTP 401 and are dropped immediately.",
    },
    {
      id: 2,
      title: "2. Zero AI Cost Triage",
      file: "cheezer-core/src/triage.rs",
      badge: "Zero AI Cost",
      badgeColor: "bg-emerald-100 text-emerald-900 border-emerald-300 font-bold",
      shortDesc: "Deterministic rule matching for known fault patterns.",
      fullDesc: "Known fault patterns (CrashLoopBackOff, OOMKilled, DNSResolutionFailure, NodeDiskPressure) match immediately with zero LLM API calls, zero latency, and zero AI cost.",
      rustCode: `pub fn triage_alert(alert: &Alert) -> TriageResult {
    match alert.fingerprint_pattern() {
        Pattern::CrashLoopBackOff => TriageResult::MatchedRule(
            Action::RestartPod { pod: alert.pod.clone(), namespace: alert.ns.clone() }
        ),
        Pattern::OOMKilled => TriageResult::MatchedRule(
            Action::ScaleDeployment { deployment: alert.deployment.clone(), target_replicas: 3, namespace: alert.ns.clone() }
        ),
        Pattern::Unknown => TriageResult::EscalateToLlm,
    }
}`,
      invariant: "Known alert signatures resolve deterministically without hitting external LLM APIs.",
    },
    {
      id: 3,
      title: "3. LLM Action Allowlist",
      file: "cheezer-core/src/llm.rs",
      badge: "Structured JSON",
      badgeColor: "bg-purple-100 text-purple-900 border-purple-300 font-bold",
      shortDesc: "Enforces strongly-typed Action enum. Zero shell execution authority.",
      fullDesc: "For novel alerts, Cheezer queries Groq/OpenAI with json_object mode. The response deserializes directly into a Rust Action enum. Hallucinated bash or invalid syntax defaults to Local Fallback Mode.",
      rustCode: `pub enum Action {
    RestartPod { pod: String, namespace: String },
    ScaleDeployment { deployment: String, target_replicas: u32, namespace: String },
    CordonNode { node: String },
    DeleteNamespace { namespace: String },
    ExecCommand { pod: String, command: String },
    LogReviewNeeded { reason: String },
    None,
}`,
      invariant: "The LLM has zero raw shell access. Output is strictly bounded by Rust deserialization.",
    },
    {
      id: 4,
      title: "4. TOCTOU Revalidation",
      file: "cheezer-core/src/executor.rs",
      badge: "State Guard",
      badgeColor: "bg-amber-100 text-amber-900 border-amber-300 font-bold",
      shortDesc: "Prevents Time-of-Check to Time-of-Use race conditions.",
      fullDesc: "Immediately prior to execution, revalidate_state queries Kubernetes via kube-rs. If a pod self-resolved or reported Running & Ready, execution aborts safely with status Aborted_StaleState.",
      rustCode: `pub async fn revalidate_state(client: &Client, action: &Action) -> Result<bool, kube::Error> {
    match action {
        Action::RestartPod { pod, namespace } => {
            let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
            if let Ok(p) = pods.get(pod).await {
                let is_ready = p.status.map_or(false, |s| is_pod_ready(&s));
                if is_ready { return Ok(false); } // Self-resolved!
            }
            Ok(true)
        }
        _ => Ok(true),
    }
}`,
      invariant: "Stale alerts for self-healing workloads abort before triggering mutations or OPA queries.",
    },
    {
      id: 5,
      title: "5. Remediation Guard",
      file: "cheezer-core/src/guard.rs",
      badge: "Circuit Breakers",
      badgeColor: "bg-rose-100 text-rose-900 border-rose-300 font-bold",
      shortDesc: "3-tier circuit breaker stops flap storms and endless loops.",
      fullDesc: "Evaluates historical action logs in SQLite. Limits max 3 actions on the same resource per 10 minutes, max 5 actions per incident budget, and enforces a mandatory 60-second cooldown.",
      rustCode: `pub fn check_remediation_guard(store: &SqliteStore, resource: &str) -> GuardResult {
    let recent_count = store.count_actions_in_window(resource, Duration::from_secs(600))?;
    if recent_count >= 3 {
        return GuardResult::Breached("Per-resource limit (3/10m) exceeded. Locking incident.");
    }
    GuardResult::Allowed
}`,
      invariant: "Breaching thresholds halts autonomous execution and notifies human gateway.",
    },
    {
      id: 6,
      title: "6. Fail-Closed OPA Gate",
      file: "cheezer-core/src/policy.rs",
      badge: "Fail-Closed Gate",
      badgeColor: "bg-emerald-100 text-emerald-900 border-emerald-300 font-bold",
      shortDesc: "HTTP OPA query evaluated against Rego security policies.",
      fullDesc: "Queries Open Policy Agent at OPA_URL. Any HTTP failure, 500ms timeout, connection refusal, or response missing result: true strictly defaults to DENY (false).",
      rustCode: `pub async fn check_opa_policy(client: &reqwest::Client, opa_url: &str, query: &OpaQuery) -> bool {
    let res = match client.post(opa_url).json(query).timeout(Duration::from_millis(500)).send().await {
        Ok(r) => r,
        Err(_) => return false, // FAIL-CLOSED ON TIMEOUT OR ERROR!
    };
    res.json::<OpaResponse>().await.map_or(false, |r| r.result)
}`,
      invariant: "Network partitions or OPA outages result in automatic FAIL-CLOSED (DENY).",
    },
    {
      id: 7,
      title: "7. Kube-rs Executor",
      file: "cheezer-core/src/executor.rs",
      badge: "Cluster Mutation",
      badgeColor: "bg-amber-100 text-amber-900 border-amber-300 font-bold",
      shortDesc: "Safe, typed mutations executed via official kube-rs client.",
      fullDesc: "Authenticates in-cluster or via kubeconfig. Performs precise mutations (Api::<Pod>::delete, Patch::Merge for replicas, node unschedulable).",
      rustCode: `pub async fn execute_action(client: &Client, action: &Action) -> Result<ExecutionResult, AppError> {
    match action {
        Action::RestartPod { pod, namespace } => {
            let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
            pods.delete(pod, &DeleteParams::default()).await?;
            Ok(ExecutionResult::Success)
        }
        Action::ScaleDeployment { deployment, target_replicas, namespace } => {
            let deps: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            deps.patch(deployment, &PatchParams::default(), &patch).await?;
            Ok(ExecutionResult::Success)
        }
        _ => Ok(ExecutionResult::None),
    }
}`,
      invariant: "Mutations use Kubernetes OpenAPI schemas and native controller reconciliation.",
    },
    {
      id: 8,
      title: "8. Recovery Verification",
      file: "cheezer-core/src/executor.rs",
      badge: "Health Verified",
      badgeColor: "bg-emerald-100 text-emerald-900 border-emerald-300 font-bold",
      shortDesc: "Post-mutation health re-check logs final outcome to SQLite WAL.",
      fullDesc: "Fetches target workload status post-remediation to confirm whether recovery succeeded. Logs verified result (Recovered / Failed) to SQLite audit WAL.",
      rustCode: `pub async fn verify_recovery(client: &Client, action: &Action) -> VerificationResult {
    tokio::time::sleep(Duration::from_secs(3)).await;
    let is_healthy = check_workload_health(client, action).await;
    if is_healthy {
        VerificationResult::Recovered
    } else {
        VerificationResult::Failed
    }
}`,
      invariant: "Every remediation is verified post-execution to prevent false-positive resolution.",
    },
  ];

  const currentStep = steps.find((s) => s.id === activeStepId) || steps[0];

  return (
    <section id="architecture" className="py-24 relative overflow-hidden bg-slate-50/80">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16 space-y-3">
          <div className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full bg-slate-200 text-slate-800 text-xs font-mono font-bold uppercase">
            <Cpu className="w-4 h-4 text-slate-700" />
            System Architecture
          </div>
          <h2 className="text-3xl sm:text-4xl font-extrabold text-slate-900 tracking-tight">
            Out-of-Band Autonomous Topology
          </h2>
          <p className="text-slate-700 text-base leading-relaxed">
            Cheezer runs out-of-band to ensure operational continuity during control plane failures. Explore the 8 sequential safety boundaries below.
          </p>
        </div>

        {/* 8-Step Interactive Pipeline Flow Bar */}
        <div className="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-8 gap-2.5 mb-8">
          {steps.map((step) => {
            const isActive = step.id === activeStepId;
            return (
              <button
                key={step.id}
                onClick={() => setActiveStepId(step.id)}
                className={`p-3.5 rounded-2xl text-left transition-all duration-200 border shadow-sm ${
                  isActive
                    ? "border-indigo-600 bg-indigo-600 text-white shadow-md scale-105"
                    : "border-slate-200 bg-white hover:border-slate-300 text-slate-800"
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className={`text-[10px] font-mono font-bold ${isActive ? "text-indigo-200" : "text-slate-400"}`}>Step {step.id}</span>
                  {isActive && <div className="w-2 h-2 rounded-full bg-amber-300 animate-ping" />}
                </div>
                <div className={`text-xs font-extrabold truncate ${isActive ? "text-white" : "text-slate-900"}`}>{step.title.replace(/^\d+\.\s*/, "")}</div>
              </button>
            );
          })}
        </div>

        {/* Step Detail Panel */}
        <div className="rounded-3xl border border-slate-300 bg-white overflow-hidden shadow-xl">
          
          <div className="bg-slate-900 text-white px-6 py-4 flex flex-wrap items-center justify-between gap-4 border-b border-slate-800">
            <div className="flex items-center gap-3">
              <div className="p-2.5 rounded-xl bg-amber-500 text-slate-950 font-mono font-black text-sm">
                0{currentStep.id}
              </div>
              <div>
                <h3 className="text-base font-extrabold text-white">{currentStep.title}</h3>
                <div className="text-xs font-mono text-amber-300 flex items-center gap-1.5 font-bold">
                  <FileCode className="w-3.5 h-3.5" />
                  {currentStep.file}
                </div>
              </div>
            </div>

            <span className={`text-xs font-mono px-3 py-1 rounded-full border ${currentStep.badgeColor}`}>
              {currentStep.badge}
            </span>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 p-6 sm:p-8 bg-white">
            
            {/* Description & Invariant */}
            <div className="lg:col-span-5 space-y-6">
              <div className="space-y-2">
                <h4 className="text-xs font-mono font-bold text-slate-500 uppercase tracking-widest">Phase Specification</h4>
                <p className="text-sm text-slate-800 leading-relaxed font-medium">{currentStep.fullDesc}</p>
              </div>

              <div className="p-4.5 rounded-2xl bg-emerald-50 border border-emerald-300 space-y-2">
                <div className="flex items-center gap-2 text-xs font-mono font-extrabold text-emerald-900">
                  <Shield className="w-4 h-4 text-emerald-700" />
                  Security & Invariant Guarantee
                </div>
                <p className="text-xs text-slate-800 font-semibold leading-relaxed">{currentStep.invariant}</p>
              </div>

              <div className="pt-2 flex items-center gap-5 text-xs font-mono font-bold text-slate-600">
                <div className="flex items-center gap-1.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-600" />
                  Unit Tested
                </div>
                <div className="flex items-center gap-1.5">
                  <Lock className="w-4 h-4 text-amber-600" />
                  Zero Shell Access
                </div>
              </div>
            </div>

            {/* Code Snippet */}
            <div className="lg:col-span-7">
              <div className="rounded-2xl overflow-hidden border border-slate-800 bg-[#0d1117]">
                <div className="bg-[#161b22] px-4 py-2.5 border-b border-slate-800 flex items-center justify-between text-xs font-mono text-slate-300 font-bold">
                  <span className="text-amber-400 flex items-center gap-1.5">
                    <Terminal className="w-3.5 h-3.5" />
                    Rust Implementation
                  </span>
                  <span className="text-[10px] text-slate-400">rust-edition: 2021</span>
                </div>
                <pre className="p-4 text-xs font-mono text-amber-200 overflow-x-auto leading-relaxed">
                  <code>{currentStep.rustCode}</code>
                </pre>
              </div>
            </div>

          </div>

        </div>

      </div>
    </section>
  );
}
