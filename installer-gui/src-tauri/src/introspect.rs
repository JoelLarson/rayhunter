use std::collections::HashMap;

use serde::Serialize;

use crate::modifiers;

#[derive(Debug, Serialize)]
pub struct Command<'a> {
    subcommands: Vec<Subcommand<'a>>,
}

impl Command<'_> {
    pub fn new(command: &clap::Command) -> Command<'_> {
        let subcommand_map: HashMap<&str, &clap::Command> = command
            .get_subcommands()
            .map(|s| (s.get_name(), s))
            .collect();

        Command {
            subcommands: modifiers::subcommand_modifiers()
                .iter()
                .filter_map(|modifier| {
                    subcommand_map
                        .get(modifier.command)
                        .map(|subcommand| Subcommand::new(subcommand, modifier))
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Argument<'a> {
    advanced: bool,
    flag: String,
    label: &'a str,
    takes_values: bool,
}

#[derive(Debug, Serialize)]
struct Subcommand<'a> {
    arguments: Vec<Argument<'a>>,
    command: &'a str,
    label: &'a str,
}

impl Argument<'_> {
    fn new<'a>(
        argument: &'a clap::Arg,
        modifier: &modifiers::ArgumentModifier<'static>,
    ) -> Argument<'a> {
        Argument {
            advanced: modifier.advanced,
            flag: argument
                .get_long()
                .map(|l| format!("--{l}"))
                .unwrap_or_else(|| {
                    argument
                        .get_short()
                        .map(|s| format!("-{s}"))
                        .unwrap_or_else(|| "".to_string())
                }),
            label: modifier.gui_label,
            takes_values: argument.get_action().takes_values(),
        }
    }
}

impl Subcommand<'_> {
    fn new<'a>(
        command: &'a clap::Command,
        modifier: &modifiers::SubcommandModifier<'static>,
    ) -> Subcommand<'a> {
        let argument_map: HashMap<&str, &clap::Arg> = command
            .get_arguments()
            .map(|a| (a.get_id().as_str(), a))
            .collect();

        Subcommand {
            arguments: modifier
                .arg_modifiers
                .iter()
                .filter_map(|arg_modifier| {
                    argument_map
                        .get(arg_modifier.cli_name)
                        .map(|arg| Argument::new(arg, arg_modifier))
                })
                .collect(),
            command: modifier.command,
            label: modifier.gui_label,
        }
    }
}
