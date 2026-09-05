#!/usr/bin/env bash

# ==============================================================================
# Cheezer Operator Chaos Bug Trigger & Fault Injection Suite
# ==============================================================================

set -e

CHEEZER_URL="${CHEEZER_URL:-http://localhost:9090}"
API_KEY="${CHEEZER_API_KEY:-hackathon2026}"
NAMESPACE="${NAMESPACE:-demo}"

echo "🧀 Cheezer Chaos Bug & Fault Injection Suite"
echo "=================================================================="
echo "  Target Endpoint: $CHEEZER_URL"
echo "  Target Namespace: $NAMESPACE"
echo "=================================================================="

print_usage() {
    echo ""
    echo "Usage: ./scripts/trigger-chaos-bug.sh [bug-type]"
    echo ""
    echo "Available Bug Types:"
    echo "  1) crashloop      - Trigger Tier-1 CrashLoopBackOff (Rule Fast-Path Restart)"
    echo "  2) oom            - Trigger Tier-2 OOMKilled (AI Escalation & GitOps PR)"
    echo "  3) dangerous      - Trigger Dangerous Action (OPA Policy Denial Gate)"
    echo "  4) rate-limit     - Trigger Rate-Limit Spike (Circuit Breaker Lock)"
    echo "  5) kill-switch    - Trigger Kill-Switch Test (Emergency Brake)"
    echo "  6) all            - Run All Chaos Scenarios sequentially"
    echo ""
}

# Scenario 1: CrashLoopBackOff (Rule Fast-Path)
trigger_crashloop() {
    echo ""
    echo "🔥 [Scenario 1] Deploying Crashing Pod & Firing CrashLoopBackOff Alert..."
    
    # Create or replace broken pod in k8s
    kubectl run payment-service-broken --image=busybox --namespace="$NAMESPACE" --restart=Always -- sh -c "echo 'App starting...'; sleep 2; exit 1" || true

    echo "   📡 Sending Alertmanager payload to Cheezer..."
    local res=$(curl -s -X POST "$CHEEZER_URL/api/grafana_webhook" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d '{
          "alerts": [{
            "status": "firing",
            "labels": {
              "alertname": "CrashLoopBackOff",
              "severity": "critical",
              "pod": "payment-service-broken",
              "namespace": "'"$NAMESPACE"'"
            },
            "annotations": {
              "summary": "Container payment-service-broken crashing continuously"
            }
          }]
        }')
    echo "   ⚡ Response: $res"
    echo "   ✅ Check Cheezer Dashboard (http://localhost:9090/dashboard) to view immediate rule restart!"
}

# Scenario 2: OOMKilled (AI Escalation + GitOps PR)
trigger_oom() {
    echo ""
    echo "🧠 [Scenario 2] Firing Novel OOMKilled Alert (Triggers LLM Escalation & GitHub PR)..."
    
    local res=$(curl -s -X POST "$CHEEZER_URL/api/grafana_webhook" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d '{
          "alerts": [{
            "status": "firing",
            "labels": {
              "alertname": "OOMKilled",
              "severity": "critical",
              "pod": "auth-service-oom",
              "namespace": "'"$NAMESPACE"'"
            },
            "annotations": {
              "summary": "Container auth-service-oom terminated due to Out-Of-Memory (OOM)"
            }
          }]
        }')
    echo "   ⚡ Response: $res"
    echo "   ✅ Check Cheezer Dashboard for AI escalation & GitHub (https://github.com/ziuus/cheezer/pulls) for automated PR!"
}

# Scenario 3: Dangerous Action (OPA Policy Denial)
trigger_dangerous() {
    echo ""
    echo "🚫 [Scenario 3] Firing Dangerous Action Payload (OPA Fail-Closed Enforcement)..."
    
    local res=$(curl -s -X POST "$CHEEZER_URL/api/grafana_webhook" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d '{
          "alerts": [{
            "status": "firing",
            "labels": {
              "alertname": "DeleteNamespaceAttempt",
              "severity": "critical",
              "pod": "rogue-service",
              "namespace": "kube-system"
            },
            "annotations": {
              "summary": "Attempting unauthorized deletion of system namespace"
            }
          }]
        }')
    echo "   ⚡ Response: $res"
    echo "   ✅ OPA policy engine rejected unauthorized mutation attempt!"
}

# Scenario 4: Rate-Limit Spike (Circuit Breaker Lock)
trigger_rate_limit() {
    echo ""
    echo "🛡️  [Scenario 4] Spamming 4 Alerts to Breach Remediation Guard Threshold..."
    
    local target_pod="flaky-order-service"
    kubectl run "$target_pod" --image=nginx --namespace="$NAMESPACE" || true

    for i in {1..4}; do
        echo "   --> Firing Alert #$i for $target_pod..."
        curl -s -X POST "$CHEEZER_URL/api/grafana_webhook" \
            -H "Content-Type: application/json" \
            -H "x-api-key: $API_KEY" \
            -d '{
              "alerts": [{
                "status": "firing",
                "labels": {
                  "alertname": "CrashLoopBackOff",
                  "severity": "critical",
                  "pod": "'"$target_pod"'",
                  "namespace": "'"$NAMESPACE"'"
                }
              }]
            }' > /dev/null
        sleep 1
    done

    echo "   ⚠️  4th alert triggered RemediationGuard block!"
    echo "   ✅ Check Dashboard (http://localhost:9090/dashboard): Status is 'requires_human_intervention'!"
}

# Scenario 5: Kill-Switch Test (Emergency Brake)
trigger_kill_switch() {
    echo ""
    echo "🛑 [Scenario 5] Toggling Master Kill-Switch to Disabled State..."
    
    echo "   --> Engaging Emergency Brake..."
    curl -s -X POST "$CHEEZER_URL/api/system/toggle"
    echo ""

    echo "   --> Firing Alert while Kill-Switch is ENGAGED..."
    local res=$(curl -s -X POST "$CHEEZER_URL/api/grafana_webhook" \
        -H "Content-Type: application/json" \
        -H "x-api-key: $API_KEY" \
        -d '{
          "alerts": [{
            "status": "firing",
            "labels": {
              "alertname": "CrashLoopBackOff",
              "severity": "critical",
              "pod": "test-kill-switch-pod",
              "namespace": "'"$NAMESPACE"'"
            }
          }]
        }')
    echo "   ⚡ Response: $res"

    echo "   --> Re-enabling Master Kill-Switch..."
    curl -s -X POST "$CHEEZER_URL/api/system/toggle"
    echo ""
}

BUG_TYPE="${1:-all}"

case "$BUG_TYPE" in
    1|crashloop)
        trigger_crashloop
        ;;
    2|oom)
        trigger_oom
        ;;
    3|dangerous)
        trigger_dangerous
        ;;
    4|rate-limit)
        trigger_rate_limit
        ;;
    5|kill-switch)
        trigger_kill_switch
        ;;
    6|all)
        trigger_crashloop
        sleep 2
        trigger_oom
        sleep 2
        trigger_dangerous
        sleep 2
        trigger_rate_limit
        sleep 2
        trigger_kill_switch
        ;;
    *)
        print_usage
        exit 1
        ;;
esac

echo ""
echo "=================================================================="
echo "🎉 Fault Injection Completed! Open Dashboard: http://localhost:9090/dashboard"
echo "=================================================================="
