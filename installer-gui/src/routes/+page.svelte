<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { listen } from '@tauri-apps/api/event';
    import { openUrl } from '@tauri-apps/plugin-opener';
    import type { PageProps } from './$types';

    let { data }: PageProps = $props();

    let currentScreen = $state<'select' | 'configure' | 'installing' | 'success' | 'failure'>('select');
    let selectedSubcommandIndex = $state<number>(-1);
    let showAdvanced = $state<boolean>(false);
    let argsValues = $state<Record<string, any>>({});
    let installerOutput = $state<string>('');
    let installerError = $state<string>('');
    let logContainer = $state<HTMLDivElement | null>(null);

    listen<string>('installer-output', (event) => {
        installerOutput += event.payload;
    });

    $effect(() => {
        if (installerOutput && logContainer) {
            logContainer.scrollTop = logContainer.scrollHeight;
        }
    });

    const selectedSubcommand = $derived(
        selectedSubcommandIndex >= 0 ? data.subcommands[selectedSubcommandIndex] : null
    );

    function selectDevice(index: number) {
        selectedSubcommandIndex = index;
        const sub = data.subcommands[index];
        const newVals: Record<string, any> = {};
        
        for (const arg of sub.arguments) {
            if (arg.takes_values) {
                if (arg.flag === '--admin-ip') {
                    newVals[arg.flag] = '192.168.0.1';
                } else {
                    newVals[arg.flag] = '';
                }
            } else {
                newVals[arg.flag] = false;
            }
        }
        argsValues = newVals;
    }

    function goNext() {
        if (currentScreen === 'select' && selectedSubcommandIndex >= 0) {
            currentScreen = 'configure';
        }
    }

    function goBack() {
        if (currentScreen === 'configure') {
            currentScreen = 'select';
        } else if (currentScreen === 'success' || currentScreen === 'failure') {
            currentScreen = 'select';
            selectedSubcommandIndex = -1;
            installerOutput = '';
            installerError = '';
        }
    }

    async function runInstaller() {
        if (!selectedSubcommand) return;

        currentScreen = 'installing';
        installerOutput = 'Starting Rayhunter GUI Installer...\n';
        installerError = '';

        const argsVec: string[] = [];
        argsVec.push(selectedSubcommand.command);
        
        for (const arg of selectedSubcommand.arguments) {
            const val = argsValues[arg.flag];
            if (arg.takes_values) {
                if (val && typeof val === 'string' && val.trim() !== '') {
                    argsVec.push(arg.flag);
                    argsVec.push(val.trim());
                }
            } else {
                if (val === true) {
                    argsVec.push(arg.flag);
                }
            }
        }

        try {
            await invoke('install_rayhunter', { args: argsVec });
            currentScreen = 'success';
        } catch (error) {
            installerError = typeof error === 'string' ? error : JSON.stringify(error);
            currentScreen = 'failure';
        }
    }

    function copyLogs() {
        navigator.clipboard.writeText(installerOutput);
    }

    async function openDashboard() {
        const customIp = argsValues['--admin-ip'];
        const ip = typeof customIp === 'string' && customIp.trim() !== '' ? customIp.trim() : '192.168.0.1';
        await openUrl(`http://${ip}:8080`);
    }
</script>

