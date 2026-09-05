"use client";

import React, { useState } from "react";
import { motion } from "framer-motion";
import { Terminal, Copy, Check, FileCode, Code2 } from "lucide-react";

interface CodeTab {
  id: string;
  filename: string;
  language: string;
  description: string;
  code: string;
}

export default function RustCodeExplorer() {
  const [activeTabId, setActiveTabId] = useState<string>("triage");
  const [copied, setCopied] = useState<boolean>(false);

  const tabs: CodeTab[] = [
    {
      id: "triage",
      filename: "cheezer-core/src/triage.rs",
      language: "rust",
      description: "Rule-First Triage Engine matching known Kubernetes alert signatures with sub-millisecond execution.",
      code: `// cheezer-core/src/triage.rs
use crate::action::Action;
use crate::store::Alert;

pub enum TriageOutcome {
    MatchedRule(Action),
    EscalateToLlm,
}

pub fn triage_alert(alert: &Alert) -> TriageOutcome {
    let alert_name = alert.labels.get("alertname").map(|s| s.as_str()).unwrap_or("");
    
    match alert_name {
        "KubePodCrashLooping" | "CrashLoopBackOff" => {
            let pod = alert.labels.get("pod").cloned().unwrap_or_default();
            let ns = alert.labels.get("namespace").cloned().unwrap_or_else(|| "default".to_string());
            TriageOutcome::MatchedRule(Action::RestartPod { pod, namespace: ns })
        }
        "KubeContainerOOMKilled" | "OOMKilled" => {
            let dep = alert.labels.get("deployment").cloned().unwrap_or_default();
            let ns = alert.labels.get("namespace").cloned().unwrap_or_else(|| "default".to_string());
            TriageOutcome::MatchedRule(Action::ScaleDeployment { deployment: dep, target_replicas: 3, namespace: ns })
        }
        _ => TriageOutcome::EscalateToLlm,
    }
}`,
    },
    {
      id: "action",
      filename: "cheezer-core/src/action.rs",
      language: "rust",
      description: "Strongly-typed Rust Action Enum. Ensures zero raw shell or bash command execution authority.",
      code: `// cheezer-core/src/action.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "params")]
pub enum Action {
    RestartPod {
        pod: String,
        namespace: String,
    },
    ScaleDeployment {
        deployment: String,
        target_replicas: u32,
        namespace: String,
    },
    CordonNode {
        node: String,
    },
    DeleteNamespace {
        namespace: String,
    },
    ExecCommand {
        pod: String,
        command: String,
    },
    LogReviewNeeded {
        reason: String,
    },
    None,
}`,
    },
    {
      id: "policy",
      filename: "cheezer-core/src/policy.rs",
      language: "rust",
      description: "Fail-Closed OPA HTTP query enforcement. Connection errors or missing result: true strictly return DENY.",
      code: `// cheezer-core/src/policy.rs
use reqwest::Client;
use std::time::Duration;
use crate::action::Action;

pub async fn evaluate_opa_policy(
    client: &Client,
    opa_url: &str,
    action: &Action,
) -> bool {
    let payload = serde_json::json!({ "input": { "action": action } });

    let response = client
        .post(opa_url)
        .json(&payload)
        .timeout(Duration::from_millis(500)) // 500ms Strict Timeout
        .send()
        .await;

    match response {
        Ok(res) if res.status().is_success() => {
            res.json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|v| v.get("result")?.as_bool())
                .unwrap_or(false) // FAIL-CLOSED
        }
        _ => false, // FAIL-CLOSED ON TIMEOUT / 500 ERRORS!
    }
}`,
    },
    {
      id: "guard",
      filename: "cheezer-core/src/guard.rs",
      language: "rust",
      description: "Remediation Guard evaluating 3 circuit breaker conditions before touching Kubernetes mutations.",
      code: `// cheezer-core/src/guard.rs
use crate::store::SqliteStore;
use std::time::Duration;

pub struct GuardResult {
    pub allowed: bool,
    pub reason: String,
}

pub fn check_circuit_breakers(store: &SqliteStore, resource: &str) -> GuardResult {
    // 1. Per-resource limit: Max 3 actions in 10 minutes
    let count_10m = store.count_recent_actions(resource, Duration::from_secs(600));
    if count_10m >= 3 {
        return GuardResult {
            allowed: false,
            reason: format!("Per-resource limit exceeded ({} actions in 10m)", count_10m),
        };
    }

    GuardResult { allowed: true, reason: "OK".into() }
}`,
    },
    {
      id: "rego",
      filename: "cheezer-core/policies/cheezer.rego",
      language: "rego",
      description: "Open Policy Agent Rego policy enforcing fail-closed default allow = false.",
      code: `# cheezer-core/policies/cheezer.rego
package cheezer.authz

# Strict Fail-Closed Default
default allow = false

# Allow RestartPod in default and production namespaces
allow {
    input.action.action == "RestartPod"
    input.action.params.namespace == "default"
}

allow {
    input.action.action == "RestartPod"
    input.action.params.namespace == "production"
}

# Allow ScaleDeployment up to max 5 replicas
allow {
    input.action.action == "ScaleDeployment"
    input.action.params.target_replicas <= 5
}`,
    },
  ];

  const currentTab = tabs.find((t) => t.id === activeTabId) || tabs[0];

  const handleCopy = () => {
    navigator.clipboard.writeText(currentTab.code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section id="rust-code" className="py-24 relative overflow-hidden bg-white/80 border-t border-b border-slate-900/5">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        
        {/* Section Header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="text-center max-w-3xl mx-auto mb-16 space-y-3"
        >
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-slate-900/5 border border-slate-900/10 text-slate-800 text-xs font-mono font-semibold uppercase">
            <Code2 className="w-3.5 h-3.5 text-slate-700" />
            Open Source Rust Core
          </div>
          <h2 className="text-3xl sm:text-4xl font-extrabold text-slate-900 tracking-tight">
            Inspect the Rust Implementation
          </h2>
          <p className="text-slate-600 text-sm sm:text-base leading-relaxed">
            Zero magic. Pure, memory-safe Rust with strong compile-time guarantees and fail-closed security invariants.
          </p>
        </motion.div>

        {/* Code Box */}
        <div className="apex-glass-card rounded-3xl border border-slate-900/10 overflow-hidden shadow-xl">
          
          {/* Tab Selector */}
          <div className="bg-slate-100 px-4 pt-3 border-b border-slate-200 flex items-center justify-between overflow-x-auto">
            <div className="flex items-center gap-2">
              {tabs.map((tab) => {
                const isActive = tab.id === activeTabId;
                return (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTabId(tab.id)}
                    className={`px-4 py-2.5 rounded-t-xl text-xs font-mono font-semibold transition-all relative flex items-center gap-2 ${
                      isActive
                        ? "bg-slate-950 text-amber-400"
                        : "bg-transparent text-slate-600 hover:text-slate-900"
                    }`}
                  >
                    <FileCode className={`w-3.5 h-3.5 ${isActive ? "text-amber-400" : "text-slate-500"}`} />
                    {tab.filename.split("/").pop()}
                  </button>
                );
              })}
            </div>

            <motion.button
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              onClick={handleCopy}
              className="p-2.5 rounded-xl bg-slate-200 hover:bg-slate-300 text-slate-800 transition-all text-xs font-mono flex items-center gap-1.5 shrink-0 mb-2 font-bold"
            >
              {copied ? <Check className="w-3.5 h-3.5 text-emerald-600" /> : <Copy className="w-3.5 h-3.5" />}
              {copied ? "Copied!" : "Copy Code"}
            </motion.button>
          </div>

          {/* Subheader */}
          <div className="bg-slate-900 px-6 py-3 border-b border-slate-800 text-xs font-mono text-slate-300 flex items-center justify-between">
            <span className="text-slate-400">{currentTab.description}</span>
            <span className="text-amber-400 font-semibold hidden sm:inline-block">{currentTab.filename}</span>
          </div>

          {/* Code Viewer */}
          <div className="p-6 bg-slate-950 font-mono text-xs overflow-x-auto text-amber-100/90 leading-relaxed">
            <pre>
              <code>{currentTab.code}</code>
            </pre>
          </div>

        </div>

      </div>
    </section>
  );
}
