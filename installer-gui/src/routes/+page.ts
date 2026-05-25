import { invoke } from '@tauri-apps/api/core';

interface InstallerCommand {
    subcommands: InstallerSubcommand[];
}

interface InstallerSubcommand {
    command: string;
    label: string;
    arguments: InstallerArgument[];
}

interface InstallerArgument {
    advanced: boolean;
    flag: string;
    label: string;
    takes_values: boolean;
}

export async function load(): Promise<InstallerCommand> {
    return await invoke('rayhunter_options');
}
