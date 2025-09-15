use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex, TryLockError};
use std::thread::sleep;
use std::time::Duration;
use log::{debug, error, info, warn};
use crate::daemon_state::DaemonState;
use crate::util::constants::SERVER_SOCKET_PATH_ENV_VAR;
use crate::util::error_exit;

const DEFAULT_WATCHDOG_INTERVAL_SECONDS: Duration = Duration::from_secs(60);
const MAX_MONITOR_FAILURES: usize = 3;

struct WatchdogState {
    monitor_failure_map: HashMap<String, usize>
}

pub(crate) fn watchdog(state: Arc<Mutex<DaemonState>>) {
    info!("[WATCHDOG] Starting wsync daemon watchdog...");

    let mut watchdog_state = WatchdogState { monitor_failure_map: HashMap::new() };
    let mut timeout: Duration = DEFAULT_WATCHDOG_INTERVAL_SECONDS;

    loop {
        sleep(timeout);

        // Reset timeout to default value in case it was previously modified
        timeout = DEFAULT_WATCHDOG_INTERVAL_SECONDS;

        let mut guard = match state.try_lock() {
            Ok(guard) => guard,
            Err(err) => {
                debug!("[WATCHDOG] Failed to obtain daemon state lock");

                match err {
                    TryLockError::WouldBlock => {
                        debug!("[WATCHDOG] Daemon state lock is currently held by another thread");
                        timeout = Duration::from_secs(30);
                        continue;
                    },
                    TryLockError::Poisoned(e) => {
                        error!("[WATCHDOG] Daemon state lock is poisoned: {e}");
                        terminate();
                    }
                }
            }
        };

        let mut monitors_to_restart: Vec<String> = vec![];

        for (workspace_id, monitor) in guard.monitor_manager.ws_id_to_monitor.iter_mut() {

            match monitor.try_wait() {
                Ok(Some(status)) => {

                    let count = watchdog_state.monitor_failure_map
                        .entry(workspace_id.clone())
                        .or_insert(0);
                    *count += 1;

                    if *count >= MAX_MONITOR_FAILURES {

                        if *count == MAX_MONITOR_FAILURES {
                            error!(
                                "[WATCHDOG] Monitor for '{}' crashed {} times (threshold reached). \
                                No more restart attempts will be made!",
                                workspace_id,
                                MAX_MONITOR_FAILURES
                            );
                        }

                        continue;
                    }

                    warn!(
                        "[WATCHDOG] Monitor process of '{}' stopped with status '{}'. Attempting \
                        to restart it...",
                        workspace_id,
                        status
                    );
                    monitors_to_restart.push(workspace_id.clone());
                },
                Err(e) => {
                    warn!("[WATCHDOG] Failed to query status of {workspace_id}'s monitor process: {e}");
                },
                _ => {}
            }
        }

        for ws in monitors_to_restart {

            let workspace = match guard.ws_config.find_by_name(&ws) {
                Some(ws_info) => ws_info,
                None => {
                    error!("[WATCHDOG] Cannot restart monitor for '{}' as this workspace does not exist.", ws);
                    continue;
                }
            };

            match guard.monitor_manager.restart_monitor(&workspace) {
                Ok(()) => {
                    debug!("[WATCHDOG] Restarted monitor for '{}'", ws);
                },
                Err(e) => {
                    error!("[WATCHDOG] Failed to restart monitor for '{}': {e}", ws);
                }
            }
        }

        drop(guard);
    }
}

fn terminate() -> ! {
    if let Ok(val) = env::var(SERVER_SOCKET_PATH_ENV_VAR) {
        let _ = std::fs::remove_file(val);
    }
    error_exit::<String>(None);
}
