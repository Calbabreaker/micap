<script lang="ts">
    import { globalConfig, updateConfig, defaultConfig } from "$lib/websocket";
    import type { InterfaceConfig } from "$lib/server_bindings";
    import ResetButton from "../inputs/ResetButton.svelte";

    let config: InterfaceConfig;

    $: if ($globalConfig) config = $globalConfig.interface;

    function apply() {
        updateConfig("interface", config);
    }
</script>

<form class="inputs-form" on:change={apply}>
    <span class="my-auto">Hide in system tray</span>
    <div class="flex items-center gap-2">
        <input type="checkbox" bind:checked={config.hide_in_system_tray} />
        <ResetButton
            bind:value={config.hide_in_system_tray}
            defaultValue={defaultConfig.interface.hide_in_system_tray}
        />
    </div>
</form>
