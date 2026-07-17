//! Owned browser-profile cleanup tests.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::error::Error;

use super::profile::{OwnedProfileGuard, ProfileDirectoryControl, ProfileDirectoryControlMock};

#[test]
fn owned_profile_cleanup_removes_only_a_generated_direct_child() {
    let root = PathBuf::from("/tmp/bowser-sessions");
    let profile = root
        .join("profiles")
        .join("4b191f8c-5187-4327-a342-e4bf0572995d");
    let removed = Arc::new(Mutex::new(Vec::new()));
    let control: Arc<dyn ProfileDirectoryControl> = Arc::new(Unimock::new(
        ProfileDirectoryControlMock::remove
            .next_call(matching!(_))
            .answers_arc({
                let removed = removed.clone();
                Arc::new(move |_, path: &Path| {
                    removed
                        .lock()
                        .expect("removed profiles")
                        .push(path.to_path_buf());
                    Ok(())
                })
            }),
    ));

    {
        let _guard =
            OwnedProfileGuard::new(control, &root, profile.clone(), true).expect("profile guard");
    }

    assert_eq!(
        removed.lock().expect("removed profiles").as_slice(),
        [profile]
    );
}

#[test]
fn owned_profile_cleanup_rejects_paths_outside_the_generated_root() {
    let root = PathBuf::from("/tmp/bowser-sessions");
    let control: Arc<dyn ProfileDirectoryControl> = Arc::new(Unimock::new(()));

    let result =
        OwnedProfileGuard::new(control, &root, PathBuf::from("/tmp/explicit-profile"), true);

    assert!(matches!(result, Err(Error::InvalidOwnedProfilePath { .. })));
}

#[test]
fn unowned_profile_cleanup_leaves_the_path_untouched() {
    let root = PathBuf::from("/tmp/bowser-sessions");
    let control: Arc<dyn ProfileDirectoryControl> = Arc::new(Unimock::new(()));

    let _guard = OwnedProfileGuard::new(
        control,
        &root,
        PathBuf::from("/tmp/explicit-profile"),
        false,
    )
    .expect("skip profile");
}

#[test]
fn armed_profile_guard_removes_an_unfinished_profile() {
    let root = PathBuf::from("/tmp/bowser-sessions");
    let profile = root
        .join("profiles")
        .join("993c319a-8c46-46d0-83ca-a5a7dbad1790");
    let control: Arc<dyn ProfileDirectoryControl> = Arc::new(Unimock::new(
        ProfileDirectoryControlMock::remove
            .next_call(matching!(_))
            .returns(Ok(())),
    ));

    let _guard = OwnedProfileGuard::new(control, &root, profile, true).expect("profile guard");
}

#[test]
fn disarmed_profile_guard_transfers_ownership_to_the_session() {
    let root = PathBuf::from("/tmp/bowser-sessions");
    let profile = root
        .join("profiles")
        .join("54837701-5b69-431a-a78a-f6f12f1bd183");
    let control: Arc<dyn ProfileDirectoryControl> = Arc::new(Unimock::new(()));
    let mut guard = OwnedProfileGuard::new(control, &root, profile, true).expect("profile guard");

    guard.disarm();
}
