use std::collections::HashMap;
use std::sync::{Arc, Mutex, TryLockError};
use std::thread::{current, sleep};
use std::time::Duration;
use log::{debug, error, info, warn};
use crate::daemon_state::DaemonState;
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

        // Reset timeout to default value
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
                        error_exit::<String>(None);
                    }
                }
            }
        };

        for (workspace_id, mut monitor) in guard.monitor_manager.map.iter_mut() {

            match monitor.try_wait() {
                Ok(Some(status)) => {
                    warn!("[WATCHDOG] Monitor process of '{workspace_id}' exited with status '{status}'");

                    let count = watchdog_state.monitor_failure_map
                        .entry(workspace_id.clone())
                        .or_insert(0);
                    *count += 1;

                    if *count >= MAX_MONITOR_FAILURES {
                        error!("[WATCHDOG] Monitor for '{}' crashed {} times (threshold reached)", workspace_id, MAX_MONITOR_FAILURES);
                        error_exit::<String>(None);
                    }
                },
                Ok(None) => {},
                Err(e) => {
                    warn!("[WATCHDOG] Failed to query status of {workspace_id}'s monitor process: {e}");
                }
            }
        }
        drop(guard);
    }
}
