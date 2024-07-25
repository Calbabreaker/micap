<script lang="ts">
    import { websocket } from "$lib/websocket";
    import WifiForm from "$lib/components/wifi_form.svelte";
</script>

<WifiForm />
<button
    class="btn btn-primary"
    on:click={() => {
        if ($websocket && confirm("Are you sure?")) {
            $websocket.send(JSON.stringify({ type: "FactoryReset" }));
        }
    }}
>
    Factory reset
</button>
