<script lang="ts">
    import { websocket } from "$lib/websocket";
    import WifiForm from "$lib/components/WifiForm.svelte";
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
