<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import {
        Database,
        Lock,
        Save,
        Skull,
        CheckCircle2,
        AlertOctagon,
        Terminal,
        Activity,
        Server,
        ArrowRight,
        DownloadCloud,
        UploadCloud,
        X,
        Cpu,
        Disc,
        Zap,
    } from "lucide-svelte";
    import { fly, fade } from "svelte/transition";
    import { onMount } from "svelte";

    // LOGGING
    let logs = $state<string[]>([]);
    let logContainer: HTMLDivElement;

    // CONFIG
    let activeTab = $state<"BACKUP" | "RESTORE">("BACKUP");
    let sourceUrl = $state("");
    let sourceKey = $state("");
    let destUrl = $state("");
    let destKey = $state("");
    let backupFile = $state("");
    let flightRecorderEnabled = $state(false);

    // STATE
    let busyState = $state<
        "IDLE" | "LINKING" | "BACKING_UP" | "RESTORING" | "INSTALLING"
    >("IDLE");
    let showProgressModal = $state(false);
    let driverStatus = $state<"MISSING" | "READY">("MISSING");

    // PROGRESS TRACKING
    type ProgressStage = {
        name: string;
        status: "PENDING" | "RUNNING" | "DONE" | "ERROR";
    };
    let progressStages = $state<ProgressStage[]>([
        { name: "DATABASE", status: "PENDING" },
        { name: "STORAGE", status: "PENDING" },
        { name: "FUNCTIONS", status: "PENDING" },
        { name: "AUTH", status: "PENDING" },
    ]);

    function addLog(msg: string) {
        const timestamp = new Date().toLocaleTimeString("en-US", {
            hour12: false,
        });
        logs = [...logs, `[${timestamp}] ${msg}`];
        setTimeout(() => {
            if (logContainer)
                logContainer.scrollTop = logContainer.scrollHeight;
        }, 10);
    }

    onMount(() => {
        const unlistenLog = listen<string>("log", (e) => addLog(e.payload));

        // Listen for Granular Progress
        const unlistenProgress = listen<{ stage: string; status: string }>(
            "progress_update",
            (e) => {
                const idx = progressStages.findIndex(
                    (s) => s.name === e.payload.stage,
                );
                if (idx !== -1) {
                    progressStages[idx].status = e.payload.status as any;
                }
            },
        );

        // Initial Driver Check
        checkDrivers();

        return () => {
            unlistenLog.then((f) => f());
            unlistenProgress.then((f) => f());
        };
    });

    async function checkDrivers() {
        try {
            await invoke("check_driver_status");
            driverStatus = "READY";
        } catch (e) {
            driverStatus = "MISSING";
            addLog("ALERT: POSTGRES DRIVERS MISSING. INITIATE HYDRATION.");
        }
    }

    async function installDrivers() {
        if (busyState !== "IDLE") return;
        busyState = "INSTALLING";
        try {
            await invoke("install_drivers");
            driverStatus = "READY";
            addLog("DRIVERS MOUNTED SUCCESSFULLY.");
        } catch (e) {
            addLog(`INSTALL ERROR: ${e}`);
        } finally {
            busyState = "IDLE";
        }
    }

    async function runBackupProcess() {
        if (driverStatus !== "READY") {
            addLog("ERROR: CANNOT BACKUP. DRIVERS MISSING.");
            return;
        }
        if (!sourceUrl || !sourceKey) {
            addLog("ERROR: MISSING SOURCE CREDENTIALS");
            return;
        }

        busyState = "BACKING_UP";
        showProgressModal = true;
        progressStages.forEach((s) => (s.status = "PENDING")); // Reset

        try {
            addLog(">>> INITIATING FULL SYSTEM BACKUP");
            // Verify First
            await invoke("verify_connection", {
                url: sourceUrl,
                key: sourceKey,
            });

            // Run Backup
            await invoke("backup_database", { url: sourceUrl });

            addLog("BACKUP SEQUENCE COMPLETE.");
        } catch (e) {
            addLog(`BACKUP FAILED: ${e}`);
        } finally {
            busyState = "IDLE";
            // Keep modal open for a moment to show success
            setTimeout(() => (showProgressModal = false), 3000);
        }
    }

    async function runRestoreProcess() {
        if (!destUrl || !destKey || !backupFile) {
            addLog("ERROR: MISSING TARGET CREDENTIALS OR BACKUP FILE");
            return;
        }
        addLog("RESTORE PROTOCOL NOT YET IMPLEMENTED IN DEMO.");
    }
