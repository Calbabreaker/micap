<script lang="ts">
    import { trackers } from "$lib/websocket";
    import TrackerInfoDisplay from "./TrackerInfoDisplay.svelte";
    import TrackerControl from "./TrackerControl.svelte";

    export let id: string;

    $: tracker = $trackers[id]!;

    // Highlight border when there is movement
    $: brightness = Math.min(
        Math.hypot(...tracker.data?.acceleration) * 50,
        50,
    );
</script>

<div
    style:border-color={`hsl(0, 0%, ${brightness}%)`}
    class="bg-neutral-600 p-4 rounded shadow w-fit border"
>
    <TrackerInfoDisplay info={tracker.info} />
    <TrackerControl {id} />
</div>
