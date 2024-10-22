<script lang="ts">
    import type { VmcConfig } from "$lib/server_bindings";
    import { globalConfig, updateConfig } from "$lib/websocket";
    import ConnectorSettings from "./ConnectorSettings.svelte";

    let config: VmcConfig;

    $: if ($globalConfig) config = $globalConfig.vmc;
</script>

<form class="inputs-form" on:change={() => updateConfig("vmc", config)}>
    <ConnectorSettings configType="vmc" bind:config />
</form>
