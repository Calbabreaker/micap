<script lang="ts">
    import type { VmcConfig } from "$lib/server_bindings";
    import { globalConfig, editConfig } from "$lib/websocket";
    import ConnectorSettings from "./ConnectorSettings.svelte";

    let config: VmcConfig;

    $: if ($globalConfig) config = $globalConfig.vmc;
</script>

<form class="inputs-form" on:change={() => editConfig("vmc", config)}>
    <ConnectorSettings configType="vmc" bind:config />
</form>
