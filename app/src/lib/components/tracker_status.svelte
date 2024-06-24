<script lang="ts">
    import type { TrackerStatus } from "$lib/websocket";
    export let status: TrackerStatus;
</script>

<span
    class:text-red-400={status == "Error"}
    class:text-green-400={status == "Ok"}
    class:text-yellow-400={status == "TimedOut"}
    class:text-gray-400={status == "Off"}
>
    {status}
</span>
