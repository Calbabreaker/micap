<script lang="ts">
    import { defaultConfig, editConfig } from "$lib/websocket";

    function reset() {
        editConfig("skeleton", {
            offsets: defaultConfig!.skeleton.offsets,
        });
    }
</script>

<button class="btn" on:click={reset}>Reset</button>
