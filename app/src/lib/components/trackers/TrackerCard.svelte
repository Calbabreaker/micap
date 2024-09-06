<script lang="ts">
    import {
        type Tracker,
        trackerLocations,
        globalConfig,
    } from "$lib/websocket";
    import TrackerPreview from "./TrackerPreview.svelte";
    import TrackerInfoDisplay from "./TrackerInfoDisplay.svelte";
    import TrashIcon from "../icons/TrashIcon.svelte";
    import MangnifyingGlassIcon from "../icons/MangnifyingGlassIcon.svelte";
    import PencilIcon from "../icons/PencilIcon.svelte";

    export let tracker: Tracker;
    export let id: string;
    let config = $globalConfig!.trackers[id];

    export let onRemove: () => void;
    export let onConfigEdit: () => void;
    let brightness: number;

    $: if (tracker.data) {
        // Show when there is movement
        brightness = Math.min(
            Math.hypot(...tracker.data?.acceleration) * 50,
            50,
        );
    }

    let showPreview = false;
</script>

<div
    style:border-color={`hsl(0, 0%, ${brightness}%)`}
    class="bg-neutral-600 p-4 rounded shadow w-fit border"
>
    <TrackerInfoDisplay info={tracker.info} />
    <p>{config.name}</p>
    <div class="flex gap-1 mt-2">
        <button class="btn-icon" on:click={onRemove}>
            <TrashIcon />
        </button>
        <button class="btn-icon" on:click={() => (showPreview = !showPreview)}>
            <MangnifyingGlassIcon />
        </button>
        <button
            class="btn-icon"
            on:click={() => {
                const name = prompt("Enter the new name: ");
                if (name) {
                    config.name = name;
                    onConfigEdit();
                }
            }}
        >
            <PencilIcon />
        </button>
    </div>
    <select
        class="text-neutral-700 px-1 mt-2 bg-white"
        value={config.location}
        on:change={(e) => {
            config.location = e.currentTarget.value;
            onConfigEdit();
        }}
    >
        {#each trackerLocations as location}
            <option value={location}>{location}</option>
        {/each}
    </select>
    {#if showPreview}
        <hr class="my-4" />
        <TrackerPreview {id} />
    {/if}
</div>
