package cheezer.authz_test

import rego.v1

import data.cheezer.authz

test_restart_pod_allowed if {
    authz.allow with input as {"action": "restart", "resource": "pod", "target_replicas": 0, "command": []}
}

test_scale_within_cap_allowed if {
    authz.allow with input as {"action": "scale", "resource": "deployment", "target_replicas": 5, "command": []}
}

test_delete_namespace_denied if {
    not authz.allow with input as {"action": "delete", "resource": "namespace", "target_replicas": 0, "command": []}
}

test_exec_command_denied if {
    not authz.allow with input as {"action": "exec", "resource": "pod", "target_replicas": 0, "command": ["exec", "sh"]}
}

test_scale_above_cap_denied if {
    not authz.allow with input as {"action": "scale", "resource": "deployment", "target_replicas": 15, "command": []}
}

test_modify_rbac_denied if {
    not authz.allow with input as {"action": "modify", "resource": "rbac", "target_replicas": 0, "command": []}
}
