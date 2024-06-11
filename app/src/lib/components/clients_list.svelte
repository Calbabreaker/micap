<script lang="ts">
    import { trackers } from "$lib/websocket";
</script>

<div class="bg-neutral-700 p-4 shadow rounded mb-4">
    <h1 class="text-xl">Clients</h1>
    {#each $trackers as tracker}
        <span>{tracker.info.id}</span>
        <span>{tracker.info.status}</span>
    {/each}
</div>
