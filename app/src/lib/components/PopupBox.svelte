<script lang="ts">
    export let title: string;
    export let message: string;

    export let onClick: (ok: boolean) => void;
</script>

<div
    class="fixed top-0 left-0 bg-black/60 w-full h-full z-20 overflow-y-auto overflow-x-hidden p-4 grid place-items-center"
>
    <div class="bg-neutral-600 p-4 rounded max-w-lg h-fit">
        <div class="h-6 flex flex-row items-center mb-1">
            <h1 class="font-bold text-xl">{title}</h1>
        </div>
        <p class="flex-1 mt-4">
            {message}
        </p>
        <div class="mt-4 flex justify-around">
            <button class="btn" on:click={() => onClick(false)}>
                Cancel
            </button>
            <button class="btn btn-primary" on:click={() => onClick(true)}>
                Ok
            </button>
        </div>
    </div>
</div>
