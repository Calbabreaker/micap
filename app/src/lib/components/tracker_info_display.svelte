<script lang="ts">
    import type { TrackerInfo } from "$lib/websocket";

    export let info: TrackerInfo;
</script>

<div class="text-sm text-slate-300">
    <span
        class:text-red-400={info.status == "Error"}
        class:text-green-400={info.status == "Ok"}
        class:text-yellow-400={info.status == "TimedOut"}
        class:text-gray-400={info.status == "Off"}
    >
        {info.status}
    </span>
    {#if info.latency_ms}
        <span>
            {info.latency_ms}ms
        </span>
    {/if}
    {#if info.battery_level}
        <span>
            {Math.round(info.battery_level * 100)}%
        </span>
    {:else}
        <span>No battery</span>
    {/if}
</div>
