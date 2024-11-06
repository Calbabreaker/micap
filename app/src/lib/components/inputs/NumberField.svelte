<script lang="ts">
    import ResetButton from "./ResetButton.svelte";

    export let value: number;
    export let defaultValue: number;
    export let type = "number";
</script>

<div class="flex items-center gap-2">
    <input
        placeholder="Send port"
        {value}
        on:change={(e) => (value = Number(e.currentTarget.value))}
        class="text-input"
        {type}
    />
    <ResetButton bind:value {defaultValue} />
</div>
