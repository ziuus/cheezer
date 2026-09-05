# Build & Run Guide

## Prerequisites
- Rust and Cargo
- OPA (`opa run --server`)
- A local k3s cluster (see `infra/k3s-setup.sh`)
- `kube-prometheus-stack` deployed

## Running Cheezer

1. **Start OPA:**
   ```bash
   opa run --server cheezer-core/policies/cheezer.rego
   ```

2. **Start Primary:**
   ```bash
   cd cheezer-core
   cargo run -- --role=primary
   ```

3. **Start Backup:**
   ```bash
   cd cheezer-core
   cargo run -- --role=backup --peer=127.0.0.1
   ```
