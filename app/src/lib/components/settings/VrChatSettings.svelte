<script lang="ts">
    import { globalConfig, editConfig, defaultConfig } from "$lib/websocket";
    import BoneCheckboxes from "./BoneCheckboxes.svelte";
    import ConnectorSettings from "./ConnectorSettings.svelte";

    let config = $globalConfig.vrchat;
</script>

<form class="inputs-form" on:change={() => editConfig("vrchat", config)}>
    <ConnectorSettings configType="vrchat" bind:config />
    <BoneCheckboxes
        bind:bonesToSend={config.bones_to_send}
        defaultBonesToSend={defaultConfig.vrchat.bones_to_send}
    />
</form>
