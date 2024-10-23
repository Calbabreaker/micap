<script lang="ts">
    import ResetButton from "./ResetButton.svelte";

    export let value: boolean;
    export let defaultValue: boolean;
</script>

<div class="flex items-center gap-4">
    <input type="checkbox" bind:checked={value} />
    <ResetButton bind:value {defaultValue} />
</div>
