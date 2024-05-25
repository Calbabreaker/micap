<script lang="ts">
    export let ws: WebSocket;

    let ssid = "";
    let password = "";

    function setWifi(e: SubmitEvent) {
        e.preventDefault();
        ws.send(
            JSON.stringify({
                type: "Serial",
                command: `WIFI\0${ssid}\0${password}`,
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
