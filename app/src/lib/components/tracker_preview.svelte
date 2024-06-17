<script lang="ts">
    import { Canvas } from "@threlte/core";
    import { T } from "@threlte/core";
    import type { TrackerData } from "$lib/websocket";
    import { ArrowHelper, Vector3 } from "three";

    export let data: TrackerData;

    $: updateArrow(
        new Vector3(data.velocity[0], data.velocity[1], data.velocity[2]),
    );

    let arrowRef: ArrowHelper;

    function updateArrow(vector: Vector3) {
        if (arrowRef) {
            arrowRef.setLength(vector.length());
            arrowRef.setDirection(vector.normalize());
            arrowRef.setColor(0xffffff);
        }
    }
</script>

<Canvas>
    <T.PerspectiveCamera
        makeDefault
        position={[10, 10, 10]}
        on:create={({ ref }) => {
            ref.lookAt(0, 1, 0);
        }}
    />
    <T.DirectionalLight args={[0xffffff, 1]} />
    <T.AmbientLight args={[0xffffff, 0.5]} />
    <T.GridHelper args={[10, 10]} />

    <T.ArrowHelper bind:ref={arrowRef} />
    <T.Mesh quaternion={data.orientation} scale={[2, 2, 2]}>
        <T.AxesHelper args={[5]} />
        <T.BoxGeometry />
        <T.MeshLambertMaterial color={0xffffff} />
    </T.Mesh>
</Canvas>
