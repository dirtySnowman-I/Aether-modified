use std::fs;
use std::path::{Path, PathBuf};

fn pid_file(data_dir: &Path) -> PathBuf {
    data_dir.join("aether.pid")
}

pub fn write_pid(data_dir: &Path, pid: u32) {
    let _ = fs::write(pid_file(data_dir), pid.to_string());
}

pub fn clear_pid(data_dir: &Path) {
    let _ = fs::remove_file(pid_file(data_dir));
}

/// On startup, if a pid file survives from a prior crash and that process is
/// still a running Aether core, kill it before the user can click Connect —
/// otherwise a leftover Aether would just fail to bind the SOCKS port for the
/// new one. This is a defensive backstop; `connect()`'s own port-in-use check
/// (see aether/mod.rs) covers the case where this file is missing or stale.
///
/// The process name is checked, not just the PID: after a reboot the old PID
/// can belong to an unrelated program, which must never be force-killed.
pub fn reap_orphan(data_dir: &Path) {
    let path = pid_file(data_dir);
    let Ok(contents) = fs::read_to_string(&path) else {
        return;
    };
    if let Ok(pid) = contents.trim().parse::<u32>() {
        if pid != 0 && is_aether(pid) {
            kill_pid(pid);
        }
    }
    let _ = fs::remove_file(&path);
}

/// `ps -o comm=` prints the executable name (possibly with a path).
#[cfg(unix)]
fn is_aether_comm(output: &str) -> bool {
    output
        .trim()
        .rsplit('/')
        .next()
        .is_some_and(|name| name == "aether")
}

#[cfg(unix)]
fn is_aether(pid: u32) -> bool {
    std::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()
        .map(|o| o.status.success() && is_aether_comm(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or(false)
}

#[cfg(unix)]
fn kill_pid(pid: u32) {
    let _ = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status();
}

/// tasklist /FO CSV /NH rows look like `"aether.exe","1234","Console",...`.
#[cfg(windows)]
fn is_aether_row(output: &str, pid: u32) -> bool {
    let needle = format!("\"aether.exe\",\"{pid}\"");
    output
        .lines()
        .any(|line| line.trim_start().to_ascii_lowercase().starts_with(&needle))
}

/// Windows GUI apps flash a console window for every child console program
/// unless it is created without one.
#[cfg(windows)]
fn hidden(program: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut command = std::process::Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(windows)]
fn is_aether(pid: u32) -> bool {
    hidden("tasklist")
        .args([
            "/FI",
            &format!("PID eq {pid}"),
            "/FI",
            "IMAGENAME eq aether.exe",
            "/FO",
            "CSV",
            "/NH",
        ])
        .output()
        .map(|o| is_aether_row(&String::from_utf8_lossy(&o.stdout), pid))
        .unwrap_or(false)
}

#[cfg(windows)]
fn kill_pid(pid: u32) {
    let _ = hidden("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn only_an_aether_row_with_the_same_pid_matches() {
        let row = "\"aether.exe\",\"1234\",\"Console\",\"1\",\"12,345 K\"\r\n";
        assert!(is_aether_row(row, 1234));
        assert!(!is_aether_row(row, 123));
        assert!(!is_aether_row(
            "\"notepad.exe\",\"1234\",\"Console\",\"1\",\"1,234 K\"",
            1234
        ));
        assert!(!is_aether_row(
            "INFO: No tasks are running which match the specified criteria.",
            1234
        ));
    }

    #[cfg(windows)]
    #[test]
    fn a_reused_pid_of_another_program_is_not_treated_as_aether() {
        // This test process is not aether.exe.
        assert!(!is_aether(std::process::id()));
    }

    #[cfg(unix)]
    #[test]
    fn only_the_aether_executable_matches() {
        assert!(is_aether_comm("aether\n"));
        assert!(is_aether_comm("/opt/app/binaries/aether"));
        assert!(!is_aether_comm("aether-gui"));
        assert!(!is_aether_comm("bash"));
        assert!(!is_aether_comm(""));
        assert!(!is_aether(std::process::id()));
    }
}
