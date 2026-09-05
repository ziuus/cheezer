import re

with open("src/dashboard.rs", "r") as f:
    content = f.read()

# Fix 1: Add-Watcher Modal Overlay Backdrop & Card Background
content = content.replace(
    '<div id="add-watcher-modal" class="fixed inset-0 z-50 bg-[#F3F6FC]/80 -sm hidden items-center justify-center p-4">',
    '<div id="add-watcher-modal" class="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm hidden items-center justify-center p-4">'
)
content = content.replace(
    '<div class=" rounded-lg p-6 max-w-lg w-full border border-[#DADCE0] shadow-2xl space-y-5">',
    '<div class="bg-white rounded-3xl p-6 max-w-lg w-full border border-[#DADCE0] shadow-2xl space-y-5 text-[#1F1F1F] z-50">'
)

# Fix 2: Incident-Doc Modal Overlay Backdrop & Card Background
content = content.replace(
    '<div id="incident-doc-modal" class="fixed inset-0 z-50 bg-[#F3F6FC]/80 -sm hidden items-center justify-center p-4">',
    '<div id="incident-doc-modal" class="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm hidden items-center justify-center p-4">'
)
content = content.replace(
    '<div class=" rounded-lg p-6 max-w-2xl w-full border border-[#DADCE0] shadow-2xl space-y-5 max-h-[90vh] overflow-y-auto">',
    '<div class="bg-white rounded-3xl p-6 max-w-2xl w-full border border-[#DADCE0] shadow-2xl space-y-5 max-h-[90vh] overflow-y-auto text-[#1F1F1F] z-50">'
)

# Fix 3: OAuth Modal Overlay Backdrop & Card Background
content = content.replace(
    '<div id="oauth-modal" class="fixed inset-0 z-50 bg-[#1F1F1F]/40 backdrop-blur-sm hidden items-center justify-center p-4">',
    '<div id="oauth-modal" class="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm hidden items-center justify-center p-4">'
)
content = content.replace(
    '<div class="bg-white border border-[#DADCE0] rounded-2xl w-full max-w-md p-6 shadow-2xl space-y-5">',
    '<div class="bg-white border border-[#DADCE0] rounded-3xl w-full max-w-md p-6 shadow-2xl space-y-5 text-[#1F1F1F] z-50">'
)

# Fix 4: Main layout padding & grid spacing
content = content.replace(
    '<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">',
    '<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">'
)

# Fix 5: Ensure all modal buttons have clean Google M3 styling
content = content.replace(
    'class="px-4 py-2 rounded-lg text-xs font-mono bg-[#F1F3F4] text-[#444746] hover:bg-[#F3F6FC] transition"',
    'class="px-4 py-2.5 rounded-full text-xs font-medium bg-[#F1F3F4] text-[#444746] hover:bg-[#E8EAED] transition"'
)
content = content.replace(
    'class="px-4 py-2 rounded-lg text-xs font-mono font-bold bg-[#0B57D0] hover:bg-[#174EA6] text-[#1F1F1F] transition flex items-center gap-1.5"',
    'class="px-5 py-2.5 rounded-full text-xs font-medium bg-[#1A73E8] hover:bg-[#174EA6] text-white transition flex items-center gap-1.5 shadow"'
)

with open("src/dashboard.rs", "w") as f:
    f.write(content)

print("All modal backgrounds, backdrops, and spacing fixes applied!")
