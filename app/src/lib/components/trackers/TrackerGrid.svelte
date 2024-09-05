<script lang="ts">
    import {
        trackers,
        sendWebsocket,
        globalConfig,
        setConfig,
        type TrackerConfig,
    } from "$lib/websocket";
    import TrackerCard from "./TrackerCard.svelte";

    function removeTracker(id: string) {
        const message =
            "Are you sure you want to remove the tracker? This will also prevent the device from connecting to the server once all the associated trackers are removed as well.";
        if (!confirm(message)) {
            return;
        }

        sendWebsocket({
            type: "RemoveTracker",
            id,
        });

        trackers.update((trackers) => {
            delete trackers[id];
            return trackers;
        });
    }

    function editTrackerConfig(id: string, config: TrackerConfig) {
        setConfig((globalConfig) => {
            globalConfig.trackers[id] = config;
        });
    }
</script>

<div class="flex flex-wrap gap-2 mt-4 justify-center">
    {#each Object.entries($trackers) as [id, tracker]}
        <TrackerCard
            {tracker}
            config={$globalConfig.trackers[id]}
            onRemove={() => removeTracker(id)}
            onConfigEdit={(config) => editTrackerConfig(id, config)}
        />
    {/each}
    {#if Object.keys($trackers).length == 0}
        <span class="text-neutral-400">No trackers connected yet.</span>
    {/if}
</div>