<div class="p-4 xl:px-8 bg-slate-900 border-b border-slate-800 flex flex-row justify-between items-center shadow-lg">
    <div class="flex items-center gap-3">
        <img src="/orca.svg" alt="Rayhunter Orca" class="h-10 w-10 animate-pulse" />
        <span class="text-white font-bold text-xl tracking-wide">Rayhunter Installer</span>
    </div>
    <div class="flex flex-row gap-6">
        <a
            class="flex flex-row items-center gap-1.5 text-slate-300 hover:text-white transition-colors duration-200 text-sm font-medium"
            href="https://github.com/EFForg/rayhunter/issues"
            target="_blank"
        >
            <svg
                class="w-5 h-5"
                aria-hidden="true"
                xmlns="http://www.w3.org/2000/svg"
                fill="currentColor"
                viewBox="0 0 24 24"
            >
                <path
                    fill-rule="evenodd"
                    d="M12.006 2a9.847 9.847 0 0 0-6.484 2.44 10.32 10.32 0 0 0-3.393 6.17 10.48 10.48 0 0 0 1.317 6.955 10.045 10.045 0 0 0 5.4 4.418c.504.095.683-.223.683-.494 0-.245-.01-1.052-.014-1.908-2.78.62-3.366-1.21-3.366-1.21a2.711 2.711 0 0 0-1.11-1.5c-.907-.637.07-.621.07-.621.317.044.62.163.885.346.266.183.487.426.647.71.135.253.318.476.538.655a2.079 2.079 0 0 0 2.37.196c.045-.52.27-1.006.635-1.37-2.219-.259-4.554-1.138-4.554-5.07a4.022 4.022 0 0 1 1.031-2.75 3.77 3.77 0 0 1 .096-2.713s.839-.275 2.749 1.05a9.26 9.26 0 0 1 5.004 0c1.906-1.325 2.74-1.05 2.74-1.05.37.858.406 1.828.101 2.713a4.017 4.017 0 0 1 1.029 2.75c0 3.939-2.339 4.805-4.564 5.058a2.471 2.471 0 0 1 .679 1.897c0 1.372-.012 2.477-.012 2.814 0 .272.18.592.687.492a10.05 10.05 0 0 0 5.388-4.421 10.473 10.473 0 0 0 1.313-6.948 10.32 10.32 0 0 0-3.39-6.165A9.847 9.847 0 0 0 12.007 2Z"
                />
            </svg>
            <span>Report Issue</span>
        </a>
        <a
            class="flex flex-row items-center gap-1.5 text-slate-300 hover:text-white transition-colors duration-200 text-sm font-medium"
            href="https://efforg.github.io/rayhunter/"
            target="_blank"
        >
            <svg
                class="w-5 h-5"
                aria-hidden="true"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M5 19V4a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v13H7a2 2 0 0 0-2 2Zm0 0a2 2 0 0 0 2 2h12M9 3v14m7 0v4"
                />
            </svg>
            <span>Docs</span>
        </a>
    </div>
</div>

