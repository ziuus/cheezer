import re

with open("src/dashboard.rs", "r") as f:
    content = f.read()

html_start = content.find('const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>')
rust_part = content[:html_start]
html_part = content[html_start:]

# 1. Add Material Symbols & Font (Outfit for headers to mimic Google Sans, Roboto for body)
head_end = html_part.find('</head>')
material_fonts = """
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@400;500;600;700&family=Roboto:wght@400;500;700&display=swap" rel="stylesheet">
    <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0" rel="stylesheet" />
    <style>
        body { font-family: 'Roboto', sans-serif; background-color: #F3F6FC; }
        h1, h2, h3, h4, h5, .google-sans { font-family: 'Outfit', 'Google Sans', sans-serif; }
        .material-symbols-outlined { font-size: 20px; font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24; }
        .material-symbols-outlined.filled { font-variation-settings: 'FILL' 1; }
        
        /* Material 3 Tabs */
        .tab-active { background-color: #C2E7FF; color: #001D35; font-weight: 500; border-radius: 9999px; }
        .tab-inactive { color: #444746; font-weight: 500; border-radius: 9999px; }
        .tab-inactive:hover { background-color: #1F1F1F14; }
        
        /* Material 3 Card */
        .m3-card { background-color: #FFFFFF; border: 1px solid #C7C7C7; border-radius: 12px; }
        
        /* Material 3 Primary Button */
        .btn-primary { background-color: #0B57D0; color: #FFFFFF; border-radius: 9999px; font-weight: 500; padding: 10px 24px; transition: background-color 0.2s; }
        .btn-primary:hover { background-color: #0842A0; box-shadow: 0 1px 3px rgba(0,0,0,0.2); }
        
        /* Material 3 Outlined Button */
        .btn-outlined { background-color: transparent; border: 1px solid #747775; color: #0B57D0; border-radius: 9999px; font-weight: 500; padding: 10px 24px; transition: background-color 0.2s; }
        .btn-outlined:hover { background-color: #0B57D014; }
    </style>
"""
html_part = html_part[:head_end] + material_fonts + html_part[head_end:]

# 2. Update Body & Fonts
html_part = html_part.replace('font-sans', 'font-roboto')
html_part = html_part.replace('bg-[#F8F9FA]', 'bg-[#F3F6FC]')
html_part = html_part.replace('text-[#202124]', 'text-[#1F1F1F]')
html_part = html_part.replace('text-[#5F6368]', 'text-[#444746]')

# 3. Swap ALL Lucide Icons to Material Symbols
# We use regex to find `<i data-lucide="icon-name" class="..."></i>`
def repl_lucide(m):
    icon_name = m.group(1)
    classes = m.group(2)
    # Map lucide names to material names
    mapping = {
        'shield-check': 'security',
        'shield': 'security',
        'alert-triangle': 'warning',
        'activity': 'monitoring',
        'lock': 'lock',
        'terminal': 'terminal',
        'check-circle': 'check_circle',
        'x-circle': 'cancel',
        'server': 'dns',
        'cloud': 'cloud',
        'database': 'database',
        'cpu': 'memory',
        'settings': 'settings',
        'search': 'search',
        'refresh-cw': 'refresh',
        'link': 'link',
        'bar-chart': 'bar_chart',
        'history': 'history',
        'zap': 'bolt',
        'play': 'play_arrow',
        'bot': 'smart_toy',
        'alert-circle': 'error',
        'info': 'info',
        'box': 'inventory_2',
        'git-merge': 'call_merge',
        'code': 'code',
    }
    mat_name = mapping.get(icon_name, icon_name)
    # Remove width/height from material symbols as font-size handles it, keep color
    classes = re.sub(r'w-\d+(\.\d+)?', '', classes)
    classes = re.sub(r'h-\d+(\.\d+)?', '', classes)
    return f'<span class="material-symbols-outlined {classes}">{mat_name}</span>'

