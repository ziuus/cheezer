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

# 1. CSS Styles
css_to_replace = r""".tab-active \{ background-color: rgba\(245, 158, 11, 0.12\); color: #fbbf24; border-color: rgba\(245, 158, 11, 0.35\); box-shadow: 0 0 15px rgba\(245, 158, 11, 0.08\); \}
        \. \{ background: rgba\(15, 23, 42, 0.65\); backdrop-filter: blur\(12px\); border: 1px solid rgba\(51, 65, 85, 0.5\); \}"""
new_css = """.tab-active { background-color: #E8F0FE; color: #1A73E8; border-color: transparent; }
        .bg-surface { background: #ffffff; border: 1px solid #DADCE0; box-shadow: 0 1px 2px 0 rgba(60,64,67,0.3), 0 1px 3px 1px rgba(60,64,67,0.15); }
        .text-google-primary { color: #202124; }
        .text-google-secondary { color: #5F6368; }
        .google-header { background-color: #FFFFFF; border-bottom: 1px solid #DADCE0; }
"""
html_part = re.sub(css_to_replace, new_css, html_part)

# Also fix the weird `. { ... }` if the regex failed due to indentation
html_part = re.sub(r'\.tab-active\s*\{[^\}]+\}', '.tab-active { background-color: #E8F0FE; color: #1A73E8; border-color: transparent; }', html_part)
html_part = re.sub(r'\.\s*\{[^\}]+\}', '', html_part)

# 2. Ambient Background
html_part = re.sub(r'<!-- Ambient Background Lighting Mesh -->.*?</div>\s*</div>', '', html_part, flags=re.DOTALL)

# 3. Logo and Header
old_logo = r'<div class="flex-shrink-0 w-12 h-12 bg-amber-500 rounded-2xl flex items-center justify-center shadow-lg shadow-amber-500/20">\s*<i data-lucide="shield" class="w-6 h-6 text-amber-950"></i>\s*</div>'
new_logo = r"""<div class="flex-shrink-0 w-10 h-10 bg-[#1A73E8] rounded flex items-center justify-center">
                    <i data-lucide="shield-check" class="w-6 h-6 text-white"></i>
                </div>"""
html_part = re.sub(old_logo, new_logo, html_part)

# Remove the text-shadow from title if exists
html_part = html_part.replace('shadow-amber-500/20', '')

# 4. Status Badges at the top (Engine Active, Watchdog, OPA)
old_engine_badge = r'<span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-[#1E8E3E]/20 text-[#1E8E3E] border border-[#1E8E3E]/30">\s*<div class="w-2 h-2 rounded-full bg-[#1E8E3E] animate-pulse"></div>\s*ENGINE ACTIVE\s*</span>'
new_engine_badge = r'<span class="inline-flex items-center gap-1.5 px-3 py-1 rounded text-xs font-semibold bg-[#E6F4EA] text-[#1E8E3E]"><div class="w-2 h-2 rounded-full bg-[#1E8E3E] animate-pulse"></div>ENGINE ACTIVE</span>'
html_part = re.sub(old_engine_badge, new_engine_badge, html_part)

old_watchdog_badge = r'<span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-white text-[#5F6368] border border-[#DADCE0]">\s*<i data-lucide="activity" class="w-3.5 h-3.5"></i>\s*WATCHDOG ACTIVE\s*</span>'
new_watchdog_badge = r'<span class="inline-flex items-center gap-1.5 px-3 py-1 rounded text-xs font-semibold bg-white text-[#5F6368] border border-[#DADCE0]"><i data-lucide="activity" class="w-3.5 h-3.5 text-[#1A73E8]"></i>WATCHDOG ACTIVE</span>'
html_part = re.sub(old_watchdog_badge, new_watchdog_badge, html_part)

old_opa_badge = r'<span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-[#E8F0FE] text-[#1A73E8] border border-[#1A73E8]/30">\s*<i data-lucide="lock" class="w-3.5 h-3.5 text-[#1A73E8]"></i>\s*OPA FAIL-CLOSED\s*</span>'
new_opa_badge = r'<span class="inline-flex items-center gap-1.5 px-3 py-1 rounded text-xs font-semibold bg-white text-[#5F6368] border border-[#DADCE0]"><i data-lucide="lock" class="w-3.5 h-3.5 text-[#1A73E8]"></i>OPA FAIL-CLOSED</span>'
html_part = re.sub(old_opa_badge, new_opa_badge, html_part)

# 5. Fix card layouts and remove empty class attributes (leftover from "glass-card")
html_part = html_part.replace('class="  ', 'class="')
html_part = html_part.replace('class=" bg-white', 'class="bg-white')
html_part = html_part.replace('border border-[#DADCE0] rounded-xl p-5', 'border border-[#DADCE0] rounded-lg p-6 bg-white shadow-sm')

# 6. Change all rounded-xl to rounded-lg, typical for Google Cloud
html_part = html_part.replace('rounded-xl', 'rounded-lg')
html_part = html_part.replace('rounded-2xl', 'rounded-lg')

# 7. Update Tabs border styling
html_part = html_part.replace('pb-6 border-b border-[#DADCE0]/80 gap-4', 'p-4 gap-4 bg-white border-b border-[#DADCE0] w-full flex items-center justify-between shadow-sm')

# Let's wrap the Header in a true top App Bar.
html_part = html_part.replace('<div class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">', '<div class="relative z-10 w-full">')
html_part = html_part.replace('<!-- Header -->\n        <header class="flex flex-col md:flex-row md:items-center md:justify-between', '<!-- Header -->\n        <header class="flex flex-col md:flex-row md:items-center md:justify-between bg-white px-6 py-3 border-b border-[#DADCE0]')

# The tabs container should have a cleaner look
html_part = html_part.replace('class="flex flex-wrap gap-2 py-4 border-b border-[#DADCE0]/50"', 'class="flex flex-wrap gap-1 px-6 py-2 border-b border-[#DADCE0] bg-white w-full"')

# Add padding to main content
html_part = html_part.replace('<!-- Tab Contents -->', '<!-- Tab Contents -->\n        <div class="px-6 py-6 max-w-[1400px] mx-auto">')
# Close the div at the end before </body>
html_part = html_part.replace('    <!-- End Content -->', '    <!-- End Content -->\n        </div>')


# 8. Colors and Text Sizes
# Fix the big green 100.0% text
html_part = html_part.replace('text-5xl font-black text-[#1E8E3E] tracking-tight', 'text-4xl font-normal text-[#202124]')
html_part = html_part.replace('text-5xl font-black text-[#1A73E8] tracking-tight', 'text-4xl font-normal text-[#202124]')

# Fix card headers
html_part = html_part.replace('text-xs font-medium text-[#1A73E8] uppercase tracking-wider', 'text-sm font-medium text-[#202124]')
html_part = html_part.replace('text-xs font-medium text-[#1E8E3E] uppercase tracking-wider', 'text-sm font-medium text-[#202124]')

# Fix incident badges in JS
html_part = html_part.replace('text-[#1E8E3E] border border-emerald-500/30', 'text-[#1E8E3E] border-none')
html_part = html_part.replace('text-[#D93025] border border-rose-500/30', 'text-[#D93025] border-none')

# The top title Cheezer Core
html_part = html_part.replace('text-2xl font-bold tracking-tight', 'text-xl font-medium text-[#202124]')
html_part = html_part.replace('text-sm text-[#5F6368]', 'text-xs text-[#5F6368]')

with open("src/dashboard.rs", "w") as f:
    f.write(rust_part + html_part)

print("Redesign 3 applied.")
