import re

with open("src/dashboard.rs", "r") as f:
    content = f.read()

# Isolate the HTML
html_start = content.find('const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>')
rust_part = content[:html_start]
html_part = content[html_start:]

# 1. Clean the header completely
header_pattern = re.compile(r'<!-- Header -->\s*<header.*?</header>', re.DOTALL)

clean_header = """<!-- Header -->
        <header class="flex items-center justify-between bg-[#FFFFFF] px-6 py-3 border-b border-[#DADCE0]">
            <div class="flex items-center space-x-4">
                <div class="flex-shrink-0 w-8 h-8 flex items-center justify-center">
                    <!-- Clean Google-style Logo Icon -->
                    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <path d="M12 2L3 6V11C3 16.55 6.84 21.74 12 23C17.16 21.74 21 16.55 21 11V6L12 2Z" fill="#1A73E8"/>
                        <path d="M12 11.9999L3 6V11C3 16.55 6.84 21.74 12 23V11.9999Z" fill="#174EA6"/>
                        <path d="M21 6L12 2V11.9999L21 6Z" fill="#4285F4"/>
                    </svg>
                </div>
                <h1 class="text-[22px] font-normal text-[#1F1F1F] tracking-tight" style="font-family: 'Outfit', 'Google Sans', sans-serif;">
                    Cheezer Core
                </h1>
            </div>
            
            <!-- Clean utility section (e.g. Account, Settings icons) -->
            <div class="flex items-center space-x-3 text-[#5F6368]">
                <button class="w-10 h-10 rounded-full hover:bg-[#F8F9FA] flex items-center justify-center transition">
                    <span class="material-symbols-outlined">help</span>
                </button>
                <button class="w-10 h-10 rounded-full hover:bg-[#F8F9FA] flex items-center justify-center transition">
                    <span class="material-symbols-outlined">settings</span>
                </button>
                <div class="w-8 h-8 rounded-full bg-[#1A73E8] text-white flex items-center justify-center text-sm font-medium ml-2 cursor-pointer">
                    A
                </div>
            </div>
        </header>"""

html_part = header_pattern.sub(clean_header, html_part)

with open("src/dashboard.rs", "w") as f:
    f.write(rust_part + html_part)

print("Header cleaned up.")
