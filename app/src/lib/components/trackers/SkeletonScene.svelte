<script lang="ts">
    import { T, useThrelte } from "@threlte/core";
    import * as THREE from "three";
    import { GLTF, OrbitControls } from "@threlte/extras";
    import { bones, type BoneDict } from "$lib/websocket";
    import type { BoneLocation } from "$lib/server_bindings";

    const { invalidate, scene } = useThrelte();

    export let showModel = true;
    export let showLines = true;

    let modelRef: THREE.Group;

    let boneObjects: { [key in BoneLocation]?: THREE.Bone };

    function updateBones(bonesData?: BoneDict) {
        if (!bonesData || !modelRef) {
            return;
        }

        // if (!boneObjects) {
        //     boneObjects = {};
        //     for (const location in bonesData) {
        //         boneObjects[location as BoneLocation] = new THREE.Bone();
        //     }

        //     Object.entries(bonesData).forEach(([location, boneData]) => {
        //         const bone = boneObjects[location as BoneLocation]!;
        //         bone.name = location;
        //         if (boneData.parent) {
        //             boneObjects[boneData.parent as BoneLocation]!.add(bone);
        //         }
        //     });

        //     // const skeleton = new THREE.Skeleton(Object.values(boneObjects));
        //     const skeletonHelper = new THREE.SkeletonHelper(boneObjects.Hip!);
        //     console.log(skeletonHelper);
        //     // scene.add(skeletonHelper);
        //     scene.add(skeletonHelper);
        // }

        // Object.entries(boneObjects).forEach(([location, bone]) => {
        //     // @ts-ignore
        //     const boneData = bonesData[location];
        //     bone.children.forEach((child) => {
        //         child.position.fromArray(boneData.tail_offset);
        //     });
        //     const quat = new THREE.Quaternion().fromArray(boneData.orientation);
        //     bone.rotation.setFromQuaternion(quat);
        // });

        Object.entries(bonesData).forEach(([location, boneData]) => {
            const part = modelRef.getObjectByName(location);
            if (part) {
                // part.children.forEach((child) => {
                //     child.position.fromArray(boneData.tail_offset);
                // });
                let quat = new THREE.Quaternion().fromArray(
                    boneData.orientation,
                );
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
{#if showModel}
    <GLTF
        url={"default.glb"}
        scale={4}
        on:create={({ ref }) => (modelRef = ref)}
    />
{/if}
