<script lang="ts">
    import { autodetectDevice, installRayhunter, listen, openUrl, getCurrentWindow } from '$lib/tauri';
    import type { PageProps } from './$types';

    let { data }: PageProps = $props();

    let currentScreen = $state<'select' | 'configure' | 'installing' | 'success' | 'failure'>('select');
    let selectedSubcommandIndex = $state<number>(-1);
    let selectedDeviceId = $state<string>('');
    let showAdvanced = $state<boolean>(false);
    let argsValues = $state<Record<string, any>>({});
    let installerOutput = $state<string>('');
    let installerError = $state<string>('');
    let logContainer = $state<HTMLDivElement | null>(null);

    let detectedDevices = $state([] as Awaited<ReturnType<typeof autodetectDevice>>);
    let isDetecting = $state<boolean>(false);
    let hasAttemptedDetection = $state<boolean>(false);
    let deviceDisconnected = $state<boolean>(false);
    let showHelp = $state<boolean>(false);

    async function detectDevice() {
        isDetecting = true;
        hasAttemptedDetection = true;
        deviceDisconnected = false;
        try {
            const detected = await autodetectDevice();
            detectedDevices = detected || [];
            if (selectedDeviceId === '' && detectedDevices.length > 0) {
                selectDevice(detectedDevices[0]);
            } else if (selectedDeviceId !== '') {
                const stillDetected = detectedDevices.some(d => d.id === selectedDeviceId);
                if (!stillDetected) {
                    selectedDeviceId = '';
                    selectedSubcommandIndex = -1;
                    argsValues = {};
                }
            }
        } catch (e) {
            detectedDevices = [];
        } finally {
            isDetecting = false;
        }
    }

    listen<string>('installer-output', (event) => {
        installerOutput += event.payload;
    });

    $effect(() => {
        if (installerOutput && logContainer) {
            logContainer.scrollTop = logContainer.scrollHeight;
        }
    });

    $effect(() => {
        if (currentScreen === 'select' && !hasAttemptedDetection) {
            detectDevice();
        }
    });

    $effect(() => {
        if (currentScreen !== 'select') return;
        const interval = setInterval(async () => {
            if (isDetecting) return;
            try {
                const detected = await autodetectDevice();
                const updated = detected || [];
                detectedDevices = updated;
                if (updated.length > 0) deviceDisconnected = false;
                if (selectedDeviceId !== '' && !updated.some(d => d.id === selectedDeviceId)) {
                    selectedDeviceId = '';
                    selectedSubcommandIndex = -1;
                    argsValues = {};
                    deviceDisconnected = true;
                }
            } catch {
                // silent — don't clear state on transient errors
            }
        }, 3000);
        return () => clearInterval(interval);
    });

    const selectedSubcommand = $derived(
        selectedSubcommandIndex >= 0 ? data.subcommands[selectedSubcommandIndex] : null
    );

    const selectedDevice = $derived(
        detectedDevices.find(d => d.id === selectedDeviceId) ?? null
    );

    function selectDevice(dev: (typeof detectedDevices)[number]) {
        selectedDeviceId = dev.id;
        const index = data.subcommands.findIndex(s => s.command === dev.subcommand);
        selectedSubcommandIndex = index;
        if (index === -1) return;

        const sub = data.subcommands[index];
        const newVals: Record<string, any> = {};
        
        for (const arg of sub.arguments) {
            if (arg.takes_values) {
                if (arg.flag === '--admin-ip') {
                    newVals[arg.flag] = dev.admin_ip || '192.168.0.1';
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
            selectedDeviceId = '';
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
            await installRayhunter(argsVec);
            currentScreen = 'success';
        } catch (error) {
            installerError = typeof error === 'string' ? error : JSON.stringify(error);
            currentScreen = 'failure';
        }
    }

    function copyLogs() {
        navigator.clipboard.writeText(installerOutput);
    }

    function dashboardUrl(): string {
        const customIp = argsValues['--admin-ip'];
        const ip = typeof customIp === 'string' && customIp.trim() !== '' ? customIp.trim() : selectedDevice?.admin_ip ?? '192.168.0.1';
        return `http://${ip}:8080`;
    }

    async function openDashboard() {
        await openUrl(dashboardUrl());
    }
</script>

<div class="p-4 xl:px-8 bg-slate-900 border-b border-slate-800 flex flex-row justify-between items-center shadow-lg">
    <div class="flex items-center gap-3">
        <img src="/orca.svg" alt="Rayhunter Orca" class="h-10 w-10 animate-pulse" />
        <span class="text-white font-bold text-xl tracking-wide">Rayhunter Installer</span>
    </div>
    <div class="flex flex-row gap-6">
        <button
            class="flex flex-row items-center gap-1.5 text-slate-300 hover:text-white transition-colors duration-200 text-sm font-medium cursor-pointer bg-transparent border-none"
            onclick={() => showHelp = !showHelp}
        >
            <svg
                class="w-5 h-5 text-rayhunter-blue"
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
                    d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z"
                />
            </svg>
            <span>Setup Guide & Help</span>
        </button>
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
        
        <nav aria-label="Installation steps" class="border-b border-slate-800/80 pb-6 flex justify-center">
            <ol class="flex items-center gap-3 text-sm font-medium">
                <li aria-current={currentScreen === 'select' ? 'step' : undefined}>
                    <span class="flex items-center gap-1.5 transition-colors
                        {currentScreen === 'select' ? 'text-white' : 'text-rayhunter-green'}">
                        {#if currentScreen !== 'select'}
                            <span aria-hidden="true">✓</span>
                        {/if}
                        Device
                    </span>
                </li>
                <li aria-hidden="true" class="text-slate-500">›</li>
                <li aria-current={currentScreen === 'configure' ? 'step' : undefined}>
                    <span class="flex items-center gap-1.5 transition-colors
                        {currentScreen === 'configure' ? 'text-white' :
                         currentScreen === 'installing' || currentScreen === 'success' || currentScreen === 'failure' ? 'text-rayhunter-green' :
                         'text-slate-500'}">
                        {#if currentScreen === 'installing' || currentScreen === 'success' || currentScreen === 'failure'}
                            <span aria-hidden="true">✓</span>
                        {/if}
                        Configure
                    </span>
                </li>
                <li aria-hidden="true" class="text-slate-500">›</li>
                <li aria-current={currentScreen === 'installing' || currentScreen === 'success' || currentScreen === 'failure' ? 'step' : undefined}>
                    <span class="transition-colors
                        {currentScreen === 'installing' || currentScreen === 'success' || currentScreen === 'failure' ? 'text-white' : 'text-slate-500'}">
                        Install
                    </span>
                </li>
            </ol>
        </nav>

        {#if currentScreen === 'select'}
            <div class="flex flex-col items-center gap-8 py-2">

                <!-- Logo + heading -->
                <div class="flex flex-col items-center gap-3 text-center">
                    <img src="/orca.svg" alt="Rayhunter" class="h-20 w-20" />
                    <div class="flex flex-col gap-1">
                        <h1 class="text-3xl font-bold text-white tracking-tight">Connect Your Device</h1>
                        <p class="text-slate-400 text-sm whitespace-nowrap">Select the cellular hotspot device you wish to install Rayhunter onto.</p>
                    </div>
                </div>

                <!-- Warning -->
                <div class="w-full bg-amber-950/20 border border-amber-700/30 rounded-xl p-4 flex gap-3 items-start">
                    <svg class="w-5 h-5 text-amber-400 mt-0.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                    <p class="text-amber-200/80 text-sm leading-relaxed">
                        Ensure your device is powered on and connected to its Wi-Fi hotspot, or via USB for legacy installs.
                    </p>
                </div>

                <!-- Device list with section header -->
                <div class="w-full flex flex-col gap-3">

                    <!-- Section header: status left, refresh right -->
                    <div class="flex items-center justify-between">
                        <div
                            role="status"
                            aria-live="polite"
                            aria-atomic="true"
                            class="flex items-center gap-2"
                        >
                            {#if isDetecting}
                                <span class="h-4 w-4 border-2 border-slate-700 border-t-rayhunter-blue rounded-full animate-spin" aria-hidden="true"></span>
                                <span class="text-sm text-slate-400">Scanning for devices…</span>
                            {:else if detectedDevices.length > 0}
                                <span class="h-2 w-2 rounded-full bg-rayhunter-green animate-pulse" aria-hidden="true"></span>
                                <span class="text-sm text-slate-300">
                                    <strong class="text-white">{detectedDevices.length}</strong>
                                    {detectedDevices.length === 1 ? 'device' : 'devices'} found
                                </span>
                            {:else if hasAttemptedDetection}
                                <span class="h-2 w-2 rounded-full bg-slate-600 animate-pulse" aria-hidden="true"></span>
                                <span class="text-sm text-slate-500">No devices found</span>
                            {:else}
                                <span class="text-sm text-slate-500">Looking for devices…</span>
                            {/if}
                        </div>

                        <button
                            onclick={detectDevice}
                            disabled={isDetecting}
                            aria-label="Scan for devices now"
                            class="flex items-center gap-1.5 text-sm font-medium transition-colors duration-150
                                {isDetecting
                                    ? 'text-slate-600 cursor-not-allowed'
                                    : 'text-slate-400 hover:text-white cursor-pointer'}"
                        >
                            <svg
                                class="w-4 h-4 {isDetecting ? 'animate-spin' : ''}"
                                fill="none" viewBox="0 0 24 24" stroke="currentColor"
                                aria-hidden="true"
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                    d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                            </svg>
                            Scan Now
                        </button>
                    </div>

                    <!-- Disconnection warning -->
                    {#if deviceDisconnected}
                        <div role="alert" class="bg-red-950/20 border border-red-700/30 rounded-xl p-4 flex gap-3 items-start">
                            <svg class="w-5 h-5 text-red-400 mt-0.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                            </svg>
                            <p class="text-red-300/80 text-sm leading-relaxed">
                                Your selected device was disconnected. Reconnect it and scan again to continue.
                            </p>
                        </div>
                    {/if}

                    <!-- Devices or empty state -->
                    {#if detectedDevices.length > 0}
                        {#each detectedDevices as dev (dev.id)}
                            <button
                                onclick={() => selectDevice(dev)}
                                aria-pressed={selectedDeviceId === dev.id}
                                class="w-full text-left p-4 rounded-xl border transition-all duration-150 cursor-pointer
                                    {selectedDeviceId === dev.id
                                        ? 'border-rayhunter-blue bg-rayhunter-blue/5 ring-2 ring-rayhunter-blue/20'
                                        : 'border-slate-700 bg-slate-800/30 hover:border-slate-600 hover:bg-slate-800/50'}"
                            >
                                <div class="flex items-center justify-between">
                                    <span class="font-semibold text-white">{dev.display_name}</span>
                                    <div class="flex items-center gap-2" aria-hidden="true">
                                        <span class="text-[11px] font-semibold text-rayhunter-green bg-rayhunter-green/10 border border-rayhunter-green/20 px-2 py-0.5 rounded flex items-center gap-1">
                                            <span class="h-1.5 w-1.5 bg-rayhunter-green rounded-full animate-pulse"></span>
                                            Detected
                                        </span>
                                        {#if selectedDeviceId === dev.id}
                                            <span class="text-[11px] font-semibold text-rayhunter-blue bg-rayhunter-blue/10 border border-rayhunter-blue/30 px-2 py-0.5 rounded">
                                                ✓ Selected
                                            </span>
                                        {/if}
                                    </div>
                                </div>
                            </button>
                        {/each}
                    {:else if hasAttemptedDetection}
                        <div class="border border-dashed border-slate-800 rounded-xl p-8 text-center flex flex-col gap-1">
                            <p class="text-slate-500 text-sm">No compatible devices detected.</p>
                            <p class="text-slate-600 text-xs">Connect your device via USB or join its Wi-Fi network.</p>
                        </div>
                    {/if}

                </div>

                <!-- Next -->
                <div class="w-full flex justify-end pt-2">
                    <button
                        onclick={goNext}
                        disabled={selectedSubcommandIndex < 0}
                        class="px-8 py-3 rounded-xl font-semibold transition-all duration-150 flex items-center gap-2
                            {selectedSubcommandIndex >= 0
                                ? 'bg-rayhunter-blue hover:bg-rayhunter-dark-blue text-white cursor-pointer hover:scale-[1.02]'
                                : 'bg-slate-800 text-slate-600 cursor-not-allowed'}"
                    >
                        Next
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                        </svg>
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
                        Rayhunter has been successfully installed and started on your cellular hotspot device.
                    </p>
                </div>

                {#if selectedDevice?.admin_ip}
                    <div class="bg-slate-950 border border-slate-800 rounded-xl p-4 flex flex-col gap-2 text-left w-full text-sm">
                        <span class="font-bold text-slate-300">Accessing Rayhunter</span>
                        <p class="text-slate-400">
                            The web dashboard runs directly on the device. Connect your browser to the device to view cellular network alerts.
                        </p>
                        <span class="font-mono text-xs text-rayhunter-green mt-1">Dashboard Address: {dashboardUrl()}</span>
                    </div>
                {/if}

                <div class="flex flex-col gap-3 w-full mt-4">
                    {#if selectedDevice?.admin_ip}
                        <button
                            class="w-full py-3 rounded-xl font-bold bg-rayhunter-green text-slate-950 hover:bg-opacity-90 shadow-lg shadow-rayhunter-green/10 transition-all duration-200 cursor-pointer"
                            onclick={openDashboard}
                        >
                            Open Dashboard in Browser
                        </button>
                    {/if}
                    <button
                        class="w-full py-3 rounded-xl border border-slate-800 hover:border-slate-700 text-slate-300 font-semibold transition-colors duration-200 cursor-pointer"
                        onclick={() => getCurrentWindow().close()}
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

{#if showHelp}
    <!-- Backdrop -->
    <div 
        class="fixed inset-0 bg-black/60 backdrop-blur-xs z-40 transition-opacity duration-300"
        onclick={() => showHelp = false}
        aria-hidden="true"
    ></div>

    <!-- Drawer Content -->
    <div 
        class="fixed right-0 top-0 bottom-0 w-full max-w-md bg-slate-950 border-l border-slate-800 z-50 shadow-2xl p-6 overflow-y-auto flex flex-col gap-6 transition-transform duration-300"
    >
        <div class="flex justify-between items-center border-b border-slate-800 pb-4">
            <h3 class="text-lg font-bold text-white flex items-center gap-2">
                <span>❓</span> Setup & Help Instructions
            </h3>
            <button 
                class="text-slate-400 hover:text-white cursor-pointer text-sm font-semibold p-1.5 bg-slate-900 hover:bg-slate-800 rounded border border-slate-700 transition-colors"
                onclick={() => showHelp = false}
            >
                ✕ Close
            </button>
        </div>

        <div class="flex flex-col gap-4 text-sm text-slate-300 leading-relaxed">
            <p class="text-xs text-slate-400">Select your device below to view its specific setup steps:</p>
            
            <div class="flex flex-col gap-3">
                <details class="group bg-slate-900/60 border border-slate-800/80 rounded-xl p-3">
                    <summary class="font-semibold text-white cursor-pointer list-none flex justify-between items-center">
                        <span>Orbic RC400L (USB & Wi-Fi)</span>
                        <span class="text-xs text-slate-500 group-open:rotate-180 transition-transform">▼</span>
                    </summary>
                    <div class="mt-3 text-xs flex flex-col gap-2 pl-2 border-l-2 border-rayhunter-blue text-slate-400">
                        <p><strong class="text-slate-200">USB Installer:</strong> Recommended if you need ADB root access. Connect the Orbic via USB, power it on, and run the scan. Standard driver setup is required.</p>
                        <p><strong class="text-slate-200">Network Installer:</strong> Recommended for most usecases. Connect to the Orbic's Wi-Fi network (default gateway `192.168.1.1` or `192.168.0.1`), configure username (`admin`) and your admin portal password.</p>
                        <a href="https://efforg.github.io/rayhunter/orbic.html" target="_blank" class="text-rayhunter-blue hover:underline mt-1 block font-semibold">Full Orbic Setup Guide ➔</a>
                    </div>
                </details>

                <details class="group bg-slate-900/60 border border-slate-800/80 rounded-xl p-3">
                    <summary class="font-semibold text-white cursor-pointer list-none flex justify-between items-center">
                        <span>Moxee Hotspot</span>
                        <span class="text-xs text-slate-500 group-open:rotate-180 transition-transform">▼</span>
                    </summary>
                    <div class="mt-3 text-xs flex flex-col gap-2 pl-2 border-l-2 border-rayhunter-blue text-slate-400">
                        <p>1. Connect your PC to the Moxee Hotspot's Wi-Fi.</p>
                        <p>2. The admin interface is typically reached at `http://192.168.1.1` or `http://192.168.8.1`.</p>
                        <p>3. Enter your web portal password in the configuration step to authenticate and install.</p>
                        <a href="https://efforg.github.io/rayhunter/moxee.html" target="_blank" class="text-rayhunter-blue hover:underline mt-1 block font-semibold">Full Moxee Setup Guide ➔</a>
                    </div>
                </details>

                <details class="group bg-slate-900/60 border border-slate-800/80 rounded-xl p-3">
                    <summary class="font-semibold text-white cursor-pointer list-none flex justify-between items-center">
                        <span>TP-Link M7350</span>
                        <span class="text-xs text-slate-500 group-open:rotate-180 transition-transform">▼</span>
                    </summary>
                    <div class="mt-3 text-xs flex flex-col gap-2 pl-2 border-l-2 border-rayhunter-blue text-slate-400">
                        <p>1. Insert a formatted Micro SD card into the TP-Link hotspot (required for storage).</p>
                        <p>2. Connect to the TP-Link Wi-Fi network.</p>
                        <p>3. Run the installer. By default it communicates with the admin gateway at `http://192.168.0.1`.</p>
                        <a href="https://efforg.github.io/rayhunter/tplink-m7350.html" target="_blank" class="text-rayhunter-blue hover:underline mt-1 block font-semibold">Full TP-Link Setup Guide ➔</a>
                    </div>
                </details>

                <details class="group bg-slate-900/60 border border-slate-800/80 rounded-xl p-3">
                    <summary class="font-semibold text-white cursor-pointer list-none flex justify-between items-center">
                        <span>Wingtech CT2MHS01</span>
                        <span class="text-xs text-slate-500 group-open:rotate-180 transition-transform">▼</span>
                    </summary>
                    <div class="mt-3 text-xs flex flex-col gap-2 pl-2 border-l-2 border-rayhunter-blue text-slate-400">
                        <p>1. Connect your PC to the Wingtech hotspot via USB or Wi-Fi.</p>
                        <p>2. Configure the web admin password in the settings form to authorize root telnet access and deploy the daemon.</p>
                        <a href="https://efforg.github.io/rayhunter/wingtech.html" target="_blank" class="text-rayhunter-blue hover:underline mt-1 block font-semibold">Full Wingtech Setup Guide ➔</a>
                    </div>
                </details>

                <details class="group bg-slate-900/60 border border-slate-800/80 rounded-xl p-3">
                    <summary class="font-semibold text-white cursor-pointer list-none flex justify-between items-center">
                        <span>PinePhone Modem</span>
                        <span class="text-xs text-slate-500 group-open:rotate-180 transition-transform">▼</span>
                    </summary>
                    <div class="mt-3 text-xs flex flex-col gap-2 pl-2 border-l-2 border-rayhunter-blue text-slate-400">
                        <p>1. Ensure your PinePhone is booted and the Quectel modem is enabled.</p>
                        <p>2. Connect the phone via USB. The installer will claim the USB interface, enable ADB root access, and deploy the Rayhunter daemon.</p>
                    </div>
                </details>

                <details class="group bg-slate-900/60 border border-slate-800/80 rounded-xl p-3">
                    <summary class="font-semibold text-white cursor-pointer list-none flex justify-between items-center">
                        <span>T-Mobile TMOHS1 & Uz801</span>
                        <span class="text-xs text-slate-500 group-open:rotate-180 transition-transform">▼</span>
                    </summary>
                    <div class="mt-3 text-xs flex flex-col gap-2 pl-2 border-l-2 border-rayhunter-blue text-slate-400">
                        <p><strong class="text-slate-200">T-Mobile TMOHS1:</strong> Connect via Wi-Fi/USB and supply the web portal admin password.</p>
                        <p><strong class="text-slate-200">Uz801:</strong> Connect via USB. The installer will activate USB debug mode automatically and setup Rayhunter.</p>
                    </div>
                </details>
            </div>

            <div class="mt-6 border-t border-slate-800 pt-4 flex flex-col gap-2">
                <span class="font-bold text-white text-xs uppercase tracking-wider">Troubleshooting Tips:</span>
                <ul class="list-disc pl-4 text-xs text-slate-400 flex flex-col gap-1.5">
                    <li>If a USB device is not detected, ensure it is powered on and check your USB cable/ports.</li>
                    <li>If a Wi-Fi device is not detected, confirm you are connected to the hotspot's SSID.</li>
                    <li>Ensure no other software (such as a local ADB server or debugger) is actively using the USB interface.</li>
                </ul>
            </div>
        </div>
    </div>
{/if}

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