<div class="min-h-[calc(100vh-73px)] bg-radial from-slate-900 via-slate-950 to-black flex justify-center items-center p-6">
    <div class="w-full max-w-3xl bg-slate-900/60 backdrop-blur-xl border border-slate-800 rounded-2xl shadow-2xl p-6 md:p-8 flex flex-col gap-6">
        
        <div class="flex items-center justify-between border-b border-slate-800/80 pb-6">
            <div class="flex items-center gap-2">
                <div class="h-8 w-8 rounded-full flex items-center justify-center font-bold text-sm shadow-md transition-all duration-300
                    {currentScreen === 'select' ? 'bg-rayhunter-blue text-white ring-4 ring-rayhunter-blue/20' : 'bg-rayhunter-green text-slate-950'}">
                    {currentScreen !== 'select' ? '✓' : '1'}
                </div>
                <span class="text-sm font-semibold {currentScreen === 'select' ? 'text-white' : 'text-slate-400'}">Device</span>
            </div>
            <div class="flex-1 h-0.5 mx-4 bg-slate-800">
                <div class="h-full bg-rayhunter-green transition-all duration-500" 
                     style="width: {currentScreen === 'select' ? '0%' : currentScreen === 'configure' ? '50%' : '100%'}"></div>
            </div>
            <div class="flex items-center gap-2">
                <div class="h-8 w-8 rounded-full flex items-center justify-center font-bold text-sm shadow-md transition-all duration-300
                    {currentScreen === 'configure' ? 'bg-rayhunter-blue text-white ring-4 ring-rayhunter-blue/20' : 
                     (currentScreen === 'installing' || currentScreen === 'success' || currentScreen === 'failure') ? 'bg-rayhunter-green text-slate-950' : 'bg-slate-800 text-slate-500'}">
                    {currentScreen === 'success' || currentScreen === 'failure' ? '✓' : '2'}
                </div>
                <span class="text-sm font-semibold {currentScreen === 'configure' ? 'text-white' : 'text-slate-400'}">Configure</span>
            </div>
            <div class="flex-1 h-0.5 mx-4 bg-slate-800">
                <div class="h-full bg-rayhunter-green transition-all duration-500" 
                     style="width: {currentScreen === 'success' || currentScreen === 'failure' ? '100%' : '0%'}"></div>
            </div>
            <div class="flex items-center gap-2">
                <div class="h-8 w-8 rounded-full flex items-center justify-center font-bold text-sm shadow-md transition-all duration-300
                    {currentScreen === 'installing' ? 'bg-rayhunter-blue text-white ring-4 ring-rayhunter-blue/20' : 
                     currentScreen === 'success' ? 'bg-rayhunter-green text-slate-950' : 'bg-slate-800 text-slate-500'}">
                    3
                </div>
                <span class="text-sm font-semibold {currentScreen === 'installing' || currentScreen === 'success' || currentScreen === 'failure' ? 'text-white' : 'text-slate-400'}">Install</span>
            </div>
        </div>

        {#if currentScreen === 'select'}
            <div class="flex flex-col gap-6">
                <div class="text-center flex flex-col items-center gap-2">
                    <img src="/orca.svg" alt="Rayhunter Logo" class="h-20 w-20 transition-transform duration-300 hover:scale-105" />
                    <h1 class="text-2xl font-bold text-white tracking-tight mt-2">Connect Your Device</h1>
                    <p class="text-slate-400 text-sm max-w-md">
                        Select the cellular hotspot device you wish to install the Rayhunter monitoring daemon onto.
                    </p>
                </div>

                <div class="bg-indigo-950/20 border border-indigo-900/40 rounded-xl p-4 flex gap-3 text-slate-300 text-sm">
                    <span class="text-rayhunter-green font-bold">ℹ</span>
                    <p>Ensure your device is connected to your computer via USB or Wi-Fi, powered on, and reachable before proceeding.</p>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 max-h-[300px] overflow-y-auto pr-1">
                    {#each data.subcommands as subcommand, index (subcommand.command)}
                        <button
                            class="text-left p-4 rounded-xl border transition-all duration-200 flex flex-col gap-1 cursor-pointer bg-slate-800/40
                                {selectedSubcommandIndex === index 
                                    ? 'border-rayhunter-green/80 bg-rayhunter-green/5 shadow-[0_0_15px_rgba(148,234,24,0.1)]' 
                                    : 'border-slate-800 hover:border-slate-700 hover:bg-slate-800/60'}"
                            onclick={() => selectDevice(index)}
                        >
                            <span class="font-bold text-white text-base flex justify-between items-center w-full">
                                {subcommand.label}
                                {#if selectedSubcommandIndex === index}
                                    <span class="text-rayhunter-green text-xs font-bold bg-rayhunter-green/10 px-2 py-0.5 rounded">Selected</span>
                                {/if}
                            </span>
                            <span class="text-slate-400 text-xs tracking-wider uppercase font-mono">CLI Command: {subcommand.command}</span>
                        </button>
                    {/each}
                </div>

                <div class="flex justify-end mt-4">
                    <button
                        class="px-8 py-3 rounded-xl font-semibold shadow-lg transition-all duration-200 flex items-center gap-2 cursor-pointer
                            {selectedSubcommandIndex >= 0 
                                ? 'bg-rayhunter-blue hover:bg-rayhunter-dark-blue text-white hover:scale-[1.02]' 
                                : 'bg-slate-800 text-slate-500 cursor-not-allowed'}"
                        disabled={selectedSubcommandIndex < 0}
                        onclick={goNext}
                    >
                        <span>Next</span>
                        <span>➔</span>
                    </button>
                </div>
            </div>
        {/if}

        {#if currentScreen === 'configure' && selectedSubcommand}
            <div class="flex flex-col gap-6">
                <div>
                    <h2 class="text-xl font-bold text-white">{selectedSubcommand.label} Configuration</h2>
                    <p class="text-slate-400 text-xs font-mono mt-1">Configure options for `{selectedSubcommand.command}`</p>
                </div>

                <div class="flex flex-col gap-4 overflow-y-auto max-h-[380px] pr-2">
                    {#each selectedSubcommand.arguments.filter(a => !a.advanced) as arg}
                        <div class="flex flex-col gap-1.5 p-1">
                            <label class="text-sm font-semibold text-slate-300" for={arg.flag}>{arg.label}</label>
                            {#if arg.takes_values}
                                <input
                                    id={arg.flag}
                                    type="text"
                                    class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-white focus:outline-none focus:border-rayhunter-blue transition-colors font-medium text-sm"
                                    placeholder="Leave blank for default"
                                    bind:value={argsValues[arg.flag]}
                                />
                            {:else}
                                <div class="flex items-center gap-3 mt-1">
                                    <input
                                        id={arg.flag}
                                        type="checkbox"
                                        class="h-5 w-5 rounded border-slate-800 bg-slate-950 text-rayhunter-blue focus:ring-rayhunter-blue"
                                        bind:checked={argsValues[arg.flag]}
                                    />
                                    <span class="text-slate-400 text-xs">Enable {arg.label} ({arg.flag})</span>
                                </div>
                            {/if}
                        </div>
                    {/each}

                    {#if selectedSubcommand.arguments.some(a => a.advanced)}
                        <div class="border-t border-slate-800/80 pt-4 mt-2">
                            <button
                                type="button"
                                class="flex items-center gap-2 text-slate-400 hover:text-white transition-colors text-sm font-semibold cursor-pointer"
                                onclick={() => showAdvanced = !showAdvanced}
                            >
                                <span>{showAdvanced ? '▼' : '▶'}</span>
                                <span>Advanced Settings</span>
                            </button>

                            {#if showAdvanced}
                                <div class="flex flex-col gap-4 mt-4 pl-4 border-l border-slate-800">
                                    {#each selectedSubcommand.arguments.filter(a => a.advanced) as arg}
                                        <div class="flex flex-col gap-1.5">
                                            <label class="text-sm font-semibold text-slate-300" for={arg.flag}>{arg.label}</label>
                                            {#if arg.takes_values}
                                                <input
                                                    id={arg.flag}
                                                    type="text"
                                                    class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-white focus:outline-none focus:border-rayhunter-blue transition-colors font-medium text-sm"
                                                    placeholder="Default CLI value"
                                                    bind:value={argsValues[arg.flag]}
                                                />
                                            {:else}
                                                <div class="flex items-center gap-3 mt-1">
                                                    <input
                                                        id={arg.flag}
                                                        type="checkbox"
                                                        class="h-5 w-5 rounded border-slate-800 bg-slate-950 text-rayhunter-blue focus:ring-rayhunter-blue"
                                                        bind:checked={argsValues[arg.flag]}
                                                    />
                                                    <span class="text-slate-400 text-xs">Enable {arg.label} ({arg.flag})</span>
                                                </div>
                                            {/if}
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>

                <div class="flex justify-between mt-4">
                    <button
                        class="px-6 py-3 rounded-xl border border-slate-800 hover:border-slate-700 hover:bg-slate-800/40 text-slate-300 font-semibold cursor-pointer transition-colors duration-200"
                        onclick={goBack}
                    >
                        Back
                    </button>
                    <button
                        class="px-8 py-3 rounded-xl font-bold bg-linear-to-r from-rayhunter-blue to-rayhunter-dark-blue text-white shadow-lg shadow-rayhunter-blue/20 hover:scale-[1.02] cursor-pointer transition-all duration-200"
                        onclick={runInstaller}
                    >
                        Start Installation
                    </button>
                </div>
            </div>
        {/if}

        {#if currentScreen === 'installing'}
            <div class="flex flex-col gap-6">
                <div class="flex items-center gap-4">
                    <div class="h-10 w-10 border-4 border-slate-800 border-t-rayhunter-blue rounded-full animate-spin"></div>
                    <div>
                        <h2 class="text-xl font-bold text-white">Installing Rayhunter...</h2>
                        <p class="text-slate-400 text-sm">Please do not disconnect the device or close this installer.</p>
                    </div>
                </div>

                <div class="flex flex-col gap-2">
                    <span class="text-slate-400 text-xs font-mono uppercase tracking-wider">Terminal Output Log</span>
                    <div 
                        bind:this={logContainer}
                        class="h-[320px] bg-black/90 border border-slate-800 rounded-xl p-4 font-mono text-xs text-slate-300 overflow-y-auto whitespace-pre-line leading-relaxed scrollbar-thin scrollbar-thumb-slate-800"
                    >
                        {installerOutput}
                    </div>
                </div>
            </div>
        {/if}

        {#if currentScreen === 'success'}
            <div class="flex flex-col gap-6 items-center text-center py-6">
                <div class="h-20 w-20 rounded-full bg-rayhunter-green/10 border-2 border-rayhunter-green flex items-center justify-center shadow-[0_0_20px_rgba(148,234,24,0.15)] animate-bounce">
                    <span class="text-rayhunter-green text-4xl font-bold">✓</span>
                </div>

                <div class="flex flex-col gap-2 max-w-md">
                    <h2 class="text-2xl font-bold text-white">Installation Successful!</h2>
                    <p class="text-slate-400 text-sm">
                        The Rayhunter monitoring daemon has been successfully installed and started on your hotspot device.
                    </p>
                </div>

                <div class="bg-slate-950 border border-slate-800 rounded-xl p-4 flex flex-col gap-2 text-left w-full text-sm">
                    <span class="font-bold text-slate-300">Accessing Rayhunter</span>
                    <p class="text-slate-400">
                        The web dashboard runs directly on the device. Connect your browser to the device to view cellular network alerts.
                    </p>
                    <span class="font-mono text-xs text-rayhunter-green mt-1">Dashboard Address: http://{(argsValues['--admin-ip'] as string) || '192.168.0.1'}:8080</span>
                </div>

                <div class="flex flex-col gap-3 w-full mt-4">
                    <button
                        class="w-full py-3 rounded-xl font-bold bg-rayhunter-green text-slate-950 hover:bg-opacity-90 shadow-lg shadow-rayhunter-green/10 transition-all duration-200 cursor-pointer"
                        onclick={openDashboard}
                    >
                        Open Dashboard in Browser
                    </button>
                    <button
                        class="w-full py-3 rounded-xl border border-slate-800 hover:border-slate-700 text-slate-300 font-semibold transition-colors duration-200 cursor-pointer"
                        onclick={goBack}
                    >
                        Finish & Close
                    </button>
                </div>
            </div>
        {/if}

        {#if currentScreen === 'failure'}
            <div class="flex flex-col gap-6 items-center text-center py-6">
                <div class="h-20 w-20 rounded-full bg-red-900/10 border-2 border-red-500 flex items-center justify-center shadow-[0_0_20px_rgba(239,68,68,0.15)]">
                    <span class="text-red-500 text-4xl font-bold">✗</span>
                </div>

                <div class="flex flex-col gap-2 max-w-md">
                    <h2 class="text-2xl font-bold text-white">Installation Failed</h2>
                    <p class="text-slate-400 text-sm">
                        An error occurred while trying to install Rayhunter.
                    </p>
                </div>

                <div class="bg-slate-950 border border-slate-800 rounded-xl p-4 text-left w-full text-xs font-mono text-red-400/90 whitespace-pre-wrap max-h-[150px] overflow-y-auto">
                    {installerError}
                </div>

                <div class="flex flex-col gap-3 w-full mt-4">
                    <button
                        class="w-full py-3 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-semibold transition-all duration-200 cursor-pointer flex justify-center items-center gap-2"
                        onclick={copyLogs}
                    >
                        <span>📋</span>
                        <span>Copy Logs to Clipboard</span>
                    </button>
                    <button
                        class="w-full py-3 rounded-xl border border-slate-800 hover:border-slate-700 text-slate-300 font-semibold transition-colors duration-200 cursor-pointer"
                        onclick={goBack}
                    >
                        Try Again
                    </button>
                </div>
            </div>
        {/if}

    </div>
</div>

<style>
    .scrollbar-thin::-webkit-scrollbar {
        width: 6px;
    }
    .scrollbar-thin::-webkit-scrollbar-track {
        background: transparent;
    }
    .scrollbar-thin::-webkit-scrollbar-thumb {
        background-color: #334155;
        border-radius: 3px;
    }
</style>
