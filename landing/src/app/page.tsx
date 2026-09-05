"use client";

import React from "react";
import BackgroundCanvas from "@/components/BackgroundCanvas";
import Navbar from "@/components/Navbar";
import HeroSection from "@/components/HeroSection";
import TriageSimulator from "@/components/TriageSimulator";
import ArchitecturePipeline from "@/components/ArchitecturePipeline";
import DefenseBentoGrid from "@/components/DefenseBentoGrid";
import CostBenchmarkSection from "@/components/CostBenchmarkSection";
import RustCodeExplorer from "@/components/RustCodeExplorer";
import FaqSection from "@/components/FaqSection";
import FooterSection from "@/components/FooterSection";

export default function Home() {
  return (
    <main className="relative min-h-screen bg-[#060911] text-slate-100 overflow-x-hidden">
      {/* Background Interactive Layer */}
      <BackgroundCanvas />

      {/* Floating Header Navbar */}
      <Navbar />

      {/* Hero Section */}
      <HeroSection />

      {/* Interactive Triage Simulator */}
      <TriageSimulator />

      {/* 8-Step System Architecture Pipeline */}
      <ArchitecturePipeline />

      {/* Defense & Security Bento Grid */}
      <DefenseBentoGrid />

      {/* ROI & Cost Calculator */}
      <CostBenchmarkSection />

      {/* Rust Source Code Explorer */}
      <RustCodeExplorer />

      {/* SRE FAQ Section */}
      <FaqSection />

      {/* Cyberpunk Footer */}
      <FooterSection />
    </main>
  );
}
