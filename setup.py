import os
import argparse
import sys
import subprocess
import shutil

BUILD_MODE = "release"
BUILD_CLEANUP = False
BUILD_QUIET = False
BUILD_TARGET_DIR = "./build"

HOME = os.environ["HOME"]
WSYNC_LOG_DIRECTORY = os.path.join(HOME, ".wsync", "log")
WSYNC_DAEMON_CMD_SOCKET_DIRECTORY = "/tmp"
WSYNC_DAEMON_CMD_SOCKET_NAME = "wsync-daemon-cmd.socket"
WSYNC_WORKSPACE_CONFIGURATION_DIRECTORY = os.path.join(HOME, ".wsync")
WSYNC_WORKSPACE_CONFIGURATION_FILE_NAME = "wsync-ws-config.json"

def validate_directory(dir):
    if not os.path.exists(dir):
        print(f"[ERROR] Directory '{dir}' does not exist!")
        sys.exit(1)

    if not os.path.isdir(dir):
        print(f"[ERROR] '{dir}' is not a directory!")
        sys.exit(1)

def validate_file(file):
    if not os.path.exists(file):
        print(f"[ERROR] File '{file}' does not exist!")
        sys.exit(1)

    if not os.path.isfile(file):
        print(f"[ERROR] '{file}' is not a file!")
        sys.exit(1)

def check_requirements():
    tools = ["cargo", "rsync"]

    for tool in tools:
        if shutil.which(tool) is None:
            print(f"[ERROR] '{tool}' cannot be found!")


def build_arg_parser():
    parser = argparse.ArgumentParser()

    parser.add_argument("-q", "--build-quiet",
        action="store_true", dest="BUILD_QUIET", default=BUILD_QUIET,
        help="Enable quiet build")

    parser.add_argument("-c", "--build-clean",
        action="store_true", dest="BUILD_CLEANUP", default=BUILD_CLEANUP,
        help="Clean up old build artefacts before building")

    parser.add_argument("-m", "--build-mode",
        choices={"debug", "release"}, dest="BUILD_MODE", default=BUILD_MODE,
        help="Build mode")

    parser.add_argument("--workspace-configuration-directory",
        dest="WSYNC_WORKSPACE_CONFIGURATION_DIRECTORY",
        default=WSYNC_WORKSPACE_CONFIGURATION_DIRECTORY,
        help="Directory in which the files holding information about the configured workspaces are stored")

    parser.add_argument("--log-directory",
        dest="WSYNC_LOG_DIRECTORY",
        default=WSYNC_LOG_DIRECTORY,
        help="Directory in wich the wsync log files are stored")

    parser.add_argument("--daemon-cmd-socket-directory",
        dest="WSYNC_DAEMON_CMD_SOCKET_DIRECTORY",
        default=WSYNC_DAEMON_CMD_SOCKET_DIRECTORY,
        help="Directory in which the daemon command socket is created")

    return parser

def parse_args():
    args = build_arg_parser().parse_args()
    globals().update(vars(args))

def prepare_wsync_exec_environment():

    try:
        os.makedirs(os.path.join(HOME, ".wsync"), exist_ok=True)

        os.makedirs(WSYNC_LOG_DIRECTORY, exist_ok=True)

        os.makedirs(WSYNC_DAEMON_CMD_SOCKET_DIRECTORY, exist_ok=True)

        os.makedirs(WSYNC_WORKSPACE_CONFIGURATION_DIRECTORY, exist_ok=True)

        # Ensure that the wsync workspace configuration file exists and is initialized
        wsync_config_file = os.path.join(WSYNC_WORKSPACE_CONFIGURATION_DIRECTORY, WSYNC_WORKSPACE_CONFIGURATION_FILE_NAME)
        if not os.path.exists(wsync_config_file):
            # Inintialize the configuration as empty JSON array as it contains no workspaces to be managed yet
            with open(wsync_config_file, "w") as f:
                f.write("[]\n")

    except Exception as e:
        print(f"[ERROR] Failed to prepare wsync environment: {e}")
        sys.exit(1)

def build_crate(crate_directory):
    manifest_path = os.path.join(crate_directory, "Cargo.toml")
    validate_file(manifest_path)

    cmd = ["cargo", "build", "--manifest-path", manifest_path, "--target-dir", BUILD_TARGET_DIR]

    if BUILD_MODE == "release":
        cmd.append("--release")

    if BUILD_CLEANUP:
        cmd.append("--quiet")

    print(">> " + " ".join(cmd))
    subprocess.run(cmd)

def build_crates():
    crates = [
        "./lib/daemon-client",
        "./lib/daemon-interface",
        "./lib/wsync-config",
        "./src/client",
        "./src/monitor",
        "./src/daemon"
    ]

    if BUILD_CLEANUP:
        #cmd = ["cargo", "clean", "--target-dir", BUILD_TARGET_DIR]
        cmd = ["rm", "-rf", BUILD_TARGET_DIR]
        print(">> " + " ".join(cmd))
        subprocess.run(cmd)

    for crate_directory in crates:
        build_crate(crate_directory)

def get_monitor_executable():
    project_path = os.environ['PWD']
    monitor_executable = os.path.join(project_path, BUILD_TARGET_DIR, BUILD_MODE, "monitor")
    validate_file(monitor_executable)
    return monitor_executable

def get_executables_dir_path():
    project_path = os.environ['PWD']
    executables_dir = os.path.join(project_path, BUILD_TARGET_DIR, BUILD_MODE)
    validate_directory(executables_dir)
    return executables_dir

def init_wsync_config():
    wsync_config_file = os.path.join(HOME, ".wsync", "wsync.config")

    with open(wsync_config_file, "w") as f:
        f.write("WorkspaceConfigFilePath={}\n".format(os.path.join(
            WSYNC_WORKSPACE_CONFIGURATION_DIRECTORY,
            WSYNC_WORKSPACE_CONFIGURATION_FILE_NAME
        )))

        f.write("MonitorExecutablePath={}\n".format(get_monitor_executable()))

        f.write("DaemonCommandSocketPath={}\n".format(os.path.join(
            WSYNC_DAEMON_CMD_SOCKET_DIRECTORY,
            WSYNC_DAEMON_CMD_SOCKET_NAME
        )))

        f.write("LogDirectory={}\n".format(WSYNC_LOG_DIRECTORY))

def main():
    check_requirements()

    parse_args()

    prepare_wsync_exec_environment()

    build_crates()

    init_wsync_config()

    executables_directory = get_executables_dir_path()
    print("\n\n========================================================")
    print("For ease of use, add the directory with the produced executabsles to the PATH!")
    print(f"e.g. PATH=\"{executables_directory}:$PATH\"")


if __name__ == "__main__":
    main()
