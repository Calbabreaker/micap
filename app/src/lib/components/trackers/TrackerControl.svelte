<script lang="ts">
    import type { BoneLocation } from "$lib/server_bindings";
    import { promptPopup } from "$lib/toast";
    import {
        updateTrackerConfig,
        globalConfig,
        removeTracker,
        trackers,
    } from "$lib/websocket";
    import MangnifyingGlassIcon from "../icons/MangnifyingGlassIcon.svelte";
    import PencilIcon from "../icons/PencilIcon.svelte";
    import TrashIcon from "../icons/TrashIcon.svelte";
    import TrackerPreview from "./TrackerPreview.svelte";

    export let id: string;

    const commonBoneLocations = [
        "None",
        "Head",
        "CenterHip",
        "LeftUpperLeg",
        "RightUpperLeg",
        "LeftLowerLeg",
        "RightLowerLeg",
        "LeftFoot",
        "RightFoot",
        "Waist",
        "Chest",
        "Neck",
        "LeftUpperArm",
        "RightUpperArm",
        "LeftLowerArm",
        "RightLowerArm",
        "LeftHand",
        "RightHand",
    ];

    let showInspect = false;

    $: config = $globalConfig?.trackers[id];
    $: tracker = $trackers[id]!;

    async function enterNewName() {
        const name = await promptPopup("Enter the new name");
        updateTrackerConfig(id, {
            ...config,
            name,
        });
    }

    function setLocation(location: string) {
        updateTrackerConfig(id, {
            ...config,
            location:
                location == "None" ? undefined : (location as BoneLocation),
        });
    }
</script>

<p>{config?.name || id}</p>
<div class="flex gap-1 mt-2">
    <button class="btn-icon" on:click={() => removeTracker(id)}>
        <TrashIcon />
    </button>
    <button class="btn-icon" on:click={() => (showInspect = !showInspect)}>
        <MangnifyingGlassIcon />
    </button>
    <button class="btn-icon" on:click={enterNewName}>
        <PencilIcon />
    </button>
</div>
<select
    class="text-neutral-700 px-1 mt-2 bg-white"
    value={config?.location ?? "None"}
    on:change={(e) => {
        setLocation(e.currentTarget.value);
    }}
>
    {#each commonBoneLocations as location}
        <option value={location}>{location}</option>
    {/each}
</select>
{#if showInspect}
    <hr class="my-4" />
    <div class="text-sm text-neutral-300">
        <p>Address: {tracker.info.address}</p>
        <p>ID: {id}</p>
        <TrackerPreview data={tracker.data} />
    </div>
{/if}
