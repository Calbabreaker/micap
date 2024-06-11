<script lang="ts">
    import { websocket } from "$lib/websocket";

    let ssid = "";
    let password = "";

    function setWifi(e: SubmitEvent) {
        e.preventDefault();
        $websocket.send(
            JSON.stringify({
                type: "Wifi",
                ssid,
                password,
            }),
        );
    }
</script>

<form on:submit={setWifi} class="bg-neutral-700 p-4 shadow rounded">
    <input placeholder="WiFi SSID" bind:value={ssid} class="p-2" />
    <input
        placeholder="Password"
        type="password"
        bind:value={password}
        class="p-2"
    />
    <button type="submit" class={`btn btn-primary mr-2 inline-block`}>
        Set
    </button>
</form>
