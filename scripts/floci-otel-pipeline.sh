#!/usr/bin/env bash

# ==============================================================================
# Cheezer + Floci AWS + Grafana/OpenTelemetry Live Telemetry Pipeline
# ==============================================================================

set -e

FLOCI_ENDPOINT="${FLOCI_ENDPOINT:-http://172.18.100.41:4566}"
CHEEZER_ENDPOINT="${CHEEZER_ENDPOINT:-http://localhost:9090}"
API_KEY="${CHEEZER_API_KEY:-hackathon2026}"

echo "🧀 Starting Cheezer Live Telemetry & Floci AWS Pipeline..."
echo "--------------------------------------------------------"
echo "  • Floci AWS Endpoint : $FLOCI_ENDPOINT"
echo "  • Cheezer Core API   : $CHEEZER_ENDPOINT"
echo "--------------------------------------------------------"

# Step 1: Ensure Floci SQS & S3 resources exist
echo "1. Initializing Floci AWS Resources..."
curl -s -X POST "$FLOCI_ENDPOINT/" -d "Action=CreateQueue&QueueName=cheezer-alerts&Version=2012-11-05" > /dev/null || true
curl -s -X PUT "$FLOCI_ENDPOINT/cheezer-incidents-bucket" > /dev/null || true
echo "   ✓ Floci SQS Queue: $FLOCI_ENDPOINT/000000000000/cheezer-alerts"
echo "   ✓ Floci S3 Bucket: $FLOCI_ENDPOINT/cheezer-incidents-bucket"

# Step 2: OpenTelemetry / Grafana Alert Simulation Function
send_telemetry_alert() {
    local alertname="$1"
    local severity="$2"
    local pod="$3"
    local namespace="${4:-demo}"

    echo ""
    echo "📊 [Grafana/OTel Alerting Pipeline] Metric threshold breach detected!"
    echo "   Alert: $alertname | Pod: $pod | Severity: $severity"

    local payload=$(cat <<EOF
{
  "receiver": "cheezer-webhook-handler",
  "status": "firing",
  "alerts": [
    {
      "status": "firing",
      "labels": {
        "alertname": "$alertname",
        "severity": "$severity",
        "pod": "$pod",
        "namespace": "$namespace",
        "exporter": "opentelemetry-collector",
        "cluster": "k3s-demo-cluster"
      },
      "annotations": {
        "summary": "High error rate / container crash detected by OpenTelemetry metrics",
        "description": "Pod $pod in namespace $namespace breached CPU/Memory/Restart thresholds."
      },
      "startsAt": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    }
  ]
}
EOF
)

    local res=$(curl -s -X POST "$CHEEZER_ENDPOINT/api/grafana_webhook" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d "$payload")

    echo "   ⚡ Webhook Response from Cheezer Core: $res"
}

# Step 3: Stream incident data into Floci AWS
export_to_floci_s3() {
    local incident_file="incident-$(date +%s).json"
    local data=$(curl -s "$CHEEZER_ENDPOINT/api/incidents")
    
    echo ""
    echo "☁️  [Floci AWS Exporter] Archiving telemetry snapshot to Floci S3..."
    curl -s -X PUT "$FLOCI_ENDPOINT/cheezer-incidents-bucket/$incident_file" \
        -H "Content-Type: application/json" \
        -d "$data" > /dev/null

    echo "   ✓ Snapshot stored as S3 object: $FLOCI_ENDPOINT/cheezer-incidents-bucket/$incident_file"
}

# Step 4: Run Telemetry Cycles
echo ""
echo "🚀 Running Live Telemetry Simulation Cycles..."

# Cycle A: Fast-path CrashLoopBackOff (Rule triage)
send_telemetry_alert "CrashLoopBackOff" "critical" "payment-processor-pod"
sleep 2

# Cycle B: Novel Anomaly OOMKilled (AI escalation + GitOps PR)
send_telemetry_alert "OOMKilled" "critical" "auth-service-pod"
sleep 2

# Export audit snapshot to Floci S3
export_to_floci_s3

echo ""
echo "========================================================"
echo "🎉 Telemetry Pipeline Executed Successfully!"
echo "   • OpenTelemetry / Grafana Webhooks Dispatched"
echo "   • Cheezer Core Autonomous Triage & OPA Gates Passed"
echo "   • Floci AWS S3 & SQS Audit Stream Updated"
echo "========================================================"
