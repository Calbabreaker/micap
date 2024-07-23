<script lang="ts">
    import { trackers } from "$lib/websocket";
    import TrackerPreview from "./tracker_preview.svelte";
    import TrackerStatus from "./tracker_status.svelte";
</script>

<div class="bg-neutral-700 p-4 shadow rounded mb-4">
    <h1 class="text-xl">Trackers</h1>
    {#each $trackers as tracker}
        <div class="bg-neutral-600 p-4 rounded shadow mt-4">
            <div class="text-sm text-slate-300">
                <TrackerStatus status={tracker.info.status} />
                {#if tracker.info.latency_ms}
                    <span>
                        {tracker.info.latency_ms}ms
                    </span>
                {/if}
                {#if tracker.info.level != null}
                    <span>
                        {Math.round(tracker.info.level * 100)}%
                    </span>
                {/if}
            </div>
            <span>{tracker.info.config.name}</span>
            <TrackerPreview data={tracker.data} />
        </div>
    {/each}
</div>
