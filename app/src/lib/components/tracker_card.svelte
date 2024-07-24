<script lang="ts">
    import type { Tracker } from "$lib/websocket";
    import TrackerPreview from "./tracker_preview.svelte";
    import TrackerStatus from "./tracker_status.svelte";

    export let tracker: Tracker;
</script>

<div class="bg-neutral-600 p-4 rounded shadow mt-4 w-fit">
    <div class="text-sm text-slate-300">
        <TrackerStatus status={tracker.info.status} />
        {#if tracker.info.latency_ms}
            <span>
                {tracker.info.latency_ms}ms
            </span>
        {/if}
        {#if tracker.info.level}
            <span>
                {Math.round(tracker.info.level * 100)}%
            </span>
        {:else}
            <span>No battery</span>
        {/if}
    </div>
    <span>{tracker.info.config.name}</span>
    <button>Show Preview</button>
    <TrackerPreview data={tracker.data} />
</div>
