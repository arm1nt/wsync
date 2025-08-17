use std::net::IpAddr;
use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wsync-client")]
#[command(version = env!["CARGO_PKG_VERSION"])]
#[command(author = "arm1nt")]
#[command(about = "wsync client utility", long_about = None)]
#[command(next_line_help = true)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command
}

#[derive(Subcommand)]
pub enum Command {
    /// Get detailed information about a specific workspace, e.g. name, path, remote workspaces, etc.
    WorkspaceInfo(WorkspaceInfoArgs),
    /// Get overview information about all managed local workspaces
    ListWorkspaces(NoArgs),
    /// Get detailed information about all managed local workspaces
    ListWorkspaceInfo(NoArgs),
    /// Add a local workspace to be managed by wsync
    AddWorkspace(AddWorkspaceArgs),
    /// Remove a local workspace from being managed by wsync. Afterwards, changes in the workspace
    /// will no longer be propagated to remote workspaces
    RemoveWorkspace(RemoveWorkspaceArgs),
    /// Attach a remote workspace to a local workspace managed by wsync. Afterwards, all changes in
    /// the local workspace will be propagated to the remote workspace
    AttachRemoteWorkspace(AttachRemoteWorkspaceCommand),
    /// Detach a remote workspace from a local workspace managed by wsync. Afterwards, changes in
    /// the local workspace will no longer be propagated to the remote workspace
    DetachRemoteWorkspace(DetachRemoteWorkspaceArgs)
}

#[derive(Args)]
pub struct NoArgs {}

#[derive(Args)]
pub struct WorkspaceInfoArgs {
    /// Name of the local workspace whose information should be displayed
    #[arg(short, long)]
    pub name: String
}

#[derive(Args)]
pub struct AddWorkspaceArgs {
    /// Name of the workspace to be added. This name must be unique among all managed local workspaces
    /// and will be used to identify and reference it in other commands (e.g. when attaching a
    /// remote workspace, etc.)
    #[arg(short, long)]
    pub name: String,

    /// Absolute path to the local workspace
    #[arg(short, long)]
    pub path: PathBuf
}

#[derive(Args)]
pub struct RemoveWorkspaceArgs {
    /// Name of the local workspace to be removed. The workspace will no longer be managed by wsync
    #[arg(short, long)]
    pub name: String
}

#[derive(Args)]
pub struct AttachRemoteWorkspaceCommand {
    #[command(subcommand)]
    pub command: AttachRemoteWorkspaceSubcommands
}

#[derive(Subcommand)]
pub enum AttachRemoteWorkspaceSubcommands {
    /// Attach a remote workspace to a local workspace managed by wsync.
    /// wsync will connect to the remote system via SSH
    Ssh(SshArgs),
    /// Attach a remote workspace to a local workspace managed by wsync.
    /// wsync will connect to the remote system via the rsync daemon
    Rsync(RsyncArgs)
}

#[derive(Args, Debug)]
pub struct AttachRemoteWorkspaceArgs {
    /// Name of the local workspace to which the remote workspace should be attached.
    #[arg(short, long)]
    pub workspace_name: String,

    /// Name of the remote workspace to be attached. The name must be unique among all remote
    /// workspaces attached to the local workspace and will be used to reference it in other
    /// commands (e.g. detaching it, etc.)
    #[arg(short, long)]
    pub remote_workspace_name: String,

    /// Absolute path to the remote workspace on the remote system
    #[arg(short = 'p', long)]
    pub remote_path: PathBuf
}

#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct HostInfo {
    /// IP address of the remote system
    #[arg(long)]
    pub ip_addr: Option<IpAddr>,

    /// Domain name of the remote system
    #[arg(long)]
    pub  hostname: Option<String>
}

#[derive(Args)]
pub struct SshArgs {
    #[command(flatten)]
    pub args: AttachRemoteWorkspaceArgs,

    /* Connect by manually specifying SSH related information */
    #[command(flatten)]
    pub host_info: HostInfo,

    /// Port to be used when establishing an SSH connection
    #[arg(long, default_value_t = 22)]
    pub port: u16,

    /// Username to be used when establishing an SSH connection
    #[arg(long)]
    pub user: Option<String>,

    /// Path to an SSH identity file holding the key needed to authenticate with the remote system.
    #[arg(long)]
    pub identity_file: Option<PathBuf>,

    /* Simply specify ssh host alias containing all the relevant information */
    /// Alias specified in the SSH config file that defines all the information required to establish
    /// an SSH connection to the remote system.
    #[arg(long, conflicts_with_all = vec!["hostname", "ip_addr", "port", "user", "identity_file"])]
    pub host_alias: Option<String>
}

#[derive(Args)]
pub struct RsyncArgs {
    #[command(flatten)]
    pub args: AttachRemoteWorkspaceArgs,

    #[command(flatten)]
    pub host_info: HostInfo,

    /// Port to be used when establishing a connection
    #[arg(long, default_value_t = 873)]
    pub port: u16,

    /// Username to be used when establishing a connection
    #[arg(long)]
    pub user: Option<String>
}

#[derive(Args)]
pub struct DetachRemoteWorkspaceArgs {
    /// Name of the local workspace from which the remote workspace should be detached.
    #[arg(short, long)]
    pub workspace_name: String,

    /// Name of the remote workspace to be detached. Changes will no longer be propagated to this
    /// remote workspace
    #[arg(short, long)]
    pub remote_workspace_name: String,
}
