<script>
    import { sendWebsocket } from "$lib/websocket";
    import CountDownButton from "./inputs/CountDownButton.svelte";
    import { save } from "@tauri-apps/plugin-dialog";

    let recording = false;

    async function startStopRecord() {
        recording = !recording;
        if (recording) {
            sendWebsocket({ type: "StartRecord" });
        } else {
            const savePath = await save({
                filters: [
                    {
                        name: "Biovision Hierarchy",
                        extensions: ["bvh"],
                    },
                ],
                title: "Save file",
            });

            if (savePath) {
                sendWebsocket({ type: "StopRecord", save_path: savePath });
            }
        }
    }
</script>

<div class="grid grid-cols-2 gap-2">
    <CountDownButton
        onActivate={() => {
            sendWebsocket({ type: "ResetTrackerOrientations" });
        }}
        initialContent="Reset Orientations"
        countdownContent="Reseting in"
    />
    {#if recording}
        <button class="btn" on:click={startStopRecord}>Stop Record</button>
    {:else}
        <CountDownButton
            onActivate={startStopRecord}
            initialContent="Start record"
            countdownContent="Recording in"
        />
    {/if}
</div>
