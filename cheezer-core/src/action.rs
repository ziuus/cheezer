use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    RestartPod {
        pod: String,
        namespace: String,
    },
    ScaleDeployment {
        deployment: String,
        target_replicas: u32,
        namespace: String,
    },
    CordonNode {
        node: String,
    },
    DeleteNamespace {
        namespace: String,
    },
    ExecCommand {
        pod: String,
        command: Vec<String>,
    },
    ModifyRbac {
        resource: String,
    },
    LogReviewNeeded {
        reason: String,
    },
    LogCheckCapacity {
        reason: String,
    },
    CreateGithubPR {
        file_path: String,
        new_content: String,
        pr_title: String,
        pr_body: String,
    },
    None,
}

impl Action {
    pub fn action_type(&self) -> &'static str {
        match self {
            Action::RestartPod { .. } => "restart",
            Action::ScaleDeployment { .. } => "scale",
            Action::CordonNode { .. } => "cordon",
            Action::DeleteNamespace { .. } => "delete",
            Action::ExecCommand { .. } => "exec",
            Action::ModifyRbac { .. } => "modify",
            Action::CreateGithubPR { .. } => "gitops",
            Action::LogReviewNeeded { .. } | Action::LogCheckCapacity { .. } | Action::None => "log",
        }
    }

    pub fn resource_type(&self) -> &'static str {
        match self {
            Action::RestartPod { .. } | Action::ExecCommand { .. } => "pod",
            Action::ScaleDeployment { .. } => "deployment",
            Action::CordonNode { .. } => "node",
            Action::DeleteNamespace { .. } => "namespace",
            Action::ModifyRbac { .. } => "rbac",
            Action::CreateGithubPR { .. } => "git",
            Action::LogReviewNeeded { .. } | Action::LogCheckCapacity { .. } | Action::None => "none",
        }
    }

    pub fn target_resource(&self) -> String {
        match self {
            Action::RestartPod { pod, .. } => pod.clone(),
            Action::ScaleDeployment { deployment, .. } => deployment.clone(),
            Action::CordonNode { node, .. } => node.clone(),
            Action::DeleteNamespace { namespace, .. } => namespace.clone(),
            Action::ExecCommand { pod, .. } => pod.clone(),
            Action::ModifyRbac { resource, .. } => resource.clone(),
            Action::CreateGithubPR { file_path, .. } => file_path.clone(),
            Action::LogReviewNeeded { .. } | Action::LogCheckCapacity { .. } | Action::None => "".to_string(),
        }
    }

    pub fn target_replicas(&self) -> u32 {
        match self {
            Action::ScaleDeployment { target_replicas, .. } => *target_replicas,
            _ => 0,
        }
    }

    pub fn commands(&self) -> Vec<&str> {
        match self {
            Action::ExecCommand { command, .. } => command.iter().map(|s| s.as_str()).collect(),
            _ => vec![],
        }
    }

    pub fn to_action_string(&self) -> String {
        match self {
            Action::RestartPod { pod, .. } => format!("restart pod {}", pod),
            Action::ScaleDeployment { deployment, target_replicas, .. } => format!("scale deployment {} {}", deployment, target_replicas),
            Action::CordonNode { node } => format!("cordon node {}", node),
            Action::DeleteNamespace { namespace } => format!("delete namespace {}", namespace),
            Action::ExecCommand { pod, .. } => format!("exec pod {}", pod),
            Action::ModifyRbac { resource } => format!("modify rbac {}", resource),
            Action::LogReviewNeeded { reason } => format!("log manual review needed: {}", reason),
            Action::LogCheckCapacity { reason } => format!("log check node capacity: {}", reason),
            Action::CreateGithubPR { file_path, pr_title, .. } => format!("create github pr for {} - {}", file_path, pr_title),
            Action::None => "none".to_string(),
        }
    }

    pub fn parse_from_string(s: &str) -> Self {
        let s = s.trim();
        if s.starts_with("restart pod ") {
            let pod = s.trim_start_matches("restart pod ").trim().to_string();
            Action::RestartPod {
                pod,
                namespace: "default".to_string(),
            }
        } else if s.starts_with("scale deployment ") {
            let parts: Vec<&str> = s.trim_start_matches("scale deployment ").split_whitespace().collect();
            if parts.len() >= 2 {
                let deployment = parts[0].to_string();
                let replicas = parts[1].parse::<u32>().unwrap_or(1);
                Action::ScaleDeployment {
                    deployment,
                    target_replicas: replicas,
                    namespace: "default".to_string(),
                }
            } else if !parts.is_empty() {
                Action::ScaleDeployment {
                    deployment: parts[0].to_string(),
                    target_replicas: 1,
                    namespace: "default".to_string(),
                }
            } else {
                Action::None
            }
        } else if s.starts_with("cordon node ") {
            let node = s.trim_start_matches("cordon node ").trim().to_string();
            Action::CordonNode { node }
        } else if s.starts_with("delete namespace ") {
            let namespace = s.trim_start_matches("delete namespace ").trim().to_string();
            Action::DeleteNamespace { namespace }
        } else if s.starts_with("modify rbac ") {
            let resource = s.trim_start_matches("modify rbac ").trim().to_string();
            Action::ModifyRbac { resource }
        } else if s.starts_with("exec pod ") {
            let pod = s.trim_start_matches("exec pod ").trim().to_string();
            Action::ExecCommand {
                pod,
                command: vec!["exec".to_string()],
            }
        } else if s.starts_with("log manual review needed:") {
            let reason = s.trim_start_matches("log manual review needed:").trim().to_string();
            Action::LogReviewNeeded { reason }
        } else if s.starts_with("log check node capacity:") {
            let reason = s.trim_start_matches("log check node capacity:").trim().to_string();
            Action::LogCheckCapacity { reason }
        } else {
            Action::None
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_action_string())
    }
}
