//! Human and versioned machine command output.

use serde::Serialize;
use std::path::PathBuf;

use crate::error::{CliError, Result};

/// Session and page identifiers learned while executing one command.
#[derive(Clone, Debug, Default)]
pub(crate) struct CommandContext {
    pub(crate) session: Option<String>,
    pub(crate) page: Option<String>,
    pub(crate) announce_session_on_failure: bool,
}

impl CommandContext {
    pub(crate) fn update_session(&mut self, info: &bowser::SessionInfo) {
        self.session = Some(info.id.clone());
        self.page = info.selected_page_id.clone();
    }

    pub(crate) fn update_from_error(&mut self, error: &CliError) {
        if let CliError::Bowser(bowser::Error::CheckpointRestore {
            session_id: Some(session_id),
            ..
        }) = error
        {
            self.session = Some(session_id.clone());
        }
    }
}

/// Command-specific structured result plus optional human rendering.
#[derive(Clone, Debug)]
pub(crate) struct CommandOutput {
    pub(crate) result: serde_json::Value,
    pub(crate) human_stdout: Option<String>,
    pub(crate) human_stderr: Option<String>,
    pub(crate) files: Vec<PendingFile>,
}

/// File content committed only after the browser operation has detached.
#[derive(Clone, Debug)]
pub(crate) struct PendingFile {
    path: PathBuf,
    bytes: Vec<u8>,
}

impl CommandOutput {
    pub(crate) fn result<T: Serialize>(value: T) -> Result<Self> {
        Ok(Self {
            result: serde_json::to_value(value)
                .map_err(|source| CliError::OutputSerialize { source })?,
            human_stdout: None,
            human_stderr: None,
            files: Vec::new(),
        })
    }

    pub(crate) fn human_stdout(mut self, value: impl Into<String>) -> Self {
        self.human_stdout = Some(value.into());
        self
    }

    pub(crate) fn human_stderr(mut self, value: impl Into<String>) -> Self {
        self.human_stderr = Some(value.into());
        self
    }

    pub(crate) fn text_file(mut self, path: PathBuf, value: String) -> Self {
        self.files.push(PendingFile {
            path,
            bytes: value.into_bytes(),
        });
        self
    }

    pub(crate) fn binary_file(mut self, path: PathBuf, value: Vec<u8>) -> Self {
        self.files.push(PendingFile { path, bytes: value });
        self
    }

    pub(crate) async fn commit_files(&self) -> Result<()> {
        for file in &self.files {
            if tokio::fs::write(&file.path, &file.bytes).await.is_err() {
                return Err(CliError::OutputWrite {
                    path: file.path.display().to_string(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct Envelope<'a> {
    envelope: u32,
    ok: bool,
    session: Option<&'a str>,
    page: Option<&'a str>,
    result: Option<&'a serde_json::Value>,
    error: Option<MachineError>,
}

#[derive(Debug, Serialize)]
struct MachineError {
    code: &'static str,
    message: String,
    detail: serde_json::Value,
}

pub(crate) fn emit_success(
    machine: bool,
    context: &CommandContext,
    output: &CommandOutput,
) -> Result<()> {
    if machine {
        let envelope = Envelope {
            envelope: 1,
            ok: true,
            session: context.session.as_deref(),
            page: context.page.as_deref(),
            result: Some(&output.result),
            error: None,
        };
        println!(
            "{}",
            serde_json::to_string(&envelope)
                .map_err(|source| CliError::OutputSerialize { source })?
        );
    } else if let Some(stdout) = output.human_stdout.as_deref() {
        print!("{stdout}");
    }
    if !machine && let Some(stderr) = output.human_stderr.as_deref() {
        eprint!("{stderr}");
    }
    Ok(())
}

pub(crate) fn emit_failure(
    machine: bool,
    context: &CommandContext,
    error: &CliError,
) -> Result<()> {
    if !machine {
        if context.announce_session_on_failure
            && let Some(session) = context.session.as_deref()
        {
            eprintln!("Session: {session}");
        }
        eprintln!("{error}");
        return Ok(());
    }
    let (code, detail) = error.machine_code_and_detail();
    let envelope = Envelope {
        envelope: 1,
        ok: false,
        session: context.session.as_deref(),
        page: context.page.as_deref(),
        result: None,
        error: Some(MachineError {
            code,
            message: error.to_string(),
            detail,
        }),
    };
    println!(
        "{}",
        serde_json::to_string(&envelope).map_err(|source| CliError::OutputSerialize { source })?
    );
    Ok(())
}

#[cfg(test)]
#[path = "_tests_/output_tests.rs"]
mod output_tests;
