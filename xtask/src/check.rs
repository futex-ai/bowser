//! Execution for selectable workspace verification phases.

use std::fmt::{self, Display};
use std::path::Path;
use std::process::Command;

use crate::check_plan::{CheckSelection, check_commands};
use crate::error::{Error, Result};
use crate::source_layout_lint::verify_source_adjacent_test_layout;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[expect(
    unreachable_pub,
    reason = "unimock's generated public CommandRunner mock exposes this type"
)]
pub struct CheckCommand {
    pub(crate) program: &'static str,
    pub(crate) args: &'static [&'static str],
    pub(crate) envs: &'static [(&'static str, &'static str)],
}

impl CheckCommand {
    pub(crate) const fn root(
        program: &'static str,
        args: &'static [&'static str],
        envs: &'static [(&'static str, &'static str)],
    ) -> Self {
        Self {
            program,
            args,
            envs,
        }
    }
}

impl Display for CheckCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in self.envs {
            write!(formatter, "{key}={value} ")?;
        }

        write!(formatter, "{}", self.program)?;
        for arg in self.args {
            write!(formatter, " {arg}")?;
        }
        Ok(())
    }
}

#[cfg_attr(test, unimock::unimock(api = CommandRunnerMock))]
pub(crate) trait CommandRunner {
    fn run(&self, command: CheckCommand, workspace_root: &Path) -> Result<()>;
}

pub(crate) struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, command: CheckCommand, workspace_root: &Path) -> Result<()> {
        println!("running {command}");

        let status = match Command::new(command.program)
            .args(command.args)
            .envs(command.envs.iter().copied())
            .current_dir(workspace_root)
            .status()
        {
            Ok(status) => status,
            Err(source) => return Err(Error::CommandStart { command, source }),
        };

        if status.success() {
            return Ok(());
        }

        Err(Error::CommandFailed { command, status })
    }
}

pub(crate) fn run_check(
    runner: &dyn CommandRunner,
    workspace_root: &Path,
    selection: &CheckSelection,
) -> Result<()> {
    verify_source_adjacent_test_layout(workspace_root)?;

    for command in check_commands(selection) {
        runner.run(command, workspace_root)?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "_tests_/check/mod.rs"]
mod check_tests;
