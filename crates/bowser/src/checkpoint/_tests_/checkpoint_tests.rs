use tempfile::tempdir;

use super::{
    CHECKPOINT_VERSION, CheckpointPage, SessionCheckpoint, read_checkpoint, write_checkpoint,
};

fn valid_checkpoint() -> SessionCheckpoint {
    SessionCheckpoint {
        checkpoint: CHECKPOINT_VERSION,
        created_at: chrono::Utc::now(),
        cookies: Vec::new(),
        origins: Vec::new(),
        pages: vec![CheckpointPage {
            url: "https://example.com/".to_string(),
        }],
        selected_page: 0,
    }
}

#[test]
fn checkpoint_round_trips_through_one_versioned_file() {
    let directory = tempdir().expect("checkpoint directory");
    let path = directory.path().join("session.json");
    let checkpoint = valid_checkpoint();

    write_checkpoint(&path, &checkpoint).expect("write checkpoint");

    assert_eq!(read_checkpoint(&path).expect("read checkpoint"), checkpoint);
}

#[test]
fn checkpoint_rejects_unknown_versions_before_restore() {
    let mut checkpoint = valid_checkpoint();
    checkpoint.checkpoint = CHECKPOINT_VERSION + 1;

    assert!(matches!(
        checkpoint.validate(),
        Err(crate::Error::CheckpointUnsupported { .. })
    ));
}

#[test]
fn checkpoint_rejects_an_out_of_range_selected_page() {
    let mut checkpoint = valid_checkpoint();
    checkpoint.selected_page = 1;

    assert!(matches!(
        checkpoint.validate(),
        Err(crate::Error::CheckpointInvalid { .. })
    ));
}
