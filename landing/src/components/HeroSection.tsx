"use client";

import React, { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { Shield, Zap, Terminal, Play, CheckCircle2, RefreshCw, Cpu, Sparkles } from "lucide-react";

export default function HeroSection() {
  const [activeStep, setActiveStep] = useState(0);
  const [isSimulating, setIsSimulating] = useState(true);

  const simulationSteps = [
    { label: "1. Webhook Ingested", text: "POST /api/grafana_webhook -> Alert: CrashLoopBackOff (pod: auth-service-7f9a, ns: default)", time: "0.01ms" },
    { label: "2. Rule Triage", text: "triage.rs -> Pattern MATCH: CrashLoopBackOff -> Action::RestartPod (Zero AI Cost)", time: "0.12ms" },
    { label: "3. TOCTOU Validation", text: "executor.rs -> Querying kube-rs -> Pod state: CrashLoopBackOff (State Valid)", time: "12.45ms" },
    { label: "4. Remediation Guard", text: "guard.rs -> Per-resource count: 1/3 (Within 10m budget, Cooldown OK)", time: "13.02ms" },
    { label: "5. Fail-Closed OPA Gate", text: "policy.rs -> POST http://opa:8181/v1/data/cheezer/authz/allow -> Result: ALLOWED", time: "18.89ms" },
    { label: "6. Kube-rs Executor", text: "executor.rs -> Api::<Pod>::delete('auth-service-7f9a') -> Executed successfully", time: "45.10ms" },
    { label: "7. Recovery Verified", text: "verify_recovery -> Pod auth-service-7f9a -> Running & Ready (1/1)", time: "120.00ms" }
  ];

  useEffect(() => {
    if (!isSimulating) return;
    const interval = setInterval(() => {
      setActiveStep((prev) => (prev + 1) % simulationSteps.length);
    }, 2200);
    return () => clearInterval(interval);
  }, [isSimulating, simulationSteps.length]);

  return (
    <section className="relative pt-32 pb-20 md:pt-40 md:pb-28 overflow-hidden bg-clean-grid">
      
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-center">
          
          {/* Left Column: Clear High-Contrast Text */}
          <div className="lg:col-span-7 space-y-7 text-left">
            
            {/* Pill Badge */}
            <div className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full bg-amber-500/10 border border-amber-500/20 text-amber-900 text-xs font-mono font-bold">
              <Sparkles className="w-4 h-4 text-amber-600 animate-pulse" />
              <span>Autonomous K8s Remediation Engine in Rust</span>
            </div>

            {/* Clear Title */}
            <h1 className="text-4xl sm:text-5xl lg:text-6xl font-black tracking-tight text-slate-900 leading-[1.1]">
              Autonomous Incident Remediation.{" "}
              <span className="text-amber-600">Zero AI Cost</span> for Known Alerts.
            </h1>

            {/* Subheading */}
            <p className="text-base sm:text-lg text-slate-700 leading-relaxed max-w-2xl font-normal">
              Cheezer triages cluster alerts deterministically, enforces <strong className="text-slate-900 font-bold">Fail-Closed OPA Rego security gates</strong>, and revalidates state via <code className="text-amber-900 bg-amber-100 px-2 py-0.5 rounded font-mono text-sm font-bold">kube-rs</code> before touching production workloads.
            </p>

            {/* CTAs */}
            <div className="flex flex-wrap items-center gap-4 pt-2">
              <a
                href="#simulator"
                className="px-7 py-3.5 rounded-xl bg-slate-900 hover:bg-slate-800 text-white font-bold text-sm shadow-md transition-all flex items-center gap-2"
              >
                <Zap className="w-4 h-4 text-amber-400" />
                Launch Live Simulator
              </a>

              <a
                href="#architecture"
                className="px-7 py-3.5 rounded-xl bg-white hover:bg-slate-50 border border-slate-300 text-slate-800 font-bold text-sm shadow-sm transition-all flex items-center gap-2"
              >
                <Cpu className="w-4 h-4 text-slate-600" />
                Explore Architecture
              </a>
            </div>

            {/* High Contrast Metrics */}
            <div className="pt-8 border-t border-slate-200 grid grid-cols-4 gap-4">
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-black font-mono text-slate-900">0ms</div>
                <div className="text-xs text-slate-600 font-bold uppercase tracking-wider">Rule AI Latency</div>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-black font-mono text-emerald-700">100%</div>
                <div className="text-xs text-slate-600 font-bold uppercase tracking-wider">Fail-Closed OPA</div>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-black font-mono text-amber-700">3x</div>
                <div className="text-xs text-slate-600 font-bold uppercase tracking-wider">Circuit Breakers</div>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-black font-mono text-slate-900">17/17</div>
                <div className="text-xs text-slate-600 font-bold uppercase tracking-wider">Rust Tests</div>
              </div>
            </div>

          </div>

          {/* Right Column: Ultra Crisp Terminal */}
          <div className="lg:col-span-5">
            <div className="rounded-2xl overflow-hidden border border-slate-800 bg-[#0d1117] shadow-2xl">
              
              {/* Header Bar */}
              <div className="bg-[#161b22] px-5 py-3 border-b border-slate-800 flex items-center justify-between text-white">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full bg-rose-500" />
                  <div className="w-3 h-3 rounded-full bg-amber-500" />
                  <div className="w-3 h-3 rounded-full bg-emerald-500" />
                  <span className="ml-2 text-xs font-mono text-slate-300 font-bold flex items-center gap-1.5">
                    <Terminal className="w-3.5 h-3.5 text-amber-400" />
                    cheezer-core --live-triage
                  </span>
                </div>

                <button
                  onClick={() => setIsSimulating(!isSimulating)}
                  className="text-[11px] font-mono text-slate-300 hover:text-white flex items-center gap-1 px-2.5 py-1 rounded bg-[#21262d]"
                >
                  {isSimulating ? <RefreshCw className="w-3 h-3 animate-spin text-amber-400" /> : <Play className="w-3 h-3 text-emerald-400" />}
                  {isSimulating ? "Live" : "Paused"}
                </button>
              </div>

              {/* Terminal Stream */}
              <div className="p-5 font-mono text-xs space-y-3 bg-[#0d1117] text-slate-200 min-h-[380px]">
                <div className="text-slate-400 pb-2 border-b border-slate-800 flex justify-between items-center text-[11px]">
                  <span>[Triage Execution Stream]</span>
                  <span className="text-emerald-400 font-bold flex items-center gap-1.5">
                    <span className="w-2 h-2 rounded-full bg-emerald-400 animate-ping" />
                    ENGINE ACTIVE
                  </span>
                </div>

                {simulationSteps.map((step, idx) => {
                  const isActive = idx === activeStep;
                  const isDone = idx < activeStep || (activeStep === simulationSteps.length - 1 && idx <= activeStep);

                  return (
                    <div
                      key={idx}
                      className={`p-3 rounded-xl border transition-all duration-200 ${
                        isActive
                          ? "bg-amber-500/10 border-amber-500/50 text-amber-300 shadow-md"
                          : isDone
                          ? "bg-[#161b22] border-slate-800 text-slate-200"
                          : "opacity-40 border-transparent text-slate-500"
                      }`}
                    >
                      <div className="flex items-center justify-between mb-1 text-[11px]">
                        <div className="flex items-center gap-2 font-bold">
                          {isDone ? (
                            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                          ) : isActive ? (
                            <RefreshCw className="w-4 h-4 text-amber-400 animate-spin" />
                          ) : (
                            <div className="w-3.5 h-3.5 rounded-full border border-slate-700" />
                          )}
                          <span className={isActive ? "text-amber-300" : isDone ? "text-slate-100" : "text-slate-500"}>
                            {step.label}
                          </span>
                        </div>
                        <span className="text-[10px] text-slate-400 font-mono">{step.time}</span>
                      </div>
                      <div className="text-[11px] leading-relaxed pl-6 text-slate-300 break-all font-mono">
                        {step.text}
                      </div>
                    </div>
                  );
                })}
              </div>

              {/* Footer */}
              <div className="bg-[#161b22] px-5 py-3 border-t border-slate-800 text-[11px] font-mono text-slate-300 flex items-center justify-between font-bold">
                <span className="flex items-center gap-2">
                  <span className="w-2 h-2 rounded-full bg-emerald-400" />
                  Cluster Health: 100% (OPA Authorized)
                </span>
                <span className="text-amber-400">$0 AI Spent</span>
              </div>

            </div>
          </div>

        </div>
      </div>
    </section>
  );
}
