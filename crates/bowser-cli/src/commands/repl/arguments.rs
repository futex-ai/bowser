//! REPL argument parsing helpers.

use std::path::PathBuf;

use bowser::ScrollTarget;

use crate::error::{CliError, Result};

use super::command::ReplCommand;

pub(super) fn parse_new_page(tokens: &[String], line: &str) -> Result<ReplCommand> {
    match tokens {
        [command, target] if command == "new" && target == "page" => Ok(ReplCommand::NewPage(None)),
        [command, target, url] if command == "new" && target == "page" => {
            Ok(ReplCommand::NewPage(Some(url.clone())))
        }
        _ => Err(CliError::InvalidCommand {
            input: line.to_string(),
        }),
    }
}

pub(super) fn parse_close_page(tokens: &[String], line: &str) -> Result<ReplCommand> {
    match tokens {
        [command, target] if command == "close" && target == "page" => {
            Ok(ReplCommand::ClosePage(None))
        }
        [command, target, page_id] if command == "close" && target == "page" => {
            Ok(ReplCommand::ClosePage(Some(page_id.clone())))
        }
        _ => Err(CliError::InvalidCommand {
            input: line.to_string(),
        }),
    }
}

pub(super) fn parse_scroll(tokens: &[String], line: &str) -> Result<ReplCommand> {
    match tokens {
        [command, direction] if command == "scroll" && direction == "down" => {
            Ok(ReplCommand::Scroll(ScrollTarget::Down))
        }
        [command, direction] if command == "scroll" && direction == "up" => {
            Ok(ReplCommand::Scroll(ScrollTarget::Up))
        }
        [command, target, element_id] if command == "scroll" && target == "to" => Ok(
            ReplCommand::Scroll(ScrollTarget::ToElement(parse_u32(element_id, line)?)),
        ),
        _ => Err(CliError::InvalidCommand {
            input: line.to_string(),
        }),
    }
}

pub(super) fn parse_screenshot(tokens: &[String], line: &str) -> Result<ReplCommand> {
    match tokens.len() {
        1 => Ok(ReplCommand::Screenshot(None, None)),
        2 => {
            if let Ok(id) = tokens[1].parse::<u32>() {
                Ok(ReplCommand::Screenshot(Some(id), None))
            } else {
                Ok(ReplCommand::Screenshot(
                    None,
                    Some(PathBuf::from(&tokens[1])),
                ))
            }
        }
        3 => Ok(ReplCommand::Screenshot(
            Some(parse_u32(&tokens[1], line)?),
            Some(PathBuf::from(&tokens[2])),
        )),
        _ => Err(CliError::InvalidCommand {
            input: line.to_string(),
        }),
    }
}

pub(super) fn parse_id_and_value(tokens: &[String], line: &str) -> Result<(u32, String)> {
    match tokens {
        [_, element_id, value] => Ok((parse_u32(element_id, line)?, value.clone())),
        _ => Err(CliError::InvalidCommand {
            input: line.to_string(),
        }),
    }
}

pub(super) fn one_arg(tokens: &[String], line: &str) -> Result<String> {
    match tokens {
        [_, value] => Ok(value.clone()),
        _ => Err(CliError::InvalidCommand {
            input: line.to_string(),
        }),
    }
}

pub(super) fn one_arg_u32(tokens: &[String], line: &str) -> Result<u32> {
    parse_u32(&one_arg(tokens, line)?, line)
}

pub(super) fn optional_u32(tokens: &[String], line: &str) -> Result<Option<u32>> {
    match tokens {
        [_] => Ok(None),
        [_, value] => Ok(Some(parse_u32(value, line)?)),
        _ => Err(CliError::InvalidCommand {
            input: line.to_string(),
        }),
    }
}

pub(super) fn no_args(tokens: &[String], line: &str, command: ReplCommand) -> Result<ReplCommand> {
    if tokens.len() == 1 {
        return Ok(command);
    }
    Err(CliError::InvalidCommand {
        input: line.to_string(),
    })
}

pub(super) fn parse_u32(value: &str, line: &str) -> Result<u32> {
    parse_element_id(value).ok_or_else(|| CliError::InvalidCommand {
        input: line.to_string(),
    })
}

fn parse_element_id(value: &str) -> Option<u32> {
    value
        .rsplit_once('#')
        .map_or(value, |(_, suffix)| suffix)
        .parse::<u32>()
        .ok()
}
