<script lang="ts">
    import type { VmcConfig, VrChatConfig } from "$lib/server_bindings";
    import { defaultConfig } from "$lib/websocket";
    import ResetButton from "../inputs/ResetButton.svelte";

    export let configType: "vmc" | "vrchat";
    export let config: VrChatConfig | VmcConfig;
</script>

<span class="my-auto">Enabled</span>
<div class="flex items-center gap-2">
    <input type="checkbox" bind:checked={config.enabled} />
    <ResetButton
        bind:value={config.enabled}
        defaultValue={defaultConfig[configType].enabled}
    />
</div>

<span class="my-auto">Send port</span>
<div class="flex items-center gap-2">
    <input
        placeholder="Send port"
        value={config.send_port}
        on:change={(e) => (config.send_port = Number(e.currentTarget.value))}
        class="text-input"
        type="number"
    />
    <ResetButton
        bind:value={config.send_port}
        defaultValue={defaultConfig[configType].send_port}
    />
</div>
<slot />
