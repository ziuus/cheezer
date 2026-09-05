"use client";

import React from "react";
import { Terminal } from "lucide-react";

function GithubIcon({ className = "w-4 h-4" }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
    </svg>
  );
}

export default function FooterSection() {
  return (
    <footer className="bg-slate-900 text-white pt-16 pb-12 relative overflow-hidden">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        
        <div className="grid grid-cols-1 md:grid-cols-12 gap-12 pb-12 border-b border-slate-800">
          
          {/* Brand Info */}
          <div className="md:col-span-5 space-y-4">
            <div className="flex items-center gap-3">
              <div className="w-9 h-9 rounded-xl bg-amber-500/20 border border-amber-500/30 flex items-center justify-center text-lg select-none">
                🧀
              </div>
              <span className="font-extrabold text-2xl tracking-tight text-white">
                Cheezer<span className="text-amber-400">.rs</span>
              </span>
            </div>

            <p className="text-sm text-slate-400 leading-relaxed max-w-md">
              Autonomous, Out-of-Band Kubernetes Incident-Remediation Engine built in Rust for the Cyber Resilience Track. Zero AI cost for known alerts, fail-closed OPA security gates.
            </p>

            {/* Cargo Run Snippet */}
            <div className="p-3 rounded-xl bg-slate-950 border border-slate-800 font-mono text-xs text-amber-300 inline-flex items-center gap-3">
              <span className="text-slate-500">$</span>
              <code>cargo run --release --manifest-path cheezer-core/Cargo.toml</code>
            </div>
          </div>

          {/* Quick Links */}
          <div className="md:col-span-3 space-y-3 font-mono text-xs">
            <div className="text-white font-bold uppercase tracking-wider mb-2">Navigation</div>
            <div>
              <a href="#simulator" className="text-slate-400 hover:text-amber-400 transition-colors">
                Live Simulator
              </a>
            </div>
            <div>
              <a href="#architecture" className="text-slate-400 hover:text-amber-400 transition-colors">
                8-Step Topology
              </a>
            </div>
            <div>
              <a href="#defense-grid" className="text-slate-400 hover:text-amber-400 transition-colors">
                Defense Matrix
              </a>
            </div>
            <div>
              <a href="#rust-code" className="text-slate-400 hover:text-amber-400 transition-colors">
                Rust Source Code
              </a>
            </div>
          </div>

          {/* Attribution & Legal */}
          <div className="md:col-span-4 space-y-3 text-xs text-slate-400">
            <div className="text-white font-bold uppercase tracking-wider font-mono mb-2">Architectural Attribution</div>
            <p className="leading-relaxed">
              Taxonomy and rule-first escalation patterns inspired by the public architecture of Apache 2.0 open-source projects:{" "}
              <a href="https://github.com/k8sgpt-ai/k8sgpt" target="_blank" rel="noreferrer" className="text-amber-400 hover:underline">
                K8sGPT
              </a>{" "}
              and{" "}
              <a href="https://github.com/robusta-dev/holmes" target="_blank" rel="noreferrer" className="text-amber-400 hover:underline">
                HolmesGPT
              </a>
              .
            </p>

            <div className="pt-2">
              <a
                href="https://github.com"
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-mono transition-all border border-slate-700"
              >
                <GithubIcon className="w-4 h-4 text-amber-400" />
                <span>GitHub Repository</span>
              </a>
            </div>
          </div>

        </div>

        {/* Bottom Credits */}
        <div className="pt-8 flex flex-wrap items-center justify-between gap-4 text-xs font-mono text-slate-500">
          <div>© 2026 Cheezer Project. Built in Rust for High-Resilience Kubernetes Automation.</div>
          <div className="flex items-center gap-2">
            <span>Fail-Closed OPA Gate</span>
            <span>•</span>
            <span>Zero AI Cost Triage</span>
          </div>
        </div>

      </div>
    </footer>
  );
}
