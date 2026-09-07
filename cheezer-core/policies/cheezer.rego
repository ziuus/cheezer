package cheezer.authz

import rego.v1

default allow := false

deny contains msg if {
    input.action == "delete"
    input.resource == "namespace"
    msg := "CRITICAL: Namespace deletion is absolutely prohibited during autonomous execution."
}

deny contains msg if {
    input.command[_] == "exec"
    msg := "CRITICAL: Container shell execution is blocked."
}

deny contains msg if {
    input.action == "scale"
    input.target_replicas > 10
    msg := "CRITICAL: Replica cap exceeded."
}

deny contains msg if {
    input.action == "modify"
    input.resource == "rbac"
    msg := "CRITICAL: RBAC modification is blocked."
}

allow if {
    count(deny) == 0
}
