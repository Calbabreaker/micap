<script lang="ts">
    import { trackers } from "$lib/websocket";
    import TrackerPreview from "./tracker_preview.svelte";

    function formatArray(array: number[]) {
        return array.map((value) => value.toFixed(2)).join(", ");
    }
</script>

<div class="bg-neutral-700 p-4 shadow rounded mb-4">
    <h1 class="text-xl">Trackers</h1>
    {#each $trackers as tracker}
        <div class="bg-neutral-600 p-4 rounded shadow mt-4">
            <span>"{tracker.info.config.name}"</span>
            <span>{tracker.info.status}</span>
            {#if tracker.data && tracker.info.status == "Ok"}
                <p>Orientation: {formatArray(tracker.data.orientation)}</p>
                <p>Acceleration: {formatArray(tracker.data.acceleration)}</p>
                <div class="w-96 h-96 bg-black mt-2">
                    <TrackerPreview data={tracker.data} />
                </div>
            {/if}
        </div>
    {/each}
</div>
