<script lang="ts">
    import type { BoneLocation } from "$lib/server_bindings";
    import ResetButton from "../inputs/ResetButton.svelte";

    const sendableBones: BoneLocation[] = [
        "Hip",
        "Chest",
        "LeftFoot",
        "RightFoot",
        "RightLowerLeg",
        "LeftLowerLeg",
        "LeftUpperArm",
        "RightUpperArm",
    ];

    export let bonesToSend: BoneLocation[];
    export let defaultBonesToSend: BoneLocation[];
</script>

<div class="flex items-center gap-2">
    <span class="text-lg font-bold">Bones to send</span>
    <ResetButton bind:value={bonesToSend} defaultValue={defaultBonesToSend} />
</div>
<div class="grid grid-cols-2 col-span-2 gap-x-4">
    {#each sendableBones as location}
        <div class="flex justify-between gap-2">
            <span>{location}</span>
            <input
                type="checkbox"
                checked={bonesToSend.includes(location)}
                on:change={(e) => {
                    if (e.currentTarget.checked) {
                        bonesToSend.push(location);
                    } else {
                        const i = bonesToSend.findIndex((v) => v == location);
                        bonesToSend.splice(i, 1);
                    }
                    bonesToSend = bonesToSend; // Retrigger update
                }}
            />
        </div>
    {/each}
</div>
