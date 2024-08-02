<script lang="ts">
    import { type Tracker } from "$lib/websocket";
    import TrackerPreview from "./TrackerPreview.svelte";
    import TrackerInfoDisplay from "./TrackerInfoDisplay.svelte";
    import TrashIcon from "./icons/TrashIcon.svelte";
    import MangnifyingGlassIcon from "./icons/MangnifyingGlassIcon.svelte";
    import PencilIcon from "./icons/PencilIcon.svelte";

    export let tracker: Tracker;
    export let onRemove: () => void;
    export let onConfigEdit: () => void;

    let showPreview = false;
</script>

<div class="bg-neutral-600 p-4 rounded shadow w-fit border border-neutral-900">
    <TrackerInfoDisplay info={tracker.info} />
    <p>{tracker.info.config.name}</p>
    <div class="flex gap-1 mt-2">
        <button class="btn-icon" on:click={onRemove}>
            <TrashIcon />
        </button>
        <button class="btn-icon" on:click={() => (showPreview = !showPreview)}>
            <MangnifyingGlassIcon />
        </button>
        <button class="btn-icon" on:click={onConfigEdit}>
            <PencilIcon />
        </button>
    </div>
    {#if showPreview && tracker.data}
        <hr class="my-4" />
        <TrackerPreview data={tracker.data} />
    {/if}
</div>
