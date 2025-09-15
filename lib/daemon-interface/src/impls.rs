use std::fmt::{Display, Formatter, Write};
use serde::Serialize;
use crate::response::{ErrorPayload, Response, ResponsePayload, ResponseStatus};
use crate::{WorkspaceInfo, WorkspaceOverview};

impl<T: Display + Serialize, E: Display + Serialize> Display for Response<T, E> {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        let msg_prefix = match self.status {
            ResponseStatus::Error => Some("Error!"),
            _ => None
        };

        if let Some(prefix) = msg_prefix {
            f.write_str(prefix)?;
            f.write_char('\n')?;
        }

        let payload = match self.status {
            ResponseStatus::Success => self.result.as_ref().map(|v| format!("{v}")),
            _ => self.error.as_ref().map(|e| format!("{e}"))
        };

        if let Some(args) = payload {
            f.write_str(args.as_str())?;
            f.write_char('\n')?;
        }

        Ok(())
    }
}

impl Display for ResponsePayload {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        match self {
            ResponsePayload::WorkspaceInfo(payload) => {
                write!(f, "{}", payload.info)?;
            },
            ResponsePayload::ListWorkspaces(payload) => {
                for ws_entry in &payload.entries {
                    write!(f, "{ws_entry}\n")?;
                }

                write!(f, "\n-------------------------------------")?;
                write!(f, "\nTotal number of registered workspaces: {}\n", payload.nr_of_workspaces)?;
            },
            ResponsePayload::ListWorkspaceInfo(payload) => {
                let mut counter = 1;

                for ws_info_entry in &payload.entries {
                    write!(f, "\n{counter}) {}\n", ws_info_entry)?;
                    counter += 1;
                }

                write!(f, "\n-------------------------------------")?;
                write!(f, "\nTotal number of registered workspaces: {}\n", payload.nr_of_workspaces)?;
            },
            ResponsePayload::AddWorkspace(payload) => {
                write!(f, "{}\n", payload)?;
            },
            ResponsePayload::RemoveWorkspace(payload) => {
                write!(f, "{}\n", payload)?;
            },
            ResponsePayload::AttachRemoteWorkspace(payload) => {
                write!(f, "{}\n", payload)?;
            },
            ResponsePayload::DetachRemoteWorkspace(payload) => {
                write!(f, "{}\n", payload)?;
            }
        }

        Ok(())
    }

}

impl Display for ErrorPayload {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorPayload::Message(payload) => {
                write!(f, "{}\n", payload)?;
            }
        }
        Ok(())
    }

}

impl Display for WorkspaceOverview {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[\n\tName: {}\n\tLocal Path: {:?}\n\t#remote workspaces: {}\n]",
            self.name,
            self.path,
            self.nr_of_remote_workspaces
        )
    }
}

impl Display for WorkspaceInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        write!(
            f,
            "Workspace Name: {}, Path: {:?}, #Remote Workspaces: {}, Remote Workspaces: {:?}",
            self.name,
            self.path,
            self.nr_of_remote_workspaces,
            self.remote_workspaces
        )?;

        Ok(())
    }
}
