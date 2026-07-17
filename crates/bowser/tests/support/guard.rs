//! Cross-test browser locking helpers.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

const OWNER_FILE: &str = "owner.pid";
const LEGACY_LOCK_MAX_AGE: Duration = Duration::from_secs(30);

pub struct BrowserTestGuard {
    path: PathBuf,
}

impl Drop for BrowserTestGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path.join(OWNER_FILE));
        let _ = fs::remove_dir(&self.path);
    }
}

pub fn browser_test_guard() -> BrowserTestGuard {
    let path = env::temp_dir().join("bowser-browser-tests.lock");
    loop {
        match fs::create_dir(&path) {
            Ok(()) => {
                let owner_path = path.join(OWNER_FILE);
                fs::write(&owner_path, std::process::id().to_string())
                    .expect("write browser test lock owner");
                return BrowserTestGuard { path };
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                clear_stale_lock(&path);
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(err) => panic!("failed to acquire browser test lock: {err}"),
        }
    }
}

fn clear_stale_lock(path: &PathBuf) {
    let owner_path = path.join(OWNER_FILE);
    if let Ok(owner) = fs::read_to_string(&owner_path) {
        if owner
            .trim()
            .parse()
            .ok()
            .is_some_and(|pid| !process_is_alive(pid))
        {
            let _ = fs::remove_file(owner_path);
            let _ = fs::remove_dir(path);
        }
        return;
    }
    if fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .and_then(|modified| modified.elapsed().map_err(std::io::Error::other))
        .is_ok_and(|age| age > LEGACY_LOCK_MAX_AGE)
    {
        let _ = fs::remove_dir(path);
    }
}

fn process_is_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .is_ok_and(|status| status.success())
}
