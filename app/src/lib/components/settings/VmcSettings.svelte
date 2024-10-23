<script lang="ts">
    import type { VmcConfig } from "$lib/server_bindings";
    import { defaultConfig, globalConfig, updateConfig } from "$lib/websocket";
    import Checkbox from "../inputs/Checkbox.svelte";
    import NumberField from "../inputs/NumberField.svelte";

    let config: VmcConfig;

    $: if ($globalConfig) config = $globalConfig.vmc;
</script>

<form class="inputs-form" on:change={() => updateConfig("vmc", config)}>
    <span>Enabled</span>
    <Checkbox
        bind:value={config.enabled}
        defaultValue={defaultConfig.vmc.enabled}
    />

    <span>Send port</span>
    <NumberField
        bind:value={config.send_port}
        defaultValue={defaultConfig.vmc.send_port}
    />
</form>
