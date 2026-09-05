import re

with open("src/dashboard.rs", "r") as f:
    content = f.read()

# Separate the Rust code and the HTML string
html_start = content.find('const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>')
if html_start == -1:
    print("Could not find DASHBOARD_HTML")
    exit(1)

rust_part = content[:html_start]
html_part = content[html_start:]

# Remove dark mode artifacts that the first script missed
replacements = {
    "bg-amber-950/20": "bg-white",
    "bg-amber-900/20": "bg-white",
    "bg-[#F1F3F4]/80": "bg-white",
    "hover:bg-[#E8EAED]": "hover:bg-[#F8F9FA]",
    "bg-purple-950/80": "bg-white",
    "hover:bg-purple-900": "hover:bg-[#F8F9FA]",
    "text-amber-300": "text-[#1A73E8]",
    "text-purple-300": "text-[#9333EA]",
    "text-purple-400": "text-[#9333EA]",
    "text-sky-400": "text-[#1A73E8]",
    "border-purple-500/40": "border-[#DADCE0]",
    "border-[#1A73E8]/20": "border-[#DADCE0]",
    "shadow-purple-500/20": "shadow-sm",
    "hover:bg-white/40": "hover:bg-[#F1F3F4]",  # Table rows hover
    "glass-card": "",
    "backdrop-blur": "",
    "blur-3xl": "",
    "bg-[#1A73E8]/10": "bg-transparent", # Removing the glowing blobs
    "bg-emerald-500/10": "bg-[#E6F4EA]",
    "text-emerald-400": "text-[#1E8E3E]",
    "border-emerald-500/30": "border-[#1E8E3E]/30",
    "bg-rose-500/10": "bg-[#FCE8E6]",
    "text-rose-400": "text-[#D93025]",
    "border-rose-500/30": "border-[#D93025]/30",
    "text-slate-400": "text-[#5F6368]",
    "text-slate-300": "text-[#5F6368]",
    "bg-slate-800": "bg-[#F1F3F4]",
    "bg-slate-900/40": "hover:bg-[#F8F9FA]",
    "border-[#1A73E8]/30": "border-[#DADCE0]", # Standard card borders
    "bg-gray-900": "bg-[#F8F9FA]",
    "text-white": "text-[#202124]", # Some places had white text for headings in dark mode, change to dark
    "text-[#202124] bg-white": "text-[#1A73E8] bg-[#1A73E8]", # Reverting buttons that got broken
    "hover:bg-amber-400": "hover:bg-[#174EA6]",
}

for old, new in replacements.items():
    html_part = html_part.replace(old, new)

# Fix primary buttons that might have been broken by 'text-white' to 'text-[#202124]'
html_part = html_part.replace('class="bg-[#1A73E8] hover:bg-[#174EA6] text-[#202124]', 'class="bg-[#1A73E8] hover:bg-[#174EA6] text-white')
html_part = html_part.replace('class="px-4 py-2 bg-[#1E8E3E] hover:bg-[#174EA6] text-[#202124]', 'class="px-4 py-2 bg-[#1E8E3E] hover:bg-green-700 text-white')
html_part = html_part.replace('px-3 py-1 bg-white border border-[#DADCE0] text-[#1A73E8]', 'px-3 py-1 bg-white border border-[#DADCE0] text-[#1A73E8]') # Devin button

# We need the global body to have a light background and dark text
html_part = html_part.replace('<body class="bg-[#F8F9FA] text-[#202124] h-screen overflow-hidden selection:bg-[#1A73E8]/30">', '<body class="bg-[#F8F9FA] text-[#202124] h-screen overflow-hidden font-sans">')
html_part = html_part.replace('<body class="bg-slate-950 text-slate-200 h-screen overflow-hidden selection:bg-amber-500/30">', '<body class="bg-[#F8F9FA] text-[#202124] h-screen overflow-hidden font-sans">')

with open("src/dashboard.rs", "w") as f:
    f.write(rust_part + html_part)

print("Redesign 2 complete.")
