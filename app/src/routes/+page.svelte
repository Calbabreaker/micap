<script lang="ts">
    import { websocket, websocketError } from "$lib/websocket";
    import WifiForm from "$lib/components/wifi_form.svelte";
    import ClientsList from "$lib/components/clients_list.svelte";
</script>

<ClientsList />
<WifiForm />
<p class="text-red-300">{$websocketError}</p>
<button
    class="btn btn-primary"
    on:click={() => {
        if (confirm("Are you sure?")) {
            $websocket.send(JSON.stringify({ type: "FactoryReset" }));
        }
    }}
>
    Factory reset
</button>
