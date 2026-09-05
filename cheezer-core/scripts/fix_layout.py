import re

with open("src/dashboard.rs", "r") as f:
    content = f.read()

# Fix conflicting CSS in style block
content = re.sub(r'\.tab-active \{ background-color: #E8F0FE;.*?\}\n', '', content)
content = re.sub(r'\.bg-surface \{ background: #ffffff;.*?\}\n', '', content)
content = re.sub(r'\.text-google-primary \{.*?\}\n', '', content)
content = re.sub(r'\.text-google-secondary \{.*?\}\n', '', content)
content = re.sub(r'\.google-header \{.*?\}\n', '', content)

# Fix metric cards
content = content.replace(
    '<div class=" rounded-lg p-5 ">',
    '<div class="bg-white border border-[#DADCE0] rounded-2xl p-5 flex flex-col justify-between shadow-sm">'
)
content = content.replace(
    '<div class="bg-white border-[#DADCE0] rounded-lg p-5 ">',
    '<div class="bg-white border border-[#DADCE0] rounded-2xl p-5 flex flex-col justify-between shadow-sm">'
)

# Fix KPI text (remove font-mono, use normal bold numbers)
content = re.sub(
    r'<div class="text-3xl font-extrabold text-\[([^\]]+)\] mt-2 font-mono" id="kpi-([^"]+)">',
    r'<div class="text-4xl font-medium text-[\1] mt-3" id="kpi-\2">',
    content
)

# Fix sections (table containers)
content = content.replace(
    '<section class=" rounded-lg p-6 shadow-xl">',
    '<section class="bg-white border border-[#DADCE0] rounded-2xl p-6 shadow-sm">'
)
content = content.replace(
    '<section class="bg-white border border-[#DADCE0] rounded-lg p-6 ">',
    '<section class="bg-white border border-[#DADCE0] rounded-2xl p-6 shadow-sm">'
)
content = content.replace(
    '<div class="bg-[#F3F6FC]/80 border border-[#DADCE0]/80 p-4 rounded-lg">',
    '<div class="bg-[#F8F9FA] border border-[#DADCE0] p-4 rounded-2xl">'
)

# Fix tabs navigation bar container
content = content.replace(
    '<nav class="flex flex-wrap items-center space-x-2 my-6 border-b border-[#DADCE0]/80 pb-3 gap-y-2">',
    '<nav class="flex flex-wrap items-center space-x-2 my-6 border-b border-[#DADCE0] pb-2 gap-y-2">'
)

# Fix tab Javascript classes
content = content.replace(
    'btn.className = "px-4 py-2 rounded-lg text-xs font-semibold transition text-[#444746] hover:text-[#1F1F1F] hover:bg-white/60 border border-transparent flex items-center space-x-2";',
    'btn.className = "px-5 py-2.5 rounded-full text-sm font-medium transition text-[#444746] hover:bg-[#F3F6FC] hover:text-[#1F1F1F] flex items-center space-x-2";'
)
content = content.replace(
    'activeBtn.className = "tab-active px-5 py-2 flex items-center space-x-2";',
    'activeBtn.className = "bg-[#C2E7FF] text-[#001D35] px-5 py-2.5 rounded-full text-sm font-medium flex items-center space-x-2";'
)

# Fix table header rows (make them cleaner)
content = content.replace(
    '<tr class="text-[11px] font-mono uppercase tracking-wider text-[#444746] border-b border-[#DADCE0] bg-[#F3F6FC]/60">',
    '<tr class="text-xs font-medium text-[#444746] border-b border-[#DADCE0] bg-[#F8F9FA]">'
)

# Remove the weird .tab-active from style since we inline it
content = re.sub(r'\.tab-active \{.*?\}\n', '', content)
content = re.sub(r'\.tab-inactive \{.*?\}\n', '', content)
content = re.sub(r'\.tab-inactive:hover \{.*?\}\n', '', content)

with open("src/dashboard.rs", "w") as f:
    f.write(content)

print("Layout fixes applied.")
