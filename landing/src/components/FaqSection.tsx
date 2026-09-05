"use client";

import React, { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { HelpCircle, ChevronDown } from "lucide-react";

interface FaqItem {
  question: string;
  answer: string;
  tag: string;
}

export default function FaqSection() {
  const [openIndex, setOpenIndex] = useState<number | null>(0);

  const faqs: FaqItem[] = [
    {
      question: "What happens if the Open Policy Agent (OPA) daemon crashes or times out?",
      answer: "Cheezer operates on a strict Fail-Closed security invariant (policy.rs). If the OPA daemon is unreachable, times out after 500ms, returns an HTTP 500, or returns a response missing 'result: true', Cheezer strictly defaults to DENY. No mutations are performed without explicit Rego policy approval.",
      tag: "Security Invariant",
    },
    {
      question: "Can the LLM execute raw bash commands like `rm -rf` or mutate arbitrary cluster resources?",
      answer: "No. The LLM has ZERO raw shell or command-line execution authority. System prompts enforce structured JSON output that deserializes directly into a strongly-typed Rust Action enum (RestartPod, ScaleDeployment, CordonNode). Any hallucinated bash, invalid JSON, or unallowed action is immediately rejected and routes to Local Fallback Mode.",
      tag: "Zero-Trust Design",
    },
    {
      question: "How does Cheezer prevent flap storms if a pod continuously crashes?",
      answer: "Cheezer enforces 3 operational circuit breakers in guard.rs: 1) Per-Resource Limit (Max 3 actions on the same workload within 10 minutes), 2) Incident Action Budget (Max 5 total actions), and 3) Cooldown (60s mandatory wait). Exceeding these limits locks autonomous execution, changes incident status to 'requires_human_intervention', and triggers an outbound Slack webhook.",
      tag: "Circuit Breakers",
    },
    {
      question: "Why is Cheezer engineered out-of-band rather than inside the Kubernetes cluster?",
      answer: "Running out-of-band guarantees operational continuity. If an in-cluster CNI breaks, API server degrades, or node memory starves in-cluster pods, an in-cluster operator would freeze. Running out-of-band allows Cheezer to remediate cluster issues even when internal pods cannot schedule.",
      tag: "Out-of-Band Architecture",
    },
    {
      question: "How does Time-of-Check to Time-of-Use (TOCTOU) revalidation work?",
      answer: "Immediately before executing any mutation or querying OPA, executor.rs queries Kubernetes via kube-rs. If a pod has already self-resolved (Phase: Running & Ready: true), Cheezer aborts execution cleanly as 'Aborted_StaleState' to prevent unnecessary restarts or duplicate actions.",
      tag: "TOCTOU Safety",
    },
  ];

  return (
    <section className="py-24 relative overflow-hidden bg-slate-50/50">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        
        {/* Section Header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="text-center max-w-3xl mx-auto mb-16 space-y-3"
        >
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-amber-500/10 border border-amber-500/20 text-amber-800 text-xs font-mono font-semibold uppercase">
            <HelpCircle className="w-3.5 h-3.5 text-amber-600" />
            SRE & Security FAQ
          </div>
          <h2 className="text-3xl sm:text-4xl font-extrabold text-slate-900 tracking-tight">
            Frequently Asked Questions
          </h2>
          <p className="text-slate-600 text-sm sm:text-base leading-relaxed">
            Everything you need to know about Cheezer's security guarantees, fail-closed boundaries, and architecture.
          </p>
        </motion.div>

        {/* FAQ Accordion List */}
        <div className="space-y-4">
          {faqs.map((faq, idx) => {
            const isOpen = openIndex === idx;
            return (
              <motion.div
                key={idx}
                initial={{ opacity: 0, y: 15 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.3, delay: idx * 0.05 }}
                className={`rounded-2xl transition-all duration-200 apex-glass-card ${
                  isOpen ? "border-amber-500/40 bg-amber-50/30 shadow-md" : "border-slate-900/10 hover:border-slate-900/20"
                }`}
              >
                <button
                  onClick={() => setOpenIndex(isOpen ? null : idx)}
                  className="w-full p-6 text-left flex items-center justify-between gap-4 cursor-pointer"
                >
                  <div className="space-y-1">
                    <span className="text-[10px] font-mono font-bold text-amber-800 uppercase tracking-wide">
                      {faq.tag}
                    </span>
                    <h3 className="text-base font-bold text-slate-900">{faq.question}</h3>
                  </div>

                  <motion.div
                    animate={{ rotate: isOpen ? 180 : 0 }}
                    transition={{ duration: 0.2 }}
                    className="p-2 rounded-xl bg-slate-100 text-slate-700 shrink-0"
                  >
                    <ChevronDown className="w-4 h-4" />
                  </motion.div>
                </button>

                <AnimatePresence>
                  {isOpen && (
                    <motion.div
                      initial={{ opacity: 0, height: 0 }}
                      animate={{ opacity: 1, height: "auto" }}
                      exit={{ opacity: 0, height: 0 }}
                      transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
                      className="overflow-hidden"
                    >
                      <div className="px-6 pb-6 pt-1 text-sm text-slate-600 leading-relaxed border-t border-slate-900/5 font-normal">
                        {faq.answer}
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </motion.div>
            );
          })}
        </div>

      </div>
    </section>
  );
}
