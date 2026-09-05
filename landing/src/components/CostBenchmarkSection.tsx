"use client";

import React, { useState } from "react";
import { motion } from "framer-motion";
import { DollarSign, TrendingUp, Sliders } from "lucide-react";

export default function CostBenchmarkSection() {
  const [alertVolume, setAlertVolume] = useState<number>(10000);

  const knownRatio = 0.88;
  const llmCostPerAlert = 0.04;
  const traditionalCost = alertVolume * llmCostPerAlert;
  const cheezerLlmCost = alertVolume * (1 - knownRatio) * llmCostPerAlert;
  const monthlySavings = traditionalCost - cheezerLlmCost;
  const yearlySavings = monthlySavings * 12;

  return (
    <section className="py-24 relative overflow-hidden bg-slate-50/60">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        
        {/* Section Header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="text-center max-w-3xl mx-auto mb-16 space-y-3"
        >
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-amber-500/10 border border-amber-500/20 text-amber-800 text-xs font-mono font-semibold uppercase">
            <DollarSign className="w-3.5 h-3.5 text-amber-600" />
            ROI & Cost Calculator
          </div>
          <h2 className="text-3xl sm:text-4xl font-extrabold text-slate-900 tracking-tight">
            Zero AI Cost for 88%+ of Cluster Alerts
          </h2>
          <p className="text-slate-600 text-sm sm:text-base leading-relaxed">
            Why send every CrashLoopBackOff to expensive LLMs? Cheezer's rule-first triage executes deterministic remediations in sub-milliseconds at zero cost.
          </p>
        </motion.div>

        {/* Calculator Card */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="apex-glass-card rounded-3xl border border-slate-900/10 p-8 max-w-5xl mx-auto shadow-xl space-y-8 bg-white"
        >
          
          {/* Slider */}
          <div className="space-y-4">
            <div className="flex flex-wrap items-center justify-between gap-4 font-mono text-sm">
              <span className="text-slate-700 font-bold flex items-center gap-2">
                <Sliders className="w-4 h-4 text-amber-600" />
                Monthly Cluster Alert Volume:
              </span>
              <span className="text-2xl font-extrabold text-amber-600 font-mono">{alertVolume.toLocaleString()} alerts / month</span>
            </div>

            <input
              type="range"
              min="1000"
              max="100000"
              step="1000"
              value={alertVolume}
              onChange={(e) => setAlertVolume(Number(e.target.value))}
              className="w-full h-2.5 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-amber-600"
            />

            <div className="flex justify-between text-xs font-mono text-slate-400">
              <span>1,000 alerts</span>
              <span>25,000 alerts</span>
              <span>50,000 alerts</span>
              <span>100,000 alerts</span>
            </div>
          </div>

          {/* Comparison Cards */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
            
            {/* Standard LLM Agent */}
            <div className="p-6 rounded-2xl bg-slate-50 border border-slate-200 space-y-4">
              <div className="text-xs font-mono text-slate-500 uppercase tracking-wide">Traditional LLM Incident Agent</div>
              <div className="text-3xl font-extrabold text-rose-600 font-mono">
                ${traditionalCost.toLocaleString(undefined, { maximumFractionDigits: 0 })} <span className="text-xs text-slate-500 font-normal">/ mo</span>
              </div>
              
              <div className="space-y-2 text-xs font-mono text-slate-600 pt-2 border-t border-slate-200">
                <div className="flex justify-between">
                  <span>AI Cost per Alert:</span>
                  <span className="text-slate-900">$0.04 / call</span>
                </div>
                <div className="flex justify-between">
                  <span>Triage Latency:</span>
                  <span className="text-rose-600 font-bold">3,500ms - 8,000ms</span>
                </div>
                <div className="flex justify-between">
                  <span>Shell Safety:</span>
                  <span className="text-rose-600">Unbounded Command Execution</span>
                </div>
              </div>
            </div>

            {/* Cheezer Rust Engine */}
            <div className="p-6 rounded-2xl bg-amber-50 border border-amber-200 space-y-4 shadow-sm">
              <div className="text-xs font-mono text-amber-800 uppercase tracking-wide flex justify-between">
                <span>Cheezer Rust Triage Engine</span>
                <span className="text-emerald-700 font-bold">88% Zero AI Cost</span>
              </div>
              <div className="text-3xl font-extrabold text-emerald-700 font-mono">
                ${cheezerLlmCost.toLocaleString(undefined, { maximumFractionDigits: 0 })} <span className="text-xs text-slate-500 font-normal">/ mo</span>
              </div>

              <div className="space-y-2 text-xs font-mono text-slate-700 pt-2 border-t border-amber-200">
                <div className="flex justify-between">
                  <span>Rule Match Latency:</span>
                  <span className="text-emerald-700 font-bold">0.12ms (Zero AI Cost)</span>
                </div>
                <div className="flex justify-between">
                  <span>Fail-Closed OPA:</span>
                  <span className="text-emerald-700">Enforced on 100% of Mutations</span>
                </div>
                <div className="flex justify-between">
                  <span>Structured Intent:</span>
                  <span className="text-emerald-700">Rust Action Enum (0 Shell Access)</span>
                </div>
              </div>
            </div>

          </div>

          {/* Yearly Savings Banner */}
          <motion.div
            layout
            className="p-6 rounded-2xl bg-emerald-50 border border-emerald-200 flex flex-wrap items-center justify-between gap-4"
          >
            <div className="space-y-1">
              <div className="text-xs font-mono uppercase text-emerald-800 font-bold flex items-center gap-1.5">
                <TrendingUp className="w-4 h-4 text-emerald-600" />
                Estimated Annual Infrastructure Savings
              </div>
              <div className="text-xs text-slate-600">Based on 88% rule-first triage efficiency and zero hallucination retry costs.</div>
            </div>

            <div className="text-3xl sm:text-4xl font-extrabold text-emerald-700 font-mono">
              ${yearlySavings.toLocaleString(undefined, { maximumFractionDigits: 0 })} <span className="text-sm text-slate-600 font-normal">/ year</span>
            </div>
          </motion.div>

        </motion.div>

      </div>
    </section>
  );
}
