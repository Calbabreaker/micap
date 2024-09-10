<script lang="ts">
    import { trackers, sendWebsocket, setConfig } from "$lib/websocket";
    import TrackerCard from "./TrackerCard.svelte";
    import { confirmPopup } from "$lib/toast";

    async function removeTracker(id: string) {
        const message =
            "This will also prevent the device from connecting to the server once all the associated trackers are removed as well.";
        await confirmPopup(
            "Are you sure you want to remove the tracker?",
            message,
        );
        sendWebsocket({
            type: "RemoveTracker",
            id,
        });

        trackers.update((trackers) => {
            delete trackers[id];
            return trackers;
        });
    }
</script>

<div class="flex flex-wrap gap-2 mt-4 justify-center">
    {#each Object.entries($trackers) as [id, tracker]}
        <TrackerCard
            {tracker}
            {id}
            onRemove={() => removeTracker(id)}
            onConfigEdit={() => setConfig(() => {})}
        />
    {/each}
    {#if Object.keys($trackers).length == 0}
        <span class="text-neutral-400">No trackers connected yet.</span>
    {/if}
</div>
