#!/bin/bash
set -e

echo "Setting up k3s demo environment..."
# In a real environment, you'd curl the k3s installer:
# curl -sfL https://get.k3s.io | sh -

# We'll just print instructions for the hackathon context
echo "1. Run k3s (or use k3d): k3d cluster create cheezer-demo"
echo "2. Install Prometheus Stack:"
echo "   helm repo add prometheus-community https://prometheus-community.github.io/helm-charts"
echo "   helm repo update"
echo "   helm install prometheus prometheus-community/kube-prometheus-stack"
echo "3. Configure Alertmanager to send webhooks to Cheezer (http://<cheezer-ip>:9090/api/grafana_webhook)"
echo "Done."
