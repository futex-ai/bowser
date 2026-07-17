use std::fs;

#[cfg(unix)]
use std::os::unix::fs::symlink;
use tempfile::tempdir;

use super::{
    ACTIVE_PORT_FILE, DebugPortReader, FileDebugPortReader, MAX_ACTIVE_PORT_BYTES,
    parse_active_debug_port,
};

#[test]
fn parses_complete_dynamic_debug_port_handshakes() {
    let active_port = parse_active_debug_port(
        b"49152\n/devtools/browser/33333333-3333-3333-3333-333333333333\n".to_vec(),
    )
    .expect("active debug port");

    assert_eq!(active_port.port(), 49152);
}

#[test]
fn rejects_incomplete_or_zero_dynamic_debug_port_handshakes() {
    assert!(parse_active_debug_port(b"49152\n".to_vec()).is_none());
    assert!(parse_active_debug_port(b"0\n/devtools/browser/not-active\n".to_vec()).is_none());
    assert!(parse_active_debug_port(b"49152\n/devtools/page/not-browser\n".to_vec()).is_none());
}

#[test]
fn rejects_oversized_dynamic_debug_port_handshake_files() {
    let profile = tempdir().expect("profile");
    fs::write(
        profile.path().join(ACTIVE_PORT_FILE),
        vec![b'x'; MAX_ACTIVE_PORT_BYTES as usize + 1],
    )
    .expect("write oversized handshake");

    assert!(FileDebugPortReader.read(profile.path()).is_none());
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_dynamic_debug_port_handshake_files() {
    let profile = tempdir().expect("profile");
    let target = profile.path().join("handshake-target");
    fs::write(
        &target,
        b"49152\n/devtools/browser/33333333-3333-3333-3333-333333333333\n",
    )
    .expect("write handshake target");
    symlink(&target, profile.path().join(ACTIVE_PORT_FILE)).expect("symlink handshake");

    assert!(FileDebugPortReader.read(profile.path()).is_none());
}
