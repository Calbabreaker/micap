<script lang="ts">
    import { T, useThrelte } from "@threlte/core";
    import { SkeletonHelper, type Group, Quaternion } from "three";
    import { GLTF, OrbitControls } from "@threlte/extras";
    import { bones, type BoneDict } from "$lib/websocket";

    const { invalidate } = useThrelte();

    let modelRef: Group;

    function onModelLoad(model: Group) {
        model.parent!.add(new SkeletonHelper(model));
        modelRef = model;
    }

    function updateBones(bones: BoneDict) {
        if (!modelRef) {
            return;
        }

        Object.entries(bones).forEach(([location, bone]) => {
            const part = modelRef.getObjectByName(location);
            if (part) {
                // part.position.fromArray(bone.tail_offset).multiplyScalar(100);
                let quat = new Quaternion().fromArray(bone.orientation);
                part.rotation.setFromQuaternion(quat);
            }
        });

        invalidate();
    }

    $: updateBones($bones);
</script>

<T.PerspectiveCamera makeDefault position={[10, 10, -10]}>
    <OrbitControls />
</T.PerspectiveCamera>
<GLTF url={"default.glb"} scale={4} on:create={({ ref }) => onModelLoad(ref)} />
