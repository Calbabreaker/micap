<script lang="ts">
    import { T } from "@threlte/core";
    import { Euler, SkeletonHelper, type Group } from "three";
    import { GLTF, OrbitControls } from "@threlte/extras";
    import PreviewCanvas from "./PreviewCanvas.svelte";

    function onModelLoad(model: Group) {
        const root = model.getObjectByName("Armature");
        root?.traverse((object) => {
            object.setRotationFromEuler(new Euler(0, 0, 0));
            // console.log(object.position);
        });
        model.parent!.add(new SkeletonHelper(model));

        // const foot = ref.getObjectByName("LeftToe");
        // foot.position.y = 10;
    }
</script>

<PreviewCanvas>
    <T.PerspectiveCamera makeDefault position={[10, 10, -10]}>
        <OrbitControls />
    </T.PerspectiveCamera>
    <GLTF
        url={"default.glb"}
        scale={4}
        on:create={({ ref }) => onModelLoad(ref)}
    />
</PreviewCanvas>
