<script lang="ts">
    import { trackers } from "$lib/websocket";
    import TrackerCard from "./tracker_card.svelte";
</script>

<div class="bg-neutral-700 p-4 shadow rounded mb-4 w-full">
    <h1 class="text-xl text-center">Trackers</h1>
    <div class="flex flex-wrap gap-2 mt-4 justify-center">
        {#each $trackers as tracker}
            <TrackerCard {tracker} />
        {/each}
        {#if $trackers.length == 0}
            <span class="text-neutral-400">No trackers connected yet.</span>
        {/if}
    </div>
</div>
