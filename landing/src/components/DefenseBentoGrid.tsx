"use client";

import React from "react";
import { Shield, Lock, AlertTriangle, RefreshCw, Cpu, Eye } from "lucide-react";

export default function DefenseBentoGrid() {
  return (
    <section id="defense-grid" className="py-24 relative overflow-hidden bg-white border-t border-b border-slate-200">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16 space-y-3">
          <div className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full bg-emerald-100 border border-emerald-300 text-emerald-900 text-xs font-mono font-bold uppercase">
            <Shield className="w-4 h-4 text-emerald-700" />
            Security & Resilience Matrix
          </div>
          <h2 className="text-3xl sm:text-4xl font-extrabold text-slate-900 tracking-tight">
            Defense-in-Depth Engineering
          </h2>
          <p className="text-slate-700 text-base leading-relaxed">
            Cheezer is engineered with fail-closed default posture, strict rate-limiting circuit breakers, and zero raw shell execution authority.
          </p>
        </div>

        {/* Asymmetric Bento Grid */}
        <div className="grid grid-cols-1 md:grid-cols-12 gap-6">
          
          {/* Card 1: Fail-Closed OPA Policy Engine (Large Feature Card - 8 cols) */}
          <div className="md:col-span-8 clean-card clean-card-hover p-8 sm:p-10 rounded-3xl relative overflow-hidden group border-slate-300">
            <div className="absolute top-0 right-0 p-8 text-emerald-500/5 group-hover:text-emerald-500/10 transition-colors pointer-events-none">
              <Shield className="w-56 h-56 -mr-12 -mt-12" />
            </div>

            <div className="relative z-10 space-y-5">
              <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-100 border border-emerald-300 text-emerald-900 text-xs font-mono font-bold">
                <Lock className="w-3.5 h-3.5 text-emerald-700" />
                Fail-Closed Constraint
              </div>

              <h3 className="text-2xl sm:text-3xl font-extrabold text-slate-900">
                Fail-Closed Open Policy Agent (OPA) Gate
              </h3>

              <p className="text-slate-700 text-base leading-relaxed max-w-xl font-normal">
                Every mutation—whether from zero-cost rules, LLM structured outputs, or human overrides—must pass HTTP Rego validation against OPA. If the OPA daemon is unreachable, times out (500ms), or returns non-200, Cheezer strictly defaults to <strong className="text-emerald-800 font-extrabold">FAIL-CLOSED (DENY)</strong>.
              </p>

              {/* Rego Code Box */}
              <div className="p-4 rounded-2xl bg-[#0d1117] font-mono text-xs text-amber-200 max-w-lg space-y-1 shadow-md border border-slate-800 font-semibold">
                <div className="text-[10px] text-slate-400 border-b border-slate-800 pb-1 flex justify-between font-bold">
                  <span>policies/cheezer.rego</span>
                  <span className="text-emerald-400">Strict DENY Default</span>
                </div>
                <div className="text-slate-400">package cheezer.authz</div>
                <div>default allow = <span className="text-rose-400 font-bold">false</span> # Fail-Closed</div>
                <div className="text-emerald-400">allow {"{"} input.action == "RestartPod"; input.namespace == "default" {"}"}</div>
              </div>
            </div>
          </div>

          {/* Card 2: 3-Tier Operational Circuit Breakers (4 cols) */}
          <div className="md:col-span-4 clean-card clean-card-hover p-8 rounded-3xl relative overflow-hidden flex flex-col justify-between border-slate-300">
            <div className="space-y-4">
              <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-amber-100 border border-amber-300 text-amber-900 text-xs font-mono font-bold">
                <AlertTriangle className="w-3.5 h-3.5 text-amber-700" />
                Circuit Breakers
              </div>

              <h3 className="text-xl font-extrabold text-slate-900">
                3-Tier Operational Circuit Breakers
              </h3>

              <p className="text-xs text-slate-700 leading-relaxed font-normal">
                Stops flap storms and endless remediation loops before they can mutate production workloads.
              </p>
            </div>

            <div className="space-y-2.5 pt-6 font-mono text-xs font-semibold">
              <div className="p-3 rounded-xl bg-slate-100 border border-slate-200 flex justify-between">
                <span className="text-slate-700">Per-Resource Limit:</span>
                <span className="text-amber-900 font-bold">Max 3 / 10m</span>
              </div>
              <div className="p-3 rounded-xl bg-slate-100 border border-slate-200 flex justify-between">
                <span className="text-slate-700">Incident Action Budget:</span>
                <span className="text-amber-900 font-bold">Max 5 total</span>
              </div>
              <div className="p-3 rounded-xl bg-slate-100 border border-slate-200 flex justify-between">
                <span className="text-slate-700">Mandatory Cooldown:</span>
                <span className="text-amber-900 font-bold">60s window</span>
              </div>
            </div>
          </div>

          {/* Card 3: TOCTOU Protection (4 cols) */}
          <div className="md:col-span-4 clean-card clean-card-hover p-8 rounded-3xl space-y-4 border-slate-300">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-blue-100 border border-blue-300 text-blue-900 text-xs font-mono font-bold">
              <RefreshCw className="w-3.5 h-3.5 text-blue-700" />
              TOCTOU Protection
            </div>

            <h3 className="text-xl font-extrabold text-slate-900">
              Time-of-Check State Revalidation
            </h3>

            <p className="text-xs text-slate-700 leading-relaxed font-normal">
              Prevents TOCTOU race conditions. Before touching OPA or executing mutations, <code className="text-amber-900 bg-amber-100 px-1.5 py-0.5 rounded font-mono font-bold">revalidate_state</code> queries Kubernetes via kube-rs. If a pod self-resolved, execution aborts cleanly.
            </p>
          </div>

          {/* Card 4: HA Watchdog (4 cols) */}
          <div className="md:col-span-4 clean-card clean-card-hover p-8 rounded-3xl space-y-4 border-slate-300">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-purple-100 border border-purple-300 text-purple-900 text-xs font-mono font-bold">
              <Cpu className="w-3.5 h-3.5 text-purple-700" />
              HA Watchdog
            </div>

            <h3 className="text-xl font-extrabold text-slate-900">
              Active-Passive Leader Election
            </h3>

            <p className="text-xs text-slate-700 leading-relaxed font-normal">
              Runs out-of-band with active-passive TCP heartbeats between Primary and Backup daemons. If Primary dies, Backup seamlessly inherits webhook ingestion without state loss.
            </p>
          </div>

          {/* Card 5: Human Gateway (4 cols) */}
          <div className="md:col-span-4 clean-card clean-card-hover p-8 rounded-3xl space-y-4 border-slate-300">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-amber-100 border border-amber-300 text-amber-900 text-xs font-mono font-bold">
              <Eye className="w-3.5 h-3.5 text-amber-700" />
              Human Gateway
            </div>

            <h3 className="text-xl font-extrabold text-slate-900">
              Web Dashboard & Override Gateway
            </h3>

            <p className="text-xs text-slate-700 leading-relaxed font-normal">
              Embedded Axum Web UI mounted at <code className="text-amber-900 bg-amber-100 px-1.5 py-0.5 rounded font-mono font-bold">/dashboard</code>. Provides live incident polling and 1-click human approval for locked circuit breaker incidents.
            </p>
          </div>

        </div>

      </div>
    </section>
  );
}
