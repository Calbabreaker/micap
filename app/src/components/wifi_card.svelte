<script lang="ts">
    export let ws: WebSocket;

    let ssid = "";
    let password = "";

    function setWifi(e: SubmitEvent) {
        e.preventDefault();
        ws.send(`SERIAL:WIFI\0${ssid}\0${password}`);
    }
</script>

<form on:submit={setWifi} class="bg-neutral-700 p-4 shadow rounded">
    <input placeholder="WiFi SSID" bind:value={ssid} />
    <input placeholder="Password" type="password" bind:value={password} />
    <button type="submit" class={`btn btn-primary mr-2 inline-block`}>
        Set
    </button>
</form>
