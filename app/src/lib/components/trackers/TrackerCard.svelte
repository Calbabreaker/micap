<script lang="ts">
    import { trackers } from "$lib/websocket";
    import TrackerInfoDisplay from "./TrackerInfoDisplay.svelte";
    import TrackerControl from "./TrackerControl.svelte";
    import TrackerPreview from "./TrackerPreview.svelte";

    export let id: string;

    $: tracker = $trackers[id]!;

    // Highlight border when there is movement
    $: brightness = Math.min(
        Math.hypot(...tracker.data?.acceleration) * 50,
        50,
    );

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
            <p>ID: {id}</p>
            <TrackerPreview data={tracker.data} />
        </div>
    {/if}
</div>
