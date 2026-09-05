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

# Replacements
replacements = {
    # Backgrounds
    "bg-slate-950": "bg-[#F8F9FA]",
    "bg-slate-900": "bg-white",
    "bg-slate-800": "bg-[#F1F3F4]",
    "bg-slate-800/50": "bg-[#F1F3F4]/50",
    "bg-slate-700": "bg-[#E8EAED]",
    
    # Text colors
    "text-slate-100": "text-[#202124]",
    "text-slate-200": "text-[#202124]",
    "text-slate-300": "text-[#5F6368]",
    "text-slate-400": "text-[#5F6368]",
    "text-slate-500": "text-[#80868B]",
    "text-slate-600": "text-[#9AA0A6]",
    "text-slate-950": "text-white",

    # Borders
    "border-slate-800": "border-[#DADCE0]",
    "border-slate-700": "border-[#E8EAED]",
    "divide-slate-800": "divide-[#DADCE0]",
    "divide-slate-700": "divide-[#E8EAED]",
    "ring-slate-800": "ring-[#DADCE0]",
    
    # Accents (Amber -> Google Blue)
    "text-amber-400": "text-[#1A73E8]",
    "text-amber-500": "text-[#1A73E8]",
    "bg-amber-500": "bg-[#1A73E8]",
    "bg-amber-600": "bg-[#174EA6]",
    "bg-amber-500/10": "bg-[#E8F0FE]",
    "bg-amber-500/20": "bg-[#D2E3FC]",
    "border-amber-500": "border-[#1A73E8]",
    "border-amber-500/30": "border-[#1A73E8]/30",
    "border-amber-500/50": "border-[#1A73E8]/50",
    "ring-amber-500": "ring-[#1A73E8]",
    
    # Success (Emerald -> Google Green)
    "text-emerald-400": "text-[#1E8E3E]",
    "text-emerald-500": "text-[#1E8E3E]",
    "bg-emerald-500": "bg-[#1E8E3E]",
    "bg-emerald-500/10": "bg-[#E6F4EA]",
    "bg-emerald-500/20": "bg-[#CEEAD6]",
    "border-emerald-500/20": "border-[#1E8E3E]/20",
    
    # Danger (Rose -> Google Red)
    "text-rose-400": "text-[#D93025]",
    "text-rose-500": "text-[#D93025]",
    "bg-rose-500": "bg-[#D93025]",
    "bg-rose-500/10": "bg-[#FCE8E6]",
    "border-rose-500/20": "border-[#D93025]/20",
    
    # Info (Cyan/Blue)
    "text-cyan-400": "text-[#1A73E8]",
    "text-cyan-500": "text-[#1A73E8]",
    "bg-cyan-500/10": "bg-[#E8F0FE]",
    "text-blue-400": "text-[#1A73E8]",
    
    # Shadows
    "shadow-lg": "shadow-[0_1px_2px_0_rgba(60,64,67,0.3),0_1px_3px_1px_rgba(60,64,67,0.15)]",
    "shadow-md": "shadow-[0_1px_2px_0_rgba(60,64,67,0.3),0_1px_3px_1px_rgba(60,64,67,0.15)]",
    "shadow-sm": "shadow-[0_1px_2px_0_rgba(60,64,67,0.3),0_1px_3px_1px_rgba(60,64,67,0.15)]",
}

for old, new in replacements.items():
    html_part = html_part.replace(old, new)

# Update the font family from Inter to Roboto
html_part = html_part.replace(
    'https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap',
    'https://fonts.googleapis.com/css2?family=Roboto:wght@300;400;500;700&display=swap'
)
html_part = html_part.replace('font-family: \'Inter\', sans-serif;', 'font-family: \'Roboto\', sans-serif;')

# Remove some dark-mode specific stuff like text-shadow if it exists, or update logo
html_part = html_part.replace('cheezer.', '<span style="color:#4285F4">C</span><span style="color:#EA4335">h</span><span style="color:#FBBC05">e</span><span style="color:#4285F4">e</span><span style="color:#34A853">z</span><span style="color:#EA4335">e</span><span style="color:#4285F4">r</span>')
html_part = html_part.replace('🧀', '') # Google doesn't use emojis in logos typically

# Google UI elements are more rounded in Material 3
html_part = html_part.replace('rounded-md', 'rounded-lg')
html_part = html_part.replace('rounded-lg', 'rounded-xl') # Elevate border radii
html_part = html_part.replace('rounded-full', 'rounded-full') # Keep pills

# For navigation, Google sidebar often uses a specific pill shape for active item
# We can find `bg-[#F1F3F4]` in the sidebar and make it `bg-[#E8F0FE] text-[#1A73E8] rounded-r-full mr-4`
# But it might be complex via script. Let's just do class replacements.

# Let's add the Devin AI CLI button next to remediation actions.
# We will inject a Devin button in the incidents view and remediation history view.

devin_btn = """<button onclick="alert('devin --task \\'Debug incident ' + incident.id + '\\'')" class="px-3 py-1 bg-white border border-[#DADCE0] text-[#1A73E8] rounded-xl hover:bg-[#F8F9FA] transition-colors flex items-center gap-1 text-sm"><svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"></path></svg>Debug with Devin</button>"""

# Find where incidents are rendered (around line 2140)
# ' + incident.status + '</span></td><td class="px-4 py-4"><button onclick="viewIncidentDetail(\\'' + incident.id + '\\')" class="text-[#1A73E8] hover:text-[#174EA6] transition-colors font-medium text-sm">Review &rarr;</button></td></tr>';
old_incident_action = "class=\\\"text-[#1A73E8] hover:text-[#174EA6] transition-colors font-medium text-sm\\\">Review &rarr;</button>"
new_incident_action = old_incident_action + "</div><div class=\\\"mt-2\\\">" + devin_btn.replace("\"", "\\\"").replace("'", "\\'") + "</div>"

html_part = html_part.replace(old_incident_action, new_incident_action)

with open("src/dashboard.rs", "w") as f:
    f.write(rust_part + html_part)

print("Redesign complete.")
