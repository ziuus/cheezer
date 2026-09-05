import re

with open("src/dashboard.rs", "r") as f:
    content = f.read()

# Expanded list of connection services covering Serverless/PaaS, Single-Host, Lightweight Orchestrators & K8s
new_conn_services = """        ("GitHub Auth API", "github", "https://api.github.com", "OAuth / Personal Access Token"),
        ("Vercel Platform API", "vercel", "https://api.vercel.com", "Serverless Web PaaS"),
        ("Render PaaS API", "render", "https://api.render.com", "Git-based Application PaaS"),
        ("Kubernetes API Server", "k8s", "https://kubernetes.default.svc", "In-Cluster ServiceAccount Token"),
        ("AWS Lambda & App Runner", "lambda", "https://lambda.us-east-1.amazonaws.com", "Serverless & Managed Containers"),
        ("Google Cloud Run & Functions", "cloudrun", "https://run.googleapis.com", "Stateless Serverless Containers"),
        ("Azure Functions & ACI", "azure", "https://management.azure.com", "Serverless & On-Demand Containers"),
        ("Fly.io Platform Gateway", "flyio", "https://api.fly.io", "Git-based PaaS & Edge Containers"),
        ("Railway.app Platform", "railway", "https://backboard.railway.app", "Developer-Friendly Git PaaS"),
        ("Heroku Platform API", "heroku", "https://api.heroku.com", "PaaS Dyno Management"),
        ("Netlify Platform API", "netlify", "https://api.netlify.com", "Git-based Web & Functions"),
        ("Platform.sh GitOps PaaS", "platformsh", "https://api.platform.sh", "GitOps Containerized PaaS"),
        ("Docker Engine & Compose", "docker", "https://localhost:2375", "Single-Host Container Runtime"),
        ("Podman + systemd Service", "podman", "https://localhost:8888", "Daemonless OS-Init Containers"),
        ("Portainer / Ansible Gateway", "portainer", "https://localhost:9443", "Host Scripts & Container Manager"),
        ("Docker Swarm Manager", "swarm", "https://localhost:2377", "Lightweight Container Cluster"),
        ("HashiCorp Nomad Engine", "nomad", "http://localhost:4646", "Workload & Task Orchestrator"),
        ("Devin AI Autonomous Agent API", "devin", "https://api.devin.ai", "Autonomous AI Code Fixes"),
        ("Grafana / OpenTelemetry Collector", "grafana", "http://127.0.0.1:9090", "Telemetry & Webhooks"),"""

# Match old conn_services in get_metrics_json
old_conn_pattern = r'let conn_services = vec!\[\s*\("GitHub Auth API".*?\);'
content = re.sub(old_conn_pattern, f'let conn_services = vec![\n{new_conn_services}\n    ];', content, flags=re.DOTALL)

# Also update test_connection switch statement to handle all platforms cleanly
old_test_switch = r'let target_url = match req\.name\.as_str\(\) \{.*?\};'
new_test_switch = """let target_url = match req.name.as_str() {
        "Kubernetes Cluster API" | "Kubernetes API Server" => "https://kubernetes.default.svc",
        "AWS Cloud Platform" | "AWS Lambda & App Runner" => "https://ec2.amazonaws.com",
        "Google Cloud Platform" | "Google Cloud Run & Functions" => "https://compute.googleapis.com",
        "Azure Functions & ACI" => "https://management.azure.com",
        "Vercel REST API Gateway" | "Vercel Platform API" => "https://api.vercel.com",
        "Render REST API Gateway" | "Render PaaS API" => "https://api.render.com",
        "Fly.io Platform Gateway" => "https://api.fly.io",
        "Railway.app Platform" => "https://backboard.railway.app",
        "Heroku Platform API" => "https://api.heroku.com",
        "Netlify Platform API" => "https://api.netlify.com",
        "Platform.sh GitOps PaaS" => "https://api.platform.sh",
        "GitHub GitOps Repository" | "GitHub Auth API" => "https://api.github.com",
        "Devin AI Autonomous Engineer API" | "Devin AI Autonomous Agent API" => "https://api.devin.ai",
        "Docker Engine & Compose" => "https://docker.com",
        "Podman + systemd Service" => "https://podman.io",
        "Portainer / Ansible Gateway" => "https://portainer.io",
        "Docker Swarm Manager" => "https://docker.com",
        "HashiCorp Nomad Engine" => "https://nomadproject.io",
        "Grafana / OpenTelemetry Collector" => "http://127.0.0.1:9090",
        _ => "http://127.0.0.1:9090",
    };"""

content = re.sub(old_test_switch, new_test_switch, content, flags=re.DOTALL)

with open("src/dashboard.rs", "w") as f:
    f.write(content)

print("Expanded deployment platforms updated in dashboard.rs!")
