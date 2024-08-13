<script lang="ts">
    import {
        trackers,
        sendWebsocket,
        type TrackerConfig,
    } from "$lib/websocket";
    import TrackerCard from "./TrackerCard.svelte";

    function removeTracker(id: string) {
        if (
            !confirm(
                "Are you sure you want to remove the tracker? This will also prevent the device from connecting to the server once all the ascociated trackers are removed as well.",
            )
        ) {
            return;
        }

        sendWebsocket({
            type: "RemoveTracker",
            id,
        });
    }

    function editConfig(id: string, config: TrackerConfig) {
        sendWebsocket({
            type: "UpdateTrackerConfig",
            id,
            config,
        });
    }
</script>

<div class="bg-neutral-700 p-4 shadow rounded mb-4 w-full">
    <h1 class="text-xl text-center">Trackers</h1>
    <div class="flex flex-wrap gap-2 mt-4 justify-center">
        {#each $trackers.entries() as [id, tracker]}
            <TrackerCard
                {tracker}
                onRemove={() => removeTracker(id)}
                onConfigEdit={(config) => editConfig(id, config)}
            />
        {/each}
        {#if $trackers.size == 0}
            <span class="text-neutral-400">No trackers connected yet.</span>
        {/if}
    </div>
</div>
