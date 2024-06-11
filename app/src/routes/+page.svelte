<script lang="ts">
    import { websocket, trackers, TrackerStatus } from "$lib/websocket";
    import WifiCard from "../components/wifi_card.svelte";
</script>

<h1 class="text-white text-lg">Welcome to SvelteKit</h1>
<div>
    <div>
        <p>Clients:</p>
        {#each $trackers as tracker}
            {tracker.id}
            {#if tracker.status == TrackerStatus.Ok}
                <p>Ok</p>
            {/if}
        {/each}
    </div>
    <WifiCard />
    <button
        class="btn btn-primary"
        on:click={() => {
            if (confirm("Are you sure?")) {
                $websocket.send(JSON.stringify({ type: "FactoryReset" }));
            }
        }}
    >
        Factory reset
    </button>
</div>
