use log::{debug, error, info, warn};
use std::io::Write;
use std::process::{Child, Command, Stdio};

use crate::config::Config;

pub fn spawn_overlay(
    #[cfg(target_os = "windows")] job: &Option<crate::platform::JobObject>,
) -> Option<Child> {
    let exe = std::env::current_exe().expect("cannot find own executable");
    let mut cmd = Command::new(exe);
    cmd.arg("overlay").stdin(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    match cmd.spawn() {
        Ok(child) => {
            info!("spawned overlay process (pid: {})", child.id());
            #[cfg(target_os = "windows")]
            if let Some(job) = job {
                job.assign(&child);
            }
            Some(child)
        }
        Err(e) => {
            error!("failed to spawn overlay: {e}");
            None
        }
    }
}

pub fn send_config(child: &mut Option<Child>, config: &Config, prev_config: &mut Option<String>) {
    let json = match serde_json::to_string(config) {
        Ok(j) => j,
        Err(_) => return,
    };

    if prev_config.as_deref() == Some(&json) {
        return;
    }
    debug!("config changed, sending to overlay");
    *prev_config = Some(json.clone());

    if let Some(child) = child
        && let Some(stdin) = &mut child.stdin
    {
        let line = format!("{json}\n");
        if stdin.write_all(line.as_bytes()).is_err() {
            warn!("overlay stdin write failed, process likely died");
        }
    }
}
