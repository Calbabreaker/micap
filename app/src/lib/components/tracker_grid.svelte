<script lang="ts">
    import { trackers, sendWebsocket } from "$lib/websocket";
    import TrackerCard from "./tracker_card.svelte";

    function removeTracker(index: number) {
        if (
            !confirm(
                "Are you sure you want to remove the tracker? This will also prevent the device from connecting to the server once all the ascociated trackers are removed as well.",
            )
        ) {
            return;
        }

        sendWebsocket({
            type: "RemoveTracker",
            index,
        });
    }

    function editConfig(index: number) {
        const name = prompt("Enter the new name: ");
        if (!name) {
            return;
        }

        sendWebsocket({
            type: "UpdateTrackerConfig",
            index,
            config: {
                ...$trackers[index].info.config,
                name,
            },
        });
    }
</script>

<div class="bg-neutral-700 p-4 shadow rounded mb-4 w-full">
    <h1 class="text-xl text-center">Trackers</h1>
    <div class="flex flex-wrap gap-2 mt-4 justify-center">
        {#each $trackers as tracker, index}
            {#if !tracker.info.removed}
                <TrackerCard
                    {tracker}
                    onRemove={() => removeTracker(index)}
                    onConfigEdit={() => editConfig(index)}
                />
            {/if}
        {/each}
        {#if $trackers.length == 0}
            <span class="text-neutral-400">No trackers connected yet.</span>
        {/if}
    </div>
</div>
