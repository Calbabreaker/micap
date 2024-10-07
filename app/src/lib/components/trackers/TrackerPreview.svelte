<script lang="ts">
    import { T } from "@threlte/core";
    import { type ArrowHelper, Vector3 } from "three";
    import PreviewCanvas from "../PreviewCanvas.svelte";
    import { OrbitControls } from "@threlte/extras";
    import type { TrackerData } from "$lib/server_bindings";

    export let data: TrackerData;

    $: updateArrow(new Vector3().fromArray(data.acceleration));

    let arrowRef: ArrowHelper;

    // $: positionLooped = [
    //     data?.position[0] % 6,
    //     data?.position[1] % 6,
    //     data?.position[2] % 6,
    // ];

    function updateArrow(vector: Vector3) {
        if (arrowRef) {
            arrowRef.setLength(vector.length() * 4);
            arrowRef.setDirection(vector.normalize());
            arrowRef.setColor(0xffffff);
        }
    }

    function formatArray(array: number[]) {
        return array.map((value) => value.toFixed(2)).join(", ");
    }
</script>

<p>Orientation: {formatArray(data.orientation)}</p>
<p>Acceleration: {formatArray(data.acceleration)}</p>
<p class="mb-2">Position: {formatArray(data.position)}</p>
<PreviewCanvas>
    <T.PerspectiveCamera makeDefault position={[8, 8, -8]}>
        <OrbitControls />
    </T.PerspectiveCamera>
    <T.Mesh quaternion={data.orientation} scale={[1.5, 1.5, 1.5]}>
        <T.AxesHelper args={[5]} />
        <T.BoxGeometry />
        <T.MeshLambertMaterial color={0xffffff} />
    </T.Mesh>
    <T.ArrowHelper bind:ref={arrowRef} />
</PreviewCanvas>
