<script lang="ts">
    import type { Tracker } from "$lib/websocket";
    import TrackerPreview from "./tracker_preview.svelte";
    import TrackerInfoDisplay from "./tracker_info_display.svelte";

    export let tracker: Tracker;

    let showPreview = false;
</script>

<div class="bg-neutral-600 p-4 rounded shadow w-fit border border-neutral-900">
    <TrackerInfoDisplay info={tracker.info} />
    <p>{tracker.info.config.name}</p>
    <button
        class="btn btn-small mt-2 w-full"
        on:click={() => (showPreview = !showPreview)}
    >
        Toggle Preview
    </button>
    {#if showPreview}
        <hr class="my-4" />
        <TrackerPreview data={tracker.data} />
    {/if}
</div>
