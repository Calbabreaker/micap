<script lang="ts">
    import {
        trackers,
        globalConfig,
        editTrackerConfig,
        removeTracker,
    } from "$lib/websocket";
    import { promptPopup } from "$lib/toast";
    import TrackerInspect from "./TrackerInspect.svelte";
    import TrackerInfoDisplay from "./TrackerInfoDisplay.svelte";
    import TrashIcon from "../icons/TrashIcon.svelte";
    import MangnifyingGlassIcon from "../icons/MangnifyingGlassIcon.svelte";
    import PencilIcon from "../icons/PencilIcon.svelte";

    export let id: string;

    export const commonBoneLocations = [
        "Hip",
        "LeftUpperLeg",
        "RightUpperLeg",
        "LeftLowerLeg",
        "RightLowerLeg",
        "LeftFoot",
        "RightFoot",
        "Spine",
        "Chest",
        "Neck",
        "LeftUpperArm",
        "RightUpperArm",
        "LeftLowerArm",
        "RightLowerArm",
        "LeftHand",
        "RightHand",
    ];

    $: tracker = $trackers[id];
    $: config = $globalConfig?.trackers[id];

    // Highlight border when there is movement
    $: brightness = Math.hypot(...tracker.data.acceleration) > 2 ? 60 : 10;

    let showInspect = false;
</script>

<div
    style:border-color={`hsl(0, 0%, ${brightness}%)`}
    class="bg-neutral-600 p-4 rounded shadow w-fit border"
>
    <TrackerInfoDisplay info={tracker.info} />
    <p>{config?.name || id}</p>
    <div class="flex gap-1 mt-2">
        <button class="btn-icon" on:click={() => removeTracker(id)}>
            <TrashIcon />
        </button>
        <button class="btn-icon" on:click={() => (showInspect = !showInspect)}>
            <MangnifyingGlassIcon />
        </button>
        <button
            class="btn-icon"
            on:click={async () => {
                let name = await promptPopup("Enter the new name");
                editTrackerConfig(id, (config) => {
                    config.name = name;
                });
            }}
        >
            <PencilIcon />
        </button>
    </div>
    <select
        class="text-neutral-700 px-1 mt-2 bg-white"
        value={config?.location ?? ""}
        on:change={(e) => {
            editTrackerConfig(id, (config) => {
                config.location = e.currentTarget.value;
            });
        }}
    >
        {#each commonBoneLocations as location}
            <option value={location}>{location}</option>
        {/each}
    </select>
    {#if showInspect}
        <hr class="my-4" />
        <div class="text-sm text-neutral-300">
            <p>Connected from {tracker.info.address}</p>
            <TrackerInspect data={tracker.data} />
        </div>
    {/if}
</div>
