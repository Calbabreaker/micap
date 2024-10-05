<script lang="ts">
    import { T, useThrelte } from "@threlte/core";
    import * as THREE from "three";
    import { GLTF, OrbitControls } from "@threlte/extras";
    import { bones, type BoneDict } from "$lib/websocket";
    import type { BoneLocation } from "$lib/server_bindings";

    const { invalidate } = useThrelte();

    export let showModel = true;
    export let showLines = true;

    let modelRef: THREE.Group;

    let boneLines: { [key in BoneLocation]?: THREE.Line };

    function makeBoneLines(bonesData: BoneDict) {
        const lines: { [key in BoneLocation]?: THREE.Line } = {};

        Object.entries(bonesData).forEach(([location, boneData]) => {
            const material = new THREE.LineBasicMaterial({
                color: 0xff0000,
            });
            const geometry = new THREE.BufferGeometry().setFromPoints([
                new THREE.Vector3()
                    .fromArray(boneData.tail_world_position)
                    .multiplyScalar(-10),
                new THREE.Vector3()
                    .fromArray(boneData.tail_offset)
                    .multiplyScalar(0),
            ]);

            lines[location as BoneLocation] = new THREE.Line(
                geometry,
                material,
            );
        });

        // Parent the bones
        Object.entries(bonesData).forEach(([location, boneData]) => {
            const line = lines[location as BoneLocation]!;
            line.name = location;
            if (boneData.parent) {
                lines[boneData.parent as BoneLocation]!.add(line);
            }
        });

        console.log(lines.Hip);

        return lines;
    }

    function updateBones(bonesData?: BoneDict) {
        if (!bonesData || !modelRef) {
            return;
        }

        if (!boneLines) {
            boneLines = makeBoneLines(bonesData);
        }

        Object.entries(bonesData).forEach(([location, boneData]) => {
            const quat = new THREE.Quaternion().fromArray(boneData.orientation);

            boneLines[location as BoneLocation]!.rotation.setFromQuaternion(
                quat,
            );

            if (location != "Hip") {
                const part = modelRef.getObjectByName(location);
                if (part) {
                    part.rotation.setFromQuaternion(quat);
                }
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
{#if showLines && boneLines?.Hip}
    <T is={boneLines.Hip} />
{/if}
