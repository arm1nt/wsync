use std::fmt::Display;
use serde::{Deserialize, Serialize};
use crate::{WorkspaceInfo, WorkspaceOverview};
use crate::response::ResponseStatus::{Error, NotFound, Success};

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceInfoResponse {
    #[serde(flatten)]
    pub info: WorkspaceInfo
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListWorkspacesResponse {
    pub nr_of_workspaces: usize,
    pub entries: Vec<WorkspaceOverview>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListWorkspaceInfoResponse {
    pub nr_of_workspaces: usize,
    pub entries: Vec<WorkspaceInfo>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseStatus {
    Success,
    NotFound,
    Error
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum ResponsePayload {
    WorkspaceInfo(WorkspaceInfoResponse),
    ListWorkspaces(ListWorkspacesResponse),
    ListWorkspaceInfo(ListWorkspaceInfoResponse),
    AddWorkspace(String),
    RemoveWorkspace(String),
    AttachRemoteWorkspace(String),
    DetachRemoteWorkspace(String)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ErrorPayload {
    Message(String)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<T: Serialize + Display, E: Serialize + Display> {
    pub status: ResponseStatus,
    pub result: Option<T>,
    pub error: Option<E>
}
pub type DefaultResponse = Response<ResponsePayload, ErrorPayload>;

impl<T: Serialize + Display, E: Serialize + Display> Response<T, E> {
    pub fn success(res: Option<T>) -> Response<T, E> {
        Response { status: Success, result: res, error: None }
    }

    pub fn not_found(msg: Option<E>) -> Response<T, E> {
        Response { status: NotFound, result: None, error: msg }
    }

    pub fn error(err: Option<E>) -> Response<T, E> {
        Response { status: Error, result: None, error: err }
    }
}
