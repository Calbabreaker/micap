<script lang="ts">
    import { T } from "@threlte/core";
    import { trackers } from "$lib/websocket";
    import { ArrowHelper, Vector3 } from "three";
    import PreviewCanvas from "./PreviewCanvas.svelte";
    import { OrbitControls } from "@threlte/extras";

    export let id: string;
    $: data = $trackers[id].data;

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

<div>
    <div class="mb-2 text-sm text-neutral-300">
        <p>Orientation: {formatArray(data.orientation)}</p>
        <p>Acceleration: {formatArray(data.acceleration)}</p>
        <p>Position: {formatArray(data.position)}</p>
    </div>
    <PreviewCanvas>
        <T.PerspectiveCamera makeDefault position={[10, 10, 10]}>
            <OrbitControls />
        </T.PerspectiveCamera>
        <T.Mesh quaternion={data.orientation} scale={[1.5, 1.5, 1.5]}>
            <T.AxesHelper args={[5]} />
            <T.BoxGeometry />
            <T.MeshLambertMaterial color={0xffffff} />
        </T.Mesh>
        <T.ArrowHelper bind:ref={arrowRef} />
    </PreviewCanvas>
</div>
