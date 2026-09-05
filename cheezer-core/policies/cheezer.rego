package cheezer.authz

default allow = false

deny[msg] {
    input.action == "delete"
    input.resource == "namespace"
    msg := "CRITICAL: Namespace deletion is absolutely prohibited during autonomous execution."
}

deny[msg] {
    input.command[_] == "exec"
    msg := "CRITICAL: Container shell execution is blocked."
}

deny[msg] {
    input.action == "scale"
    input.target_replicas > 10
    msg := "CRITICAL: Replica cap exceeded."
}

deny[msg] {
    input.action == "modify"
    input.resource == "rbac"
    msg := "CRITICAL: RBAC modification is blocked."
}

allow {
    count(deny) == 0
}
