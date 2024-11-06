<script lang="ts">
    import { defaultConfig, globalConfig, updateConfig } from "$lib/websocket";
    import NumberField from "../inputs/NumberField.svelte";

    let config = $globalConfig.skeleton;
</script>

<form class="inputs-form" on:change={() => updateConfig("skeleton", config)}>
    <span>Height (meters)</span>
    <NumberField
        bind:value={config.user_height}
        defaultValue={defaultConfig.skeleton.user_height}
        type="text"
    />
</form>
