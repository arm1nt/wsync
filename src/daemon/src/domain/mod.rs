use serde::{Deserialize, Serialize};
use daemon_interface::WorkspaceInfo;
use daemon_interface::request::{AddWorkspaceRequest, AttachRemoteWorkspaceRequest};

pub(crate) mod errors;
pub(crate) mod socket;
pub(crate) mod models;
