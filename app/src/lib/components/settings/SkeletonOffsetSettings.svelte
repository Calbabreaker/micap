<script lang="ts">
    import { defaultConfig, updateConfig } from "$lib/websocket";

    function reset() {
        updateConfig("skeleton", {
            offsets: defaultConfig!.skeleton.offsets,
        });
    }
</script>

<button class="btn" on:click={reset}>Reset</button>
