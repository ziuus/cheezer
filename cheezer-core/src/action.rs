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
            Action::None => "none".to_string(),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_action_string())
    }
}
