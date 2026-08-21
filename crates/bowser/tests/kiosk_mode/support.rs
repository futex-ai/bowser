//! Managed external X display for the kiosk integration test.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub(super) fn is_child(marker: &str) -> bool {
    std::env::var(marker).is_ok_and(|value| value == "1")
}

pub(super) fn run_on_managed_display(test_name: &str, marker: &str, width: u32, height: u32) {
    let display = ManagedDisplay::launch(width, height);
    let executable = std::env::current_exe().expect("kiosk test executable");
    let status = Command::new(executable)
        .args(["--exact", test_name, "--nocapture"])
        .env("DISPLAY", display.name())
        .env(marker, "1")
        .status()
        .expect("run kiosk test on managed display");
    assert!(
        status.success(),
        "managed-display kiosk test failed: {status}"
    );
}

struct ManagedDisplay {
    name: String,
    window_manager: Child,
    xvfb: Child,
}

impl ManagedDisplay {
    fn launch(width: u32, height: u32) -> Self {
        let name = free_display();
        let geometry = format!("{width}x{height}x24");
        let mut xvfb = Command::new("Xvfb")
            .arg(&name)
            .args(["-screen", "0"])
            .arg(&geometry)
            .args(["-nolisten", "tcp"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start kiosk Xvfb");
        wait_for_display(&name, &mut xvfb);
        let window_manager = Command::new("metacity")
            .args(["--display", name.as_str(), "--sm-disable", "--no-composite"])
            .env("GSETTINGS_BACKEND", "memory")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start kiosk window manager");
        std::thread::sleep(Duration::from_millis(500));
        Self {
            name,
            window_manager,
            xvfb,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for ManagedDisplay {
    fn drop(&mut self) {
        let _ = self.window_manager.kill();
        let _ = self.window_manager.wait();
        let _ = self.xvfb.kill();
        let _ = self.xvfb.wait();
    }
}

fn free_display() -> String {
    for number in 200..250 {
        let name = format!(":{number}");
        if !display_socket(&name).exists() {
            return name;
        }
    }
    panic!("no free X display for kiosk test");
}

fn wait_for_display(display: &str, process: &mut Child) {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(3) {
        if display_socket(display).exists() {
            return;
        }
        if process.try_wait().expect("inspect kiosk Xvfb").is_some() {
            panic!("kiosk Xvfb exited before creating its display");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for kiosk X display");
}

fn display_socket(display: &str) -> PathBuf {
    let number = display.strip_prefix(':').unwrap_or(display);
    PathBuf::from("/tmp/.X11-unix").join(format!("X{number}"))
}
