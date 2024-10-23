<script lang="ts">
    import { globalConfig, updateConfig, defaultConfig } from "$lib/websocket";
    import Checkbox from "../inputs/Checkbox.svelte";
    import NumberField from "../inputs/NumberField.svelte";
    import BoneCheckboxes from "./BoneCheckboxes.svelte";

    let config = $globalConfig.vrchat;
</script>

<form class="inputs-form" on:change={() => updateConfig("vrchat", config)}>
    <span>Enabled</span>
    <Checkbox
        bind:value={config.enabled}
        defaultValue={defaultConfig.vrchat.enabled}
    />

    <span>Send port</span>
    <NumberField
        bind:value={config.send_port}
        defaultValue={defaultConfig.vrchat.send_port}
    />

    <BoneCheckboxes
        bind:bonesToSend={config.bones_to_send}
        defaultBonesToSend={defaultConfig.vrchat.bones_to_send}
    />
</form>
