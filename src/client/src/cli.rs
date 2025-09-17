use std::net::IpAddr;
use std::path::PathBuf;
use std::process;
use clap::{Args, Parser, Subcommand};

pub(self) type Result<T> = std::result::Result<T, Error>;

pub(self) struct Error {
    msg: String
}

impl Error {
    pub(self) fn new (msg: String) -> Self {
        Error { msg }
    }
}

#[derive(Parser)]
#[command(name = "wsync-client")]
#[command(version = env!["CARGO_PKG_VERSION"])]
#[command(author = "arm1nt")]
#[command(about = "wsync client utility", long_about = None)]
#[command(next_line_help = true)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command
}

#[derive(Subcommand)]
pub(crate) enum Command {
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
pub(crate) struct NoArgs {}

#[derive(Args)]
pub(crate) struct WorkspaceInfoArgs {
    /// Name of the local workspace whose information should be displayed
    #[arg(short, long)]
    pub(crate) name: String
}

#[derive(Args)]
pub(crate) struct AddWorkspaceArgs {
    /// Name of the workspace to be added. This name must be unique among all managed local workspaces
    /// and will be used to identify and reference it in other commands (e.g. when attaching a
    /// remote workspace, etc.)
    #[arg(short, long)]
    pub(crate) name: String,

    /// Absolute path to the local workspace
    #[arg(short, long)]
    pub(crate) path: PathBuf
}

#[derive(Args)]
pub(crate) struct RemoveWorkspaceArgs {
    /// Name of the local workspace to be removed. The workspace will no longer be managed by wsync
    #[arg(short, long)]
    pub(crate) name: String
}

#[derive(Args)]
pub(crate) struct AttachRemoteWorkspaceCommand {
    #[command(subcommand)]
    pub(crate) command: AttachRemoteWorkspaceSubcommands
}

#[derive(Subcommand)]
pub(crate) enum AttachRemoteWorkspaceSubcommands {
    /// Attach a remote workspace to a local workspace managed by wsync.
    /// wsync will connect to the remote system via SSH
    Ssh(SshArgs),
    /// Attach a remote workspace to a local workspace managed by wsync.
    /// wsync will connect to the remote system via the rsync daemon
    Rsync(RsyncArgs)
}

#[derive(Args, Debug)]
pub(crate) struct AttachRemoteWorkspaceArgs {
    /// Name of the local workspace to which the remote workspace should be attached.
    #[arg(short, long)]
    pub(crate) workspace_name: String,

    /// Name of the remote workspace to be attached. The name must be unique among all remote
    /// workspaces attached to the local workspace and will be used to reference it in other
    /// commands (e.g. detaching it, etc.)
    #[arg(short, long)]
    pub(crate) remote_workspace_name: String,

    /// Absolute path to the remote workspace on the remote system
    #[arg(short = 'p', long)]
    pub(crate) remote_path: PathBuf
}

#[derive(Args)]
#[group(required = false, multiple = false)]
pub(crate) struct HostInfo {
    /// IP address of the remote system
    #[arg(long)]
    pub(crate) ip_addr: Option<IpAddr>,

    /// Domain name of the remote system
    #[arg(long)]
    pub(crate) hostname: Option<String>
}

#[derive(Args)]
pub(crate) struct SshArgs {
    #[command(flatten)]
    pub(crate) args: AttachRemoteWorkspaceArgs,

    /* Connect by manually specifying SSH related information */
    #[command(flatten)]
    pub(crate) host_info: HostInfo,

    /// Port to be used when establishing an SSH connection
    #[arg(long, default_value_t = 22)]
    pub(crate) port: u16,

    /// Username to be used when establishing an SSH connection
    #[arg(long)]
    pub(crate) user: Option<String>,

    /// Path to an SSH identity file holding the key needed to authenticate with the remote system.
    #[arg(long)]
    pub(crate) identity_file: Option<PathBuf>,

    /* Simply specify ssh host alias containing all the relevant information */
    /// Alias specified in the SSH config file that defines all the information required to establish
    /// an SSH connection to the remote system.
    #[arg(long, conflicts_with_all = vec!["hostname", "ip_addr", "port", "user", "identity_file"])]
    pub(crate) host_alias: Option<String>
}

#[derive(Args)]
pub(crate) struct RsyncArgs {
    #[command(flatten)]
    pub(crate) args: AttachRemoteWorkspaceArgs,

    #[command(flatten)]
    pub(crate) host_info: HostInfo,

    /// Port to be used when establishing a connection
    #[arg(long, default_value_t = 873)]
    pub(crate) port: u16,

    /// Username to be used when establishing a connection
    #[arg(long)]
    pub(crate) user: Option<String>
}

#[derive(Args)]
pub(crate) struct DetachRemoteWorkspaceArgs {
    /// Name of the local workspace from which the remote workspace should be detached.
    #[arg(short, long)]
    pub(crate) workspace_name: String,

    /// Name of the remote workspace to be detached. Changes will no longer be propagated to this
    /// remote workspace
    #[arg(short, long)]
    pub(crate) remote_workspace_name: String,
}

pub(self) fn validate_ssh_connection_args(args: &SshArgs) -> Result<()> {

    if args.host_alias.is_some() {
        return Ok(());
    }

    // In case no host alias is specified, the following restriction applies
    if args.host_info.hostname.is_none() && args.host_info.ip_addr.is_none() {
        return Err(Error::new(
            "Either the remote system's domain name or its IP address must be provided!".to_string()
        ));
    }

    Ok(())
}

pub(self) fn validate_rsync_connection_args(args: &RsyncArgs) -> Result<()> {

    if args.host_info.hostname.is_none() && args.host_info.ip_addr.is_none() {
        return Err(Error::new(
            "Either the remote system's domain name or its IP address must be provided!".to_string()
        ));
    }

    Ok(())
}

pub(self) fn validate_attach_remote_ws_subcommand(sub_cmd: &AttachRemoteWorkspaceCommand) -> Result<()> {
    match &sub_cmd.command {
        AttachRemoteWorkspaceSubcommands::Ssh(args) => {
            validate_ssh_connection_args(args)?;
        },
        AttachRemoteWorkspaceSubcommands::Rsync(args) => {
            validate_rsync_connection_args(args)?;
        }
    }
    Ok(())
}

pub(crate) fn parse_cli_arguments() -> Cli {
    let cli: Cli = Cli::parse();

    // Perform additional validation that cannot be expressed with clap
    match &cli.command {
        Command::AttachRemoteWorkspace(sub_cmd) => {
            if let Err(e) = validate_attach_remote_ws_subcommand(sub_cmd) {
                eprintln!("[INPUT VALIDATION ERROR] {}", e.msg);
                process::exit(1);
            }
        },
        _ => {}
    }

    cli
}
