import re

with open("src/dashboard.rs", "r") as f:
    content = f.read()

# Dictionary of lucide -> material symbols
icon_map = {
    "bar-chart-2": "bar_chart",
    "check-check": "done_all",
    "check-circle-2": "check_circle",
    "eye": "visibility",
    "file-text": "description",
    "git-fork": "fork_right",
    "github": "code",
    "globe": "public",
    "plus-circle": "add_circle",
    "rotate-cw": "refresh",
    "rss": "rss_feed",
    "shield-alert": "gpp_maybe",
    "shield-ban": "gpp_bad",
    "shield-plus": "health_and_safety",
    "trash-2": "delete",
    "x": "close",
    "server": "dns",
    "cpu": "memory"
}

# Replace static icons in HTML
for old, new in icon_map.items():
    # specifically for the ones inside the material-symbols-outlined span
    # e.g. <span class="material-symbols-outlined  ">shield-alert</span>
    pattern = rf'(<span class="material-symbols-outlined[^>]*>\s*){old}(\s*</span>)'
    content = re.sub(pattern, rf'\1{new}\2', content)

# Fix Javascript iconName logic
content = content.replace("iconName = 'server'", "iconName = 'dns'")
content = content.replace("iconName = 'globe'", "iconName = 'public'")
content = content.replace("iconName = 'cloud'", "iconName = 'cloud'")
content = content.replace("iconName = 'cpu'", "iconName = 'memory'")
content = content.replace("iconName = 'layers'", "iconName = 'layers'")

# Additionally, the status icons inside javascript for incidents:
# I need to ensure they match material symbols. Let's look for any other JS assignments.

with open("src/dashboard.rs", "w") as f:
    f.write(content)

print("Icons fixed.")
