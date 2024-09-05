<script lang="ts">
    import { sendWebsocket, info } from "$lib/websocket";

    let ssid = "";
    let password = "";

    function setWifi() {
        sendWebsocket({
            type: "Wifi",
            ssid,
            password,
        });
    }
</script>

<form on:submit|preventDefault={setWifi}>
    <input
        placeholder="WiFi SSID"
        bind:value={ssid}
        class="p-2 rounded text-black"
    />
    <input
        placeholder="Password"
        type="password"
        bind:value={password}
        class="p-2 rounded text-black"
    />
    <button type="submit" class="btn btn-primary mr-2 inline-block">
        Set
    </button>
    <p>{$info}</p>
</form>
