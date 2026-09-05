#!/bin/bash

echo "Cheezer Demo Runbook"
echo "====================="
echo "1. Show cluster is healthy"
echo "2. Deploy a broken pod (e.g., OOMKilled)"
echo "3. Watch Cheezer Primary receive the alert and fix it (Rule Path)"
echo "4. Deploy an unknown anomaly"
echo "5. Watch Cheezer Primary escalate to LLM, consult OPA, and execute the fix"
echo "6. Kill Cheezer Primary process (pkill -f 'cheezer --role=primary')"
echo "7. Watch Cheezer Backup take over"
echo "8. Deploy another broken pod to prove Backup works"
