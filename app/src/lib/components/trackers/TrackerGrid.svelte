<script lang="ts">
    import { trackers } from "$lib/websocket";
    import TrackerCard from "./TrackerCard.svelte";
</script>

<div class="flex flex-wrap gap-2 justify-center">
    {#each Object.keys($trackers) as id}
        <TrackerCard {id} />
    {/each}
    {#if Object.keys($trackers).length == 0}
        <span class="text-neutral-400">No trackers connected yet.</span>
    {/if}
</div>
