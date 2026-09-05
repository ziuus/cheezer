"use client";

import React, { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Shield, Zap, Terminal, Cpu, ChevronRight, Menu, X } from "lucide-react";

function GithubIcon({ className = "w-4 h-4" }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
    </svg>
  );
}

export default function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 20);
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const scrollTo = (id: string) => {
    setMobileOpen(false);
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: "smooth" });
    }
  };

  return (
    <header
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled
          ? "bg-white/90 backdrop-blur-md border-b border-slate-200 py-3.5 shadow-sm"
          : "bg-transparent py-5"
      }`}
    >
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between">
          
          {/* Brand Logo */}
          <div
            className="flex items-center gap-3 cursor-pointer"
            onClick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
          >
            <div className="w-10 h-10 rounded-xl bg-amber-500 flex items-center justify-center text-xl shadow-sm text-white select-none">
              🧀
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-extrabold text-xl tracking-tight text-slate-900">
                  Cheezer<span className="text-amber-600">.rs</span>
                </span>
                <span className="text-[10px] font-mono font-bold tracking-wider uppercase px-2 py-0.5 rounded bg-slate-100 text-slate-700 border border-slate-200 hidden sm:inline-block">
                  Rust Core
                </span>
              </div>
            </div>
          </div>

          {/* Desktop Navigation */}
          <nav className="hidden lg:flex items-center gap-8 text-xs font-bold uppercase tracking-wider text-slate-700">
            <button
              onClick={() => scrollTo("simulator")}
              className="hover:text-amber-600 transition-colors flex items-center gap-1.5"
            >
              <Zap className="w-3.5 h-3.5 text-amber-600" />
              Simulator
            </button>
            <button
              onClick={() => scrollTo("architecture")}
              className="hover:text-amber-600 transition-colors flex items-center gap-1.5"
            >
              <Cpu className="w-3.5 h-3.5 text-slate-500" />
              Architecture
            </button>
            <button
              onClick={() => scrollTo("defense-grid")}
              className="hover:text-amber-600 transition-colors flex items-center gap-1.5"
            >
              <Shield className="w-3.5 h-3.5 text-emerald-600" />
              Defense Matrix
            </button>
            <button
              onClick={() => scrollTo("rust-code")}
              className="hover:text-amber-600 transition-colors flex items-center gap-1.5"
            >
              <Terminal className="w-3.5 h-3.5 text-slate-500" />
              Rust Source
            </button>
          </nav>

          {/* Action CTAs */}
          <div className="hidden sm:flex items-center gap-3">
            <a
              href="https://github.com"
              target="_blank"
              rel="noreferrer"
              className="p-2.5 rounded-xl bg-slate-100 hover:bg-slate-200 text-slate-800 transition-all border border-slate-200"
              title="GitHub Repo"
            >
              <GithubIcon className="w-4 h-4" />
            </a>

            <button
              onClick={() => scrollTo("simulator")}
              className="px-5 py-2.5 rounded-xl bg-slate-900 hover:bg-slate-800 text-white font-bold text-xs transition-all flex items-center gap-2 shadow-sm"
            >
              <span>Test Simulator</span>
              <ChevronRight className="w-3.5 h-3.5 text-amber-400" />
            </button>
          </div>

          {/* Mobile Drawer Toggle */}
          <button
            onClick={() => setMobileOpen(!mobileOpen)}
            className="lg:hidden p-2 rounded-xl bg-slate-100 text-slate-800"
          >
            {mobileOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </button>
        </div>
      </div>

      {/* Mobile Drawer */}
      <AnimatePresence>
        {mobileOpen && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            className="lg:hidden border-b border-slate-200 bg-white px-6 py-6 space-y-3 font-bold text-xs uppercase tracking-wider text-slate-800"
          >
            <button
              onClick={() => scrollTo("simulator")}
              className="w-full text-left py-2 flex items-center gap-2"
            >
              <Zap className="w-4 h-4 text-amber-600" />
              Simulator
            </button>
            <button
              onClick={() => scrollTo("architecture")}
              className="w-full text-left py-2 flex items-center gap-2"
            >
              <Cpu className="w-4 h-4 text-slate-500" />
              Architecture
            </button>
            <button
              onClick={() => scrollTo("defense-grid")}
              className="w-full text-left py-2 flex items-center gap-2"
            >
              <Shield className="w-4 h-4 text-emerald-600" />
              Defense Matrix
            </button>
            <button
              onClick={() => scrollTo("rust-code")}
              className="w-full text-left py-2 flex items-center gap-2"
            >
              <Terminal className="w-4 h-4 text-slate-500" />
              Rust Source
            </button>
            <button
              onClick={() => scrollTo("simulator")}
              className="w-full mt-4 py-3 rounded-xl bg-slate-900 text-white font-bold text-center block"
            >
              Launch Simulator
            </button>
          </motion.div>
        )}
      </AnimatePresence>
    </header>
  );
}
