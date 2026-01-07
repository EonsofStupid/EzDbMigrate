<script>
    import { Lock } from "lucide-svelte";
</script>

<div
    class="h-screen bg-black text-red-500 flex flex-col items-center justify-center font-mono p-4"
>
    <Lock class="w-16 h-16 mb-6 animate-pulse" />
    <h1 class="text-2xl font-black uppercase tracking-[0.3em] mb-4">
        Security Clearance Required
    </h1>
    <p class="text-slate-500 text-sm mb-8 text-center max-w-sm">
        This area is restricted to authorized personnel only. Verify clearance
        level to proceed.
    </p>

    <button
        class="bg-red-950/50 border border-red-800 text-red-400 px-8 py-3 uppercase tracking-widest hover:bg-red-900 transition-colors"
    >
        Authenticate via YouTube
    </button>
</div>
