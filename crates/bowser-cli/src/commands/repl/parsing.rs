//! REPL command parsing.

use crate::commands::repl_help::COMMANDS;
use crate::error::{CliError, Result};

use super::arguments::{
    no_args, one_arg, one_arg_u32, optional_u32, parse_close_page, parse_id_and_value,
    parse_new_page, parse_screenshot, parse_scroll,
};
use super::command::ReplCommand;

pub(crate) fn parse_command(line: &str) -> Result<ReplCommand> {
    if let Some(rest) = line.strip_prefix("js ") {
        return Ok(ReplCommand::Js(rest.to_string()));
    }
    let tokens = shlex::split(line).ok_or_else(|| CliError::InvalidCommand {
        input: line.to_string(),
    })?;
    if tokens.is_empty() {
        return Err(CliError::InvalidCommand {
            input: line.to_string(),
        });
    }
    match tokens[0].as_str() {
        "goto" => one_arg(&tokens, line).map(ReplCommand::Goto),
        "back" => no_args(&tokens, line, ReplCommand::Back),
        "forward" => no_args(&tokens, line, ReplCommand::Forward),
        "reload" => no_args(&tokens, line, ReplCommand::Reload),
        "pages" => no_args(&tokens, line, ReplCommand::Pages),
        "page" => one_arg(&tokens, line).map(ReplCommand::Page),
        "new" => parse_new_page(&tokens, line),
        "close" => parse_close_page(&tokens, line),
        "yaml" => optional_u32(&tokens, line).map(ReplCommand::Yaml),
        "click" => one_arg_u32(&tokens, line).map(ReplCommand::Click),
        "keypress" => parse_keypress(&tokens, line),
        "type" => parse_id_and_value(&tokens, line).map(|(id, value)| ReplCommand::Type(id, value)),
        "clear" => one_arg_u32(&tokens, line).map(ReplCommand::Clear),
        "select" => {
            parse_id_and_value(&tokens, line).map(|(id, value)| ReplCommand::Select(id, value))
        }
        "submit" => one_arg_u32(&tokens, line).map(ReplCommand::Submit),
        "scroll" => parse_scroll(&tokens, line),
        "screenshot" => parse_screenshot(&tokens, line),
        "wait" => one_arg(&tokens, line).map(ReplCommand::Wait),
        "refresh" => no_args(&tokens, line, ReplCommand::Refresh),
        "expand" => one_arg_u32(&tokens, line).map(ReplCommand::Expand),
        "meta" => one_arg_u32(&tokens, line).map(ReplCommand::Meta),
        "describe" => one_arg_u32(&tokens, line).map(ReplCommand::Describe),
        "session" => no_args(&tokens, line, ReplCommand::Session),
        "url" => no_args(&tokens, line, ReplCommand::Url),
        "title" => no_args(&tokens, line, ReplCommand::Title),
        "html" => no_args(&tokens, line, ReplCommand::Html),
        "help" => no_args(&tokens, line, ReplCommand::Help),
        "quit" | "exit" => no_args(&tokens, line, ReplCommand::Quit),
        other => Err(CliError::InvalidCommand {
            input: if let Some(suggestion) = suggest_command(other) {
                format!("{line} (did you mean `{suggestion}`?)")
            } else {
                line.to_string()
            },
        }),
    }
}

fn parse_keypress(tokens: &[String], line: &str) -> Result<ReplCommand> {
    if tokens.len() < 2 {
        return Err(CliError::InvalidCommand {
            input: line.to_string(),
        });
    }
    let keys: Vec<String> = tokens[1..]
        .iter()
        .flat_map(|token| token.split('+'))
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect();
    if keys.is_empty() {
        return Err(CliError::InvalidCommand {
            input: line.to_string(),
        });
    }
    Ok(ReplCommand::KeyPress(keys))
}

fn suggest_command(input: &str) -> Option<&'static str> {
    COMMANDS
        .iter()
        .copied()
        .max_by_key(|command| shared_prefix_len(command, input))
        .filter(|command| shared_prefix_len(command, input) >= 2)
}

fn shared_prefix_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(l, r)| l == r)
        .count()
}