html_part = re.sub(r'<i data-lucide="([^"]+)" class="([^"]*)"></i>', repl_lucide, html_part)
# Remove lucide script inclusion and init
html_part = re.sub(r'<script src="https://unpkg.com/lucide@latest"></script>', '', html_part)
html_part = re.sub(r'lucide\.createIcons\(\);', '', html_part)

# 4. Update Header and Top App Bar
html_part = html_part.replace('bg-white px-6 py-3 border-b border-[#DADCE0]', 'bg-[#F3F6FC] px-6 py-4')
# Cheezer Core title to Google Sans
html_part = html_part.replace('text-xl font-medium text-[#1F1F1F]', 'text-2xl font-normal text-[#1F1F1F] google-sans tracking-tight')

# 5. Fix Javascript for Tabs (Material 3 style)
js_active = 'activeBtn.className = "tab-active px-5 py-2 flex items-center space-x-2";'
js_inactive = 'btn.className = "tab-inactive px-5 py-2 flex items-center space-x-2 transition cursor-pointer";'

html_part = re.sub(r'activeBtn\.className = "tab-active[^"]*";', js_active, html_part)
html_part = re.sub(r'btn\.className = "px-4 py-2[^"]*text-\[#5F6368\][^"]*";', js_inactive, html_part)
# Also replace initial classes in HTML
html_part = html_part.replace('class="px-4 py-2 rounded-lg text-xs font-semibold transition text-slate-400 hover:text-white hover:bg-slate-900/60 border border-transparent flex items-center space-x-2"', 'class="tab-inactive px-5 py-2 flex items-center space-x-2 transition cursor-pointer"')
html_part = html_part.replace('class="tab-active px-4 py-2 rounded-lg text-xs font-semibold transition border flex items-center space-x-2"', 'class="tab-active px-5 py-2 flex items-center space-x-2"')
# Tab container border removal (M3 doesn't typically border the tab row if using pills)
html_part = html_part.replace('class="flex flex-wrap gap-1 px-6 py-2 border-b border-[#DADCE0] bg-white w-full"', 'class="flex flex-wrap gap-2 px-6 py-2 bg-[#F3F6FC] w-full"')

# 6. Buttons
# Apply btn-primary to key buttons
html_part = html_part.replace('class="bg-[#1A73E8] hover:bg-[#174EA6] text-white', 'class="btn-primary flex items-center gap-2')
html_part = html_part.replace('class="px-4 py-2 bg-[#1E8E3E] hover:bg-green-700 text-white rounded-lg font-medium transition"', 'class="btn-primary !bg-[#146C2E] flex items-center gap-2"') # Green primary
# Apply btn-outlined
html_part = html_part.replace('px-3 py-1 bg-white border border-[#DADCE0] text-[#1A73E8] rounded hover:bg-[#F8F9FA] transition', 'btn-outlined')

# 7. Card styling
html_part = html_part.replace('bg-white shadow-sm', 'm3-card')
html_part = html_part.replace('bg-white border border-[#DADCE0]', 'm3-card')

# 8. Logo Background
html_part = html_part.replace('bg-[#1A73E8]', 'bg-[#0B57D0]')
html_part = html_part.replace('text-[#1A73E8]', 'text-[#0B57D0]')

# 9. Badges
# Engine Active
html_part = html_part.replace('bg-[#E6F4EA] text-[#1E8E3E]', 'bg-[#C4EED0] text-[#0F5223] rounded-full px-3 py-1')
# Watchdog Active
html_part = html_part.replace('bg-white text-[#444746] border border-[#DADCE0]', 'bg-[#E3E3E3] text-[#1F1F1F] rounded-full border-0 px-3 py-1')
# OPA
html_part = html_part.replace('bg-white text-[#444746] border border-[#DADCE0]', 'bg-[#E3E3E3] text-[#1F1F1F] rounded-full border-0 px-3 py-1')

# 10. Search bar
html_part = html_part.replace('bg-[#F8F9FA] border border-[#DADCE0] rounded-lg', 'bg-[#FFFFFF] border border-[#747775] rounded-full px-4')

with open("src/dashboard.rs", "w") as f:
    f.write(rust_part + html_part)

print("Material 3 Complete overhaul applied.")
