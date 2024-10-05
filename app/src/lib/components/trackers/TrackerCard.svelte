<script lang="ts">
    import { trackers } from "$lib/websocket";
    import TrackerPreview from "./TrackerPreview.svelte";
    import TrackerInfoDisplay from "./TrackerInfoDisplay.svelte";
    import TrackerControl from "./TrackerControl.svelte";

    export let id: string;

    $: tracker = $trackers[id]!;

    // Highlight border when there is movement
    $: brightness = Math.hypot(...tracker.data.acceleration) > 2 ? 60 : 10;

    let showInspect = false;
</script>

<div
    style:border-color={`hsl(0, 0%, ${brightness}%)`}
    class="bg-neutral-600 p-4 rounded shadow w-fit border"
>
    <TrackerInfoDisplay info={tracker.info} />
    <TrackerControl bind:showInspect {id} />
    {#if showInspect}
        <hr class="my-4" />
        <div class="text-sm text-neutral-300">
            <p>Address: {tracker.info.address}</p>
            <p>ID: {tracker.info.address}</p>
            <TrackerPreview data={tracker.data} />
        </div>
    {/if}
</div>