</script>

<div
    class="min-h-screen bg-black text-green-500 font-mono p-4 md:p-8 flex items-center justify-center relative overflow-hidden select-none"
>
    <!-- SCANLINE OVERLAY -->
    <div
        class="absolute inset-0 bg-[linear-gradient(rgba(18,16,16,0)_50%,rgba(0,0,0,0.25)_50%),linear-gradient(90deg,rgba(255,0,0,0.06),rgba(0,255,0,0.02),rgba(0,0,255,0.06))] z-0 pointer-events-none bg-[length:100%_4px,3px_100%]"
    ></div>

    <!-- MAIN COCKPIT -->
    <div
        class="relative z-10 w-full max-w-6xl grid grid-cols-1 lg:grid-cols-2 gap-8 border-4 border-slate-800 bg-black/90 p-8 rounded-xl shadow-[0_0_50px_rgba(34,197,94,0.1)]"
    >
        <!-- LEFT: OPERATIONAL CONSOLE -->
        <div class="space-y-6">
            <header
                class="border-b border-green-900 pb-4 flex justify-between items-start"
            >
                <div>
                    <h1
                        class="text-3xl font-black uppercase tracking-[0.1em] text-white flex items-center gap-3"
                    >
                        <Terminal class="w-8 h-8 text-green-500" />
                        DevPulse
                        <span
                            class="text-xs text-green-700 ml-2 tracking-normal font-normal"
                            >v3.1 (PULSE)</span
                        >
                    </h1>
                    <p
                        class="text-[10px] text-green-800 mt-1 uppercase tracking-widest"
                    >
                        Postgres Migration Engine
                    </p>
                </div>

                <!-- FLIGHT RECORDER TOGGLE -->
                <button
                    onclick={() =>
                        (flightRecorderEnabled = !flightRecorderEnabled)}
                    class="flex items-center gap-2 px-3 py-1 border rounded text-[10px] uppercase tracking-widest transition-all {flightRecorderEnabled
                        ? 'border-red-500 bg-red-950/30 text-red-200'
                        : 'border-slate-800 text-slate-600'}"
                >
                    {#if flightRecorderEnabled}
                        <div
                            class="w-2 h-2 rounded-full bg-red-500 animate-pulse"
                        ></div>
                         REC
                    {:else}
                        <div class="w-2 h-2 rounded-full bg-slate-700"></div>
                         OFF
                    {/if}
                </button>
            </header>

            <!-- DRIVER STATUS (MECHANICAL ARM) -->
            <div
                class="border border-slate-800 bg-slate-900/40 p-4 rounded flex items-center justify-between gap-4"
            >
                <div class="flex items-center gap-3">
                    <Cpu
                        class="w-6 h-6 {driverStatus === 'READY'
                            ? 'text-green-500'
                            : 'text-yellow-500 animate-pulse'}"
                    />
                    <div>
                        <h3
                            class="text-xs uppercase tracking-widest text-slate-400"
                        >
                            Driver Status (Pulse Pack)
                        </h3>
                        <div
                            class="font-bold {driverStatus === 'READY'
                                ? 'text-white'
                                : 'text-yellow-500'}"
                        >
                            {driverStatus === "READY"
                                ? "DRIVERS MOUNTED"
                                : "DRIVERS MISSING"}
                        </div>
                    </div>
                </div>

                {#if driverStatus === "MISSING"}
                    <button
                        onclick={installDrivers}
                        disabled={busyState === "INSTALLING"}
                        class="px-4 py-2 bg-yellow-900/20 border border-yellow-600 text-yellow-500 text-xs font-bold uppercase tracking-widest hover:bg-yellow-900/40 transition-all flex items-center gap-2"
                    >
                        {#if busyState === "INSTALLING"}
                            <Zap class="w-4 h-4 animate-spin" /> INSTALLING...
                        {:else}
                            <DownloadCloud class="w-4 h-4" /> INSTALL DRIVERS
                        {/if}
                    </button>
                {/if}
            </div>

            <!-- TABS -->
            <div class="flex gap-2 border-b border-slate-800">
                <button
                    onclick={() => (activeTab = "BACKUP")}
                    class="px-6 py-2 text-xs font-bold uppercase tracking-wider border-t border-x rounded-t transition-all {activeTab ===
                    'BACKUP'
                        ? 'bg-green-950/10 text-green-400 border-green-900/50'
                        : 'bg-transparent text-slate-600 border-transparent hover:text-slate-400'}"
                >
                    BACKUP
                </button>
                <button
                    onclick={() => (activeTab = "RESTORE")}
                    class="px-6 py-2 text-xs font-bold uppercase tracking-wider border-t border-x rounded-t transition-all {activeTab ===
                    'RESTORE'
                        ? 'bg-blue-950/10 text-blue-400 border-blue-900/50'
                        : 'bg-transparent text-slate-600 border-transparent hover:text-slate-400'}"
                >
                    RESTORE
                </button>
            </div>

            {#if activeTab === "BACKUP"}
                <!-- BACKUP MODULE -->
                <div class="space-y-6 relative" in:fade={{ duration: 200 }}>
                    {#if driverStatus !== "READY"}
                        <div
                            class="absolute inset-0 bg-black/60 z-20 flex items-center justify-center backdrop-blur-[1px]"
                        >
                            <span
                                class="text-xs text-yellow-500 uppercase tracking-widest bg-black px-3 py-1 border border-yellow-900"
                                >Drivers Required</span
                            >
                        </div>
                    {/if}

                    <div
                        class="bg-green-950/5 border border-green-900/30 p-6 rounded-b rounded-tr"
                    >
                        <h2
                            class="text-green-600 font-bold uppercase tracking-widest flex items-center gap-2 mb-4 text-xs"
                        >
                            <Database class="w-4 h-4" /> Source Parameters
                        </h2>

                        <div class="space-y-3">
                            <div>
                                <input
                                    type="text"
                                    bind:value={sourceUrl}
                                    class="w-full bg-black border border-green-900/50 text-green-400 text-xs p-3 focus:border-green-500 outline-none rounded transition-colors"
                                    placeholder="Source Project URL (https://...)"
                                />
                            </div>
                            <div>
                                <input
                                    type="password"
                                    bind:value={sourceKey}
                                    class="w-full bg-black border border-green-900/50 text-green-400 text-xs p-3 focus:border-green-500 outline-none rounded transition-colors"
                                    placeholder="Service Role Key (sbp_...)"
                                />
                            </div>
                        </div>

                        <div
                            class="mt-6 p-3 bg-slate-900/50 rounded border border-slate-800"
                        >
                            <h3
                                class="text-[10px] text-slate-500 uppercase mb-2 tracking-widest"
                            >
                                Included In Snapshot:
                            </h3>
                            <ul
                                class="grid grid-cols-2 gap-2 text-[10px] text-slate-400"
                            >
                                <li class="flex items-center gap-2">
                                    <div
                                        class="w-1 h-1 bg-green-500 rounded-full"
                                    ></div>
                                     Database Schema & Data
                                </li>
                                <li class="flex items-center gap-2">
                                    <div
                                        class="w-1 h-1 bg-green-500 rounded-full"
                                    ></div>
                                     Storage Buckets & Files
                                </li>
                                <li class="flex items-center gap-2">
                                    <div
                                        class="w-1 h-1 bg-green-500 rounded-full"
                                    ></div>
                                     Edge Functions
                                </li>
                                <li class="flex items-center gap-2">
                                    <div
                                        class="w-1 h-1 bg-green-500 rounded-full"
                                    ></div>
                                     Auth Users & Config
                                </li>
                            </ul>
                        </div>
                    </div>

                    <button
                        onclick={runBackupProcess}
                        disabled={busyState !== "IDLE" ||
                            driverStatus !== "READY"}
                        class="w-full py-6 bg-green-900/10 hover:bg-green-900/30 text-green-400 font-black text-xl uppercase tracking-[0.2em] border-2 border-green-800 hover:border-green-500 hover:shadow-[0_0_30px_rgba(34,197,94,0.1)] transition-all flex items-center justify-center gap-3 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        {#if busyState === "BACKING_UP"}
                            <Activity class="w-6 h-6 animate-pulse" /> ENGAGED
                        {:else}
                            <Disc class="w-6 h-6" /> INITIATE BACKUP
                        {/if}
                    </button>
                </div>
            {:else}
                <!-- RESTORE MODULE -->
                <div class="space-y-6" in:fade={{ duration: 200 }}>
                    <div
                        class="bg-blue-950/5 border border-blue-900/30 p-6 rounded-b rounded-tr"
                    >
                        <h2
                            class="text-blue-600 font-bold uppercase tracking-widest flex items-center gap-2 mb-4 text-xs"
                        >
                            <Server class="w-4 h-4" /> Target Parameters
                        </h2>

                        <div class="space-y-3">
                            <div>
                                <input
                                    type="text"
                                    bind:value={destUrl}
                                    class="w-full bg-black border border-blue-900/50 text-blue-400 text-xs p-3 focus:border-blue-500 outline-none rounded"
                                    placeholder="Target Project URL"
                                />
                            </div>
                            <div>
                                <input
                                    type="password"
                                    bind:value={destKey}
                                    class="w-full bg-black border border-blue-900/50 text-blue-400 text-xs p-3 focus:border-blue-500 outline-none rounded"
                                    placeholder="Service Role Key"
                                />
                            </div>
                            <div>
                                <div class="flex gap-2">
                                    <input
                                        type="text"
                                        bind:value={backupFile}
                                        class="flex-1 bg-black border border-blue-900/50 text-blue-400 text-xs p-3 focus:border-blue-500 outline-none rounded"
                                        placeholder="Select Local Backup File..."
                                    />
                                    <button
                                        class="px-4 bg-blue-900/10 border border-blue-800 text-blue-400 text-xs hover:bg-blue-900/30 transition-colors uppercase font-bold"
                                        >BROWSE</button
                                    >
                                </div>
                            </div>
                        </div>
                    </div>

                    <button
                        disabled
                        class="w-full py-6 bg-slate-900/50 text-slate-600 font-black text-xl uppercase tracking-[0.2em] border-2 border-slate-800 cursor-not-allowed flex items-center justify-center gap-3"
                    >
                        <Lock class="w-5 h-5" /> RESTORE LOCKED (DEMO)
                    </button>
                </div>
            {/if}
        </div>

        <!-- RIGHT: LOGS -->
        <div class="flex flex-col h-full min-h-[500px]">
            <div
                class="flex-1 bg-slate-950 border border-slate-800 rounded p-4 relative overflow-hidden font-mono text-sm group"
            >
                <div
                    class="absolute top-2 right-2 text-[10px] text-slate-700 group-hover:text-slate-500 transition-colors"
                >
                    FLIGHT RECORDER
                </div>
                <div
                    class="h-full overflow-y-auto pr-2 custom-scrollbar"
                    bind:this={logContainer}
                >
                    {#each logs as log, i (i)}
                        <div
                            class="mb-1 border-l-2 border-slate-800 pl-2 text-slate-400 text-xs font-mono"
                            in:fly={{ x: -10, duration: 200 }}
                        >
                            {log}
                        </div>
                    {/each}
                </div>
            </div>
        </div>
    </div>

    <!-- LAYMAN PROGRESS MODAL -->
    {#if showProgressModal}
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/90 backdrop-blur-sm"
            transition:fade
        >
            <div
                class="bg-black border-2 border-green-500 p-8 rounded-lg max-w-lg w-full shadow-[0_0_100px_rgba(34,197,94,0.2)]"
            >
                <h3
                    class="text-2xl text-green-500 font-black uppercase tracking-widest mb-6 flex items-center justify-between"
                >
                    <span>Operation Active</span>
                    <Activity class="w-6 h-6 animate-pulse" />
                </h3>

                <div class="space-y-4">
                    {#each progressStages as stage}
                        <div
                            class="flex items-center justify-between p-3 border border-slate-800 bg-slate-900/50 rounded"
                        >
                            <span class="text-sm font-bold text-slate-300"
                                >{stage.name}</span
                            >
                            <div class="flex items-center gap-2">
                                {#if stage.status === "PENDING"}
                                    <span
                                        class="text-[10px] text-slate-600 uppercase"
                                        >WAITING</span
                                    >
                                    <div
                                        class="w-2 h-2 rounded-full bg-slate-700"
                                    ></div>
                                {:else if stage.status === "RUNNING"}
                                    <span
                                        class="text-[10px] text-yellow-400 uppercase animate-pulse"
                                        >PROCESSING</span
                                    >
                                    <div
                                        class="w-2 h-2 rounded-full bg-yellow-500 animate-ping"
                                    ></div>
                                {:else if stage.status === "DONE"}
                                    <span
                                        class="text-[10px] text-green-400 uppercase"
                                        >COMPLETE</span
                                    >
                                    <CheckCircle2
                                        class="w-4 h-4 text-green-500"
                                    />
                                {/if}
                            </div>
                        </div>
                    {/each}
                </div>

                <div class="mt-8 text-center">
                    <p
                        class="text-xs text-slate-500 uppercase tracking-widest animate-pulse"
                    >
                        Please do not close this window...
                    </p>
                </div>
            </div>
        </div>
    {/if}
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: #000;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: #333;
    }
</style>
