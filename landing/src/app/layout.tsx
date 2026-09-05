import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Cheezer.rs — Autonomous Out-of-Band K8s Incident Remediation Engine",
  description: "Autonomous out-of-band Kubernetes incident remediation engine in Rust. Zero AI cost for known alerts, fail-closed OPA policy gate, and TOCTOU state revalidation.",
  keywords: ["kubernetes", "rust", "incident response", "remediation", "opa", "rego", "zero trust", "devops", "sre"],
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased dark`}
    >
      <body className="min-h-full flex flex-col bg-[#060911] text-slate-100 font-sans selection:bg-amber-500/30 selection:text-amber-300">
        {children}
      </body>
    </html>
  );
}
