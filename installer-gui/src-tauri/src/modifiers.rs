#[derive(Debug, Copy, Clone)]
pub struct ArgumentModifier<'a> {
    pub cli_name: &'a str,
    pub gui_label: &'a str,
    pub advanced: bool,
}

#[derive(Debug, Clone)]
pub struct SubcommandModifier<'a> {
    pub command: &'a str,
    pub gui_label: &'a str,
    pub arg_modifiers: Vec<ArgumentModifier<'a>>,
}

pub fn subcommand_modifiers() -> Vec<SubcommandModifier<'static>> {
    let admin_ip = ArgumentModifier {
        cli_name: "admin_ip",
        gui_label: "Admin IP",
        advanced: true,
    };
    let admin_username = ArgumentModifier {
        cli_name: "admin_username",
        gui_label: "Admin Username",
        advanced: true,
    };
    let admin_password = ArgumentModifier {
        cli_name: "admin_password",
        gui_label: "Admin Password",
        advanced: false,
    };
    let data_dir = ArgumentModifier {
        cli_name: "data_dir",
        gui_label: "Data Directory",
        advanced: true,
    };
    let reset_config = ArgumentModifier {
        cli_name: "reset_config",
        gui_label: "Reset config.toml",
        advanced: true,
    };
    let orbic_and_moxee_args = vec![
        admin_password,
        admin_ip,
        admin_username,
        reset_config,
        data_dir,
    ];

    let mut list = vec![
        SubcommandModifier {
            command: "orbic",
            gui_label: "Orbic/Kajeet (via network)",
            arg_modifiers: orbic_and_moxee_args.clone(),
        },
    ];

    #[cfg(not(target_os = "android"))]
    {
        list.push(SubcommandModifier {
            command: "orbic-usb",
            gui_label: "Orbic/Kajeet (via legacy USB+ADB installer)",
            arg_modifiers: vec![reset_config],
        });
    }

    list.push(SubcommandModifier {
        command: "tplink",
        gui_label: "TP-Link",
        arg_modifiers: vec![
            admin_ip,
            reset_config,
            data_dir,
            ArgumentModifier {
                cli_name: "skip_sdcard",
                gui_label: "Skip SD Card",
                advanced: true,
            },
            ArgumentModifier {
                cli_name: "sdcard_path",
                gui_label: "SD Card Path",
                advanced: true,
            },
        ],
    });

    list.push(SubcommandModifier {
        command: "moxee",
        gui_label: "Moxee",
        arg_modifiers: orbic_and_moxee_args.clone(),
    });

    #[cfg(not(target_os = "android"))]
    {
        list.push(SubcommandModifier {
            command: "pinephone",
            gui_label: "PinePhone",
            arg_modifiers: vec![],
        });
    }

    list.push(SubcommandModifier {
        command: "tmobile",
        gui_label: "TMobile",
        arg_modifiers: vec![admin_password, admin_ip],
    });

    #[cfg(not(target_os = "android"))]
    {
        list.push(SubcommandModifier {
            command: "uz801",
            gui_label: "UZ801",
            arg_modifiers: vec![admin_ip],
        });
    }

    list.push(SubcommandModifier {
        command: "wingtech",
        gui_label: "Wingtech",
        arg_modifiers: vec![admin_password, admin_ip],
    });

    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[allow(dead_code)]
    fn excluded_arguments() -> HashMap<&'static str, Vec<&'static str>> {
        HashMap::new()
    }

    fn excluded_subcommands() -> Vec<&'static str> {
        vec!["util"]
    }

    // These tests ensure that every new CLI option added to the rayhunter CLI installer
    // is explicitly mapped or excluded from the GUI so they don't drift out of sync.
    #[test]
    fn test_subcommands() {
        let exclusions = excluded_subcommands();

        let mut clap_subcommands: Vec<&str> = crate::INSTALLER_COMMAND
            .get_subcommands()
            .filter_map(|c| {
                let name = c.get_name();
                if exclusions.contains(&name) {
                    None
                } else {
                    Some(name)
                }
            })
            .collect();
        let mut modified_subcommands: Vec<&str> = subcommand_modifiers()
            .into_iter()
            .map(|m| m.command)
            .collect();

        clap_subcommands.sort_unstable();
        modified_subcommands.sort_unstable();

        assert_eq!(
            clap_subcommands, modified_subcommands,
            "Every subcommand must be included exactly once in subcommand_modifiers() or excluded_subcommands()."
        );
    }

    #[test]
    fn test_arguments() {
        let exclusions = excluded_subcommands();

        let mut clap_args: Vec<(&str, &str)> = crate::INSTALLER_COMMAND
            .get_subcommands()
            .filter_map(|c| {
                let subcommand_name = c.get_name();
                if exclusions.contains(&subcommand_name) {
                    None
                } else {
                    Some(std::iter::zip(
                        std::iter::repeat(subcommand_name),
                        c.get_arguments().map(|a| a.get_id().as_str()),
                    ))
                }
            })
            .flatten()
            .collect();
        let mut modified_args: Vec<(&str, &str)> = subcommand_modifiers()
            .into_iter()
            .flat_map(|m| {
                std::iter::zip(
                    std::iter::repeat(m.command),
                    m.arg_modifiers.into_iter().map(|arg_m| arg_m.cli_name),
                )
            })
            .collect();

        clap_args.sort_unstable();
        modified_args.sort_unstable();
        assert_eq!(
            clap_args, modified_args,
            "Every argument for non-excluded subcommands must have exactly one ArgumentModifier."
        );
    }
}
