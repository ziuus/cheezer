"use client";

import React, { useState } from "react";
import { Shield, Zap, AlertTriangle, CheckCircle2, Play, RefreshCw, Lock, Server, UserCheck } from "lucide-react";
import confetti from "canvas-confetti";

interface Scenario {
  id: string;
  name: string;
  badge: string;
  badgeColor: string;
  podName: string;
  namespace: string;
  alertType: string;
  description: string;
  steps: {
    title: string;
    description: string;
    type: "rule" | "toctou" | "guard" | "opa" | "executor" | "verify" | "llm" | "breaker";
    status: "success" | "warning" | "error" | "info";
    duration: string;
    details: string;
  }[];
}

export default function TriageSimulator() {
  const [selectedScenarioId, setSelectedScenarioId] = useState<string>("crashloop");
  const [runningStep, setRunningStep] = useState<number>(-1);
  const [isExecuting, setIsExecuting] = useState<boolean>(false);
  const [humanApproved, setHumanApproved] = useState<boolean>(false);

  const scenarios: Scenario[] = [
    {
      id: "crashloop",
      name: "CrashLoopBackOff",
      badge: "Zero AI Cost",
      badgeColor: "bg-emerald-100 text-emerald-900 border-emerald-300 font-bold",
      podName: "payment-gateway-86f7b-9x2m",
      namespace: "production",
      alertType: "KubePodCrashLooping",
      description: "Standard pod crash pattern matched instantly by Rust rule engine. $0.00 LLM API cost.",
      steps: [
        {
          title: "1. Webhook Ingested",
          description: "Alertmanager webhook authenticated via x-api-key HTTP header.",
          type: "rule",
          status: "info",
          duration: "0.2ms",
          details: "Alert: KubePodCrashLooping | Severity: Critical | Target: payment-gateway-86f7b-9x2m",
        },
        {
          title: "2. Deterministic Rule Match",
          description: "triage.rs matched CrashLoopBackOff signature. Bypassing LLM.",
          type: "rule",
          status: "success",
          duration: "0.1ms",
          details: "Action Resolved: RestartPod { pod: 'payment-gateway-86f7b-9x2m', namespace: 'production' }",
        },
        {
          title: "3. TOCTOU Revalidation",
          description: "revalidate_state queried Kubernetes API via kube-rs.",
          type: "toctou",
          status: "success",
          duration: "14.2ms",
          details: "Pod Phase: Running | Container status: Ready = false (State confirmed valid for mutation)",
        },
        {
          title: "4. Remediation Guard Check",
          description: "guard.rs evaluated 3 operational circuit breakers.",
          type: "guard",
          status: "success",
          duration: "1.1ms",
          details: "Per-resource actions (1/3) | Incident budget (1/5) | Cooldown: OK (0s elapsed)",
        },
        {
          title: "5. Fail-Closed OPA Gate",
          description: "Queried Open Policy Agent HTTP daemon (policies/cheezer.rego).",
          type: "opa",
          status: "success",
          duration: "18.5ms",
          details: "POST /v1/data/cheezer/authz/allow -> result: true (Action Authorized)",
        },
        {
          title: "6. Kube-rs Execution & Verification",
          description: "Issued Api::<Pod>::delete(). Verified recovery.",
          type: "executor",
          status: "success",
          duration: "42.0ms",
          details: "Pod recreated -> Phase: Running | Ready: true (1/1) -> Incident Status: Recovered",
        },
      ],
    },
    {
      id: "oomkilled",
      name: "OOMKilled Scaling",
      badge: "Zero AI Cost",
      badgeColor: "bg-emerald-100 text-emerald-900 border-emerald-300 font-bold",
      podName: "analytics-worker-5c4d-7k1p",
      namespace: "data-pipeline",
      alertType: "KubeContainerOOMKilled",
      description: "Memory threshold breached. Cheezer scales deployment replicas to distribute pod memory pressure.",
      steps: [
        {
          title: "1. Webhook Ingested",
          description: "Grafana webhook alert received for high memory usage OOMKilled.",
          type: "rule",
          status: "info",
          duration: "0.3ms",
          details: "Alert: KubeContainerOOMKilled | Memory limit: 512Mi breached",
        },
        {
          title: "2. Deterministic Rule Match",
          description: "triage.rs mapped OOMKilled to deployment scaling remediation.",
          type: "rule",
          status: "success",
          duration: "0.1ms",
          details: "Action Resolved: ScaleDeployment { deployment: 'analytics-worker', target_replicas: 3, namespace: 'data-pipeline' }",
        },
        {
          title: "3. TOCTOU Revalidation",
          description: "revalidate_state checked current deployment replica status.",
          type: "toctou",
          status: "success",
          duration: "16.8ms",
          details: "Current Replicas: 1 | Target: 3 (State valid, scaling needed)",
        },
        {
          title: "4. Remediation Guard Check",
          description: "Checked rate limits and incident budget.",
          type: "guard",
          status: "success",
          duration: "0.9ms",
          details: "Per-resource actions (1/3) | Incident budget (1/5) | Budget OK",
        },
        {
          title: "5. OPA Rego Authorization",
          description: "Evaluated deployment scaling policy against OPA.",
          type: "opa",
          status: "success",
          duration: "15.1ms",
          details: "OPA Policy: allow_scale_deployment == true -> Result: ALLOWED",
        },
        {
          title: "6. Kube-rs Scale & Health Check",
          description: "Patched spec.replicas = 3. Verified node memory back to nominal.",
          type: "executor",
          status: "success",
          duration: "65.4ms",
          details: "Deployment scaled to 3/3 ready. Memory pressure relieved.",
        },
      ],
    },
    {
      id: "novel",
      name: "Novel DNS Failure",
      badge: "Structured LLM",
      badgeColor: "bg-blue-100 text-blue-900 border-blue-300 font-bold",
      podName: "core-dns-worker-node-04",
      namespace: "kube-system",
      alertType: "ClusterCoreDNSResolutionFailure",
      description: "Unrecognized alert signature pattern safely escalates to Groq/OpenAI LLM under strict JSON schema enforcement.",
      steps: [
        {
          title: "1. Webhook Ingested",
          description: "Received unrecognized high-severity alert.",
          type: "rule",
          status: "info",
          duration: "0.4ms",
          details: "Alert: ClusterCoreDNSResolutionFailure | No rule pattern match -> Escalate to LLM",
        },
        {
          title: "2. LLM Escalation & Schema Validation",
          description: "Structured JSON response deserialized into Rust Action enum.",
          type: "llm",
          status: "warning",
          duration: "340ms",
          details: "LLM Intent: Action::CordonNode { node: 'worker-node-04' } (No shell access)",
        },
        {
          title: "3. TOCTOU Node Check",
          description: "revalidate_state verified node state via kube-rs API.",
          type: "toctou",
          status: "success",
          duration: "18.2ms",
          details: "Node worker-node-04 status: Ready, unschedulable = false",
        },
        {
          title: "4. Remediation Guard Check",
          description: "Circuit breaker checked node cordon rate limits.",
          type: "guard",
          status: "success",
          duration: "1.2ms",
          details: "Node cordon budget OK | Cooldown OK",
        },
        {
          title: "5. Fail-Closed OPA Policy Query",
          description: "OPA Rego authorization check for CordonNode.",
          type: "opa",
          status: "success",
          duration: "21.0ms",
          details: "OPA Rego allow_cordon_node check -> Result: ALLOWED",
        },
        {
          title: "6. Kube-rs Cordon Mutation",
          description: "Node unschedulable set to true. Workloads safely drained.",
          type: "executor",
          status: "success",
          duration: "88.0ms",
          details: "Node worker-node-04 cordoned. DNS resolution stabilized.",
        },
      ],
    },
    {
      id: "circuitbreaker",
      name: "Flap Storm Lock",
      badge: "Defensive Lock",
      badgeColor: "bg-amber-100 text-amber-900 border-amber-300 font-bold",
      podName: "unstable-api-7b89-2m1q",
      namespace: "production",
      alertType: "KubePodFlappingRepeatedly",
      description: "Repeated pod crashes breach per-resource rate limit (3 actions/10m). Cheezer locks autonomous mode & requires human approval.",
      steps: [
        {
          title: "1. Webhook Ingested",
          description: "Incoming 4th alert payload for unstable-api pod.",
          type: "rule",
          status: "info",
          duration: "0.2ms",
          details: "Alert: KubePodFlappingRepeatedly | Target: unstable-api-7b89-2m1q",
        },
        {
          title: "2. Deterministic Rule Match",
          description: "triage.rs resolved candidate action: RestartPod.",
          type: "rule",
          status: "info",
          duration: "0.1ms",
          details: "Action Candidate: RestartPod { pod: 'unstable-api-7b89-2m1q' }",
        },
        {
          title: "3. TOCTOU Check",
          description: "Pod still crashlooping.",
          type: "toctou",
          status: "info",
          duration: "12.0ms",
          details: "Pod status confirmed broken",
        },
        {
          title: "4. Remediation Guard BREACHED!",
          description: "Per-resource limit EXCEEDED (3 actions in 10-minute window).",
          type: "breaker",
          status: "error",
          duration: "0.5ms",
          details: "LOCK TRIGGERED: Status -> requires_human_intervention | Notification Webhook sent to Slack",
        },
        {
          title: "5. Autonomous Execution Blocked",
          description: "Cheezer halts execution to prevent cluster flap storm.",
          type: "breaker",
          status: "warning",
          duration: "0.0ms",
          details: "Awaiting Human Gateway Approval via /dashboard",
        },
      ],
    },
  ];

  const currentScenario = scenarios.find((s) => s.id === selectedScenarioId) || scenarios[0];

  const triggerExecution = () => {
    setIsExecuting(true);
    setRunningStep(0);
    setHumanApproved(false);

    let stepIndex = 0;
    const interval = setInterval(() => {
      stepIndex++;
      if (stepIndex < currentScenario.steps.length) {
        setRunningStep(stepIndex);
      } else {
        clearInterval(interval);
        setIsExecuting(false);

        if (currentScenario.id !== "circuitbreaker") {
          confetti({
            particleCount: 80,
            spread: 60,
            origin: { y: 0.6 },
            colors: ["#d97706", "#059669", "#2563eb"],
          });
        }
      }
    }, 750);
  };

  const handleHumanApprove = () => {
    setHumanApproved(true);
    confetti({
      particleCount: 100,
      spread: 70,
      origin: { y: 0.6 },
      colors: ["#059669", "#d97706"],
    });
  };

  return (
    <section id="simulator" className="py-24 relative overflow-hidden bg-slate-100/60 border-t border-b border-slate-200">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16 space-y-3">
          <div className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full bg-indigo-50 border border-indigo-200 text-indigo-900 text-xs font-mono font-bold uppercase">
            <Zap className="w-4 h-4 text-indigo-600" />
            Interactive Triage Studio
          </div>
          <h2 className="text-3xl sm:text-4xl font-extrabold text-slate-900 tracking-tight">
            Simulate Real Incident Remediations
          </h2>
          <p className="text-slate-700 text-base leading-relaxed">
            Select an alert scenario below to watch Cheezer step through Rule Matching, TOCTOU State Validation, Remediation Guarding, OPA Rego Authorization, and Kube-rs Execution.
          </p>
        </div>

        {/* Scenario Selection Tabs */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
          {scenarios.map((sc) => {
            const isSelected = sc.id === selectedScenarioId;
            return (
              <button
                key={sc.id}
                onClick={() => {
                  setSelectedScenarioId(sc.id);
                  setRunningStep(-1);
                  setIsExecuting(false);
                  setHumanApproved(false);
                }}
                className={`p-5 rounded-2xl text-left transition-all duration-200 border shadow-sm ${
                  isSelected
                    ? "border-indigo-600 bg-indigo-600 text-white shadow-md"
                    : "border-slate-200 bg-white hover:border-slate-300 text-slate-800"
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className={`text-[10px] font-mono font-bold px-2 py-0.5 rounded border ${isSelected ? "bg-white/20 text-white border-white/30" : sc.badgeColor}`}>
                    {sc.badge}
                  </span>
                  {isSelected && <div className="w-2.5 h-2.5 rounded-full bg-amber-300 animate-ping" />}
                </div>
                <div className={`font-bold text-sm mb-1 ${isSelected ? "text-white" : "text-slate-900"}`}>{sc.name}</div>
                <div className={`text-xs line-clamp-2 ${isSelected ? "text-indigo-100" : "text-slate-600"}`}>{sc.description}</div>
              </button>
            );
          })}
        </div>

        {/* Main Studio Box */}
        <div className="rounded-3xl border border-slate-300 bg-white overflow-hidden shadow-xl">
          
          {/* Header Bar */}
          <div className="bg-slate-900 text-white px-6 py-4 flex flex-wrap items-center justify-between gap-4 border-b border-slate-800">
            <div className="flex items-center gap-3">
              <div className="p-2.5 rounded-xl bg-amber-500 text-slate-950">
                <Server className="w-5 h-5 font-bold" />
              </div>
              <div>
                <div className="text-[10px] font-mono text-slate-400 uppercase tracking-wider font-bold">Target Workload</div>
                <div className="text-sm font-bold font-mono text-white flex items-center gap-2">
                  {currentScenario.podName}
                  <span className="text-xs text-slate-400 font-normal">({currentScenario.namespace})</span>
                </div>
              </div>
            </div>

            <div className="flex items-center gap-4">
              <div className="text-right hidden sm:block font-mono text-xs">
                <div className="text-slate-400 font-bold">Triggered Alert</div>
                <div className="font-bold text-amber-400">{currentScenario.alertType}</div>
              </div>

              <button
                onClick={triggerExecution}
                disabled={isExecuting}
                className="px-6 py-3 rounded-xl bg-amber-500 hover:bg-amber-400 text-slate-950 font-bold text-xs shadow-md transition-all flex items-center gap-2 disabled:opacity-50"
              >
                {isExecuting ? <RefreshCw className="w-4 h-4 animate-spin text-slate-950" /> : <Play className="w-4 h-4 fill-slate-950" />}
                {isExecuting ? "Executing Triage..." : "Run Scenario Triage"}
              </button>
            </div>
          </div>

          {/* Body Split Grid */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 p-6 sm:p-8 bg-white">
            
            {/* Steps Timeline */}
            <div className="lg:col-span-7 space-y-3">
              <div className="text-xs font-mono font-bold text-slate-500 uppercase tracking-widest pb-1 flex justify-between">
                <span>Execution Pipeline Steps</span>
                <span>{runningStep >= 0 ? `Step ${Math.min(runningStep + 1, currentScenario.steps.length)} of ${currentScenario.steps.length}` : "Ready to run"}</span>
              </div>

              {currentScenario.steps.map((step, idx) => {
                const isStepActive = idx === runningStep;
                const isStepDone = runningStep > idx || runningStep === currentScenario.steps.length - 1;

                return (
                  <div
                    key={idx}
                    className={`p-4 rounded-xl border transition-all duration-200 ${
                      isStepActive
                        ? "bg-amber-50 border-amber-400 text-slate-900 shadow-md"
                        : isStepDone
                        ? "bg-slate-50 border-slate-200 text-slate-800"
                        : "bg-slate-50/50 border-slate-100 text-slate-400 opacity-60"
                    }`}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <div className="flex items-center gap-2.5 font-bold text-xs">
                        {isStepDone ? (
                          <CheckCircle2 className="w-4 h-4 text-emerald-600 shrink-0" />
                        ) : isStepActive ? (
                          <RefreshCw className="w-4 h-4 text-amber-600 animate-spin shrink-0" />
                        ) : (
                          <div className="w-4 h-4 rounded-full border border-slate-400 shrink-0" />
                        )}
                        <span className={isStepActive ? "text-amber-900 font-extrabold" : isStepDone ? "text-slate-900 font-bold" : "text-slate-500"}>
                          {step.title}
                        </span>
                      </div>
                      <span className="text-[11px] font-mono text-slate-500 font-bold">{step.duration}</span>
                    </div>

                    <p className="text-xs text-slate-700 pl-6 mb-1">{step.description}</p>
                    <div className="text-[11px] font-mono pl-6 text-slate-800 bg-slate-100 p-2.5 rounded-lg border border-slate-200 mt-2 font-semibold">
                      {step.details}
                    </div>
                  </div>
                );
              })}
            </div>

            {/* Visual State & Breakers */}
            <div className="lg:col-span-5 space-y-6">
              
              {/* Pod Health Visualizer */}
              <div className="p-5 rounded-2xl bg-slate-50 border border-slate-200 space-y-4">
                <div className="text-xs font-mono font-bold text-slate-600 uppercase tracking-widest flex items-center justify-between">
                  <span>Target Pod State</span>
                  <span className="text-[10px] text-amber-800 font-bold">kube-rs Monitor</span>
                </div>

                <div className="p-5 rounded-2xl bg-white border border-slate-200 text-center space-y-3 shadow-sm">
                  <div className="inline-flex p-3.5 rounded-2xl bg-amber-500/10 border border-amber-500/20">
                    <Server
                      className={`w-9 h-9 ${
                        runningStep === currentScenario.steps.length - 1 && currentScenario.id !== "circuitbreaker"
                          ? "text-emerald-600"
                          : currentScenario.id === "circuitbreaker" && runningStep >= 3
                          ? "text-rose-600"
                          : "text-amber-600 animate-pulse"
                      }`}
                    />
                  </div>

                  <div>
                    <div className="text-xs font-mono font-bold text-slate-500">Status Phase</div>
                    <div className="text-sm font-black font-mono text-slate-900">
                      {runningStep < 0
                        ? "Degraded (Incident Alert)"
                        : runningStep === currentScenario.steps.length - 1 && currentScenario.id !== "circuitbreaker"
                        ? "Running & Ready (1/1)"
                        : currentScenario.id === "circuitbreaker" && runningStep >= 3
                        ? "Locked: Requires Human Intervention"
                        : "Remediating Pod State..."}
                    </div>
                  </div>

                  <div className="pt-1">
                    <span
                      className={`inline-block text-xs font-mono font-bold px-3.5 py-1 rounded-full border ${
                        runningStep === currentScenario.steps.length - 1 && currentScenario.id !== "circuitbreaker"
                          ? "bg-emerald-100 text-emerald-900 border-emerald-300"
                          : currentScenario.id === "circuitbreaker" && runningStep >= 3
                          ? "bg-rose-100 text-rose-900 border-rose-300"
                          : "bg-amber-100 text-amber-900 border-amber-300"
                      }`}
                    >
                      {runningStep < 0
                        ? "Awaiting Triage Run"
                        : runningStep === currentScenario.steps.length - 1 && currentScenario.id !== "circuitbreaker"
                        ? "✓ RESOLVED (Recovered)"
                        : currentScenario.id === "circuitbreaker" && runningStep >= 3
                        ? "⚠️ CIRCUIT BREAKER LOCKED"
                        : "⚡ In-Flight Remediation"}
                    </span>
                  </div>
                </div>
              </div>

              {/* Circuit Breaker Gauges */}
              <div className="p-5 rounded-2xl bg-slate-50 border border-slate-200 space-y-3">
                <div className="text-xs font-mono font-bold text-slate-600 uppercase tracking-widest flex items-center justify-between">
                  <span>Operational Circuit Breakers</span>
                  <Shield className="w-4 h-4 text-emerald-600" />
                </div>

                <div className="space-y-2 text-xs font-mono font-semibold">
                  <div className="flex justify-between text-slate-800">
                    <span>Per-Resource Limit (Max 3/10m):</span>
                    <span className={currentScenario.id === "circuitbreaker" && runningStep >= 3 ? "text-rose-700 font-extrabold" : "text-emerald-700 font-extrabold"}>
                      {currentScenario.id === "circuitbreaker" && runningStep >= 3 ? "4 / 3 (EXCEEDED)" : "1 / 3 (OK)"}
                    </span>
                  </div>
                  <div className="w-full h-2.5 rounded-full bg-slate-200 overflow-hidden">
                    <div
                      className={`h-full transition-all duration-500 ${
                        currentScenario.id === "circuitbreaker" && runningStep >= 3 ? "w-full bg-rose-600" : "w-1/3 bg-emerald-600"
                      }`}
                    />
                  </div>

                  <div className="flex justify-between text-slate-800 pt-1">
                    <span>Incident Action Budget (Max 5/inc):</span>
                    <span className="text-emerald-700 font-extrabold">1 / 5 (OK)</span>
                  </div>
                  <div className="w-full h-2.5 rounded-full bg-slate-200 overflow-hidden">
                    <div className="h-full w-1/5 bg-emerald-600" />
                  </div>

                  <div className="flex justify-between text-slate-800 pt-1">
                    <span>Mandatory Cooldown (60s):</span>
                    <span className="text-emerald-700 font-extrabold">60s Passed (Ready)</span>
                  </div>
                </div>
              </div>

              {/* Human Gateway */}
              {currentScenario.id === "circuitbreaker" && runningStep >= 3 && (
                <div className="p-5 rounded-2xl bg-rose-50 border border-rose-300 space-y-3">
                  <div className="flex items-center gap-2 text-rose-900 font-extrabold text-xs">
                    <Lock className="w-4 h-4" />
                    <span>Human Approval Gateway (/dashboard)</span>
                  </div>
                  <p className="text-xs text-slate-700 font-medium">
                    Cheezer safely halted autonomous execution. SRE human override requested.
                  </p>
                  
                  {humanApproved ? (
                    <div className="p-3 rounded-xl bg-emerald-100 border border-emerald-300 text-emerald-900 text-xs font-mono font-bold flex items-center justify-center gap-2">
                      <UserCheck className="w-4 h-4 text-emerald-700" />
                      Human Approved → Re-evaluating OPA Gate → Executed!
                    </div>
                  ) : (
                    <button
                      onClick={handleHumanApprove}
                      className="w-full py-3 rounded-xl bg-rose-600 hover:bg-rose-500 text-white font-bold text-xs transition-all flex items-center justify-center gap-2 shadow-sm"
                    >
                      <UserCheck className="w-4 h-4" />
                      Approve & Force OPA Re-Evaluation
                    </button>
                  )}
                </div>
              )}

            </div>

          </div>

        </div>

      </div>
    </section>
  );
}
