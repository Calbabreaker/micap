<script lang="ts">
    import { sendWebsocket } from "$lib/websocket";

    let ssid = "";
    let password = "";

    function setWifi() {
        if (ssid.length == 0) {
            return;
        }
        sendWebsocket({
            type: "SerialSend",
            data: `Wifi\0${ssid}\0${password}\n`,
        });
    }
</script>

<form class="inputs-form" on:submit|preventDefault={setWifi}>
    <span class="my-auto">SSID</span>
    <input
        placeholder="WiFi SSID"
        bind:value={ssid}
        maxlength="32"
        class="text-input"
    />

    <span class="my-auto">Password</span>
    <input
        placeholder="Password"
        bind:value={password}
        maxlength="64"
        class="text-input"
    />

    <button type="submit" class="btn col-span-2">Send</button>
</form>
