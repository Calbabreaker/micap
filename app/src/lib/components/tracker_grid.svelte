<script lang="ts">
    import { trackers } from "$lib/websocket";
    import TrackerCard from "./tracker_card.svelte";
</script>

<div class="bg-neutral-700 p-4 shadow rounded mb-4">
    <h1 class="text-xl">Trackers</h1>
    {#each $trackers as tracker}
        <TrackerCard {tracker} />
    {/each}
</div>
