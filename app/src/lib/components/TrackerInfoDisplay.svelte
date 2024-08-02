<script lang="ts">
    import type { TrackerInfo } from "$lib/websocket";
    import BatteryIcon from "./icons/BatteryIcon.svelte";

    export let info: TrackerInfo;
</script>

<div class="flex text-sm text-neutral-300">
    <div class="mr-2">
        <span
            class:text-red-400={info.status == "Error"}
            class:text-green-400={info.status == "Ok"}
            class:text-yellow-400={info.status == "TimedOut"}
            class:text-gray-400={info.status == "Off"}
        >
            {info.status}
        </span>
        {#if info.latency_ms}
            <span>
                {info.latency_ms}ms
            </span>
        {/if}
    </div>
    <BatteryIcon level={info.battery_level} />
    <span class="ml-1">
        {Math.round(info.battery_level * 100)}%
    </span>
</div>
