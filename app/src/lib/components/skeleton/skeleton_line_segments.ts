import type { BoneDict } from "$lib/websocket";
import { LineSegments2 } from "three/addons/lines/LineSegments2.js";
import { LineSegmentsGeometry } from "three/addons/lines/LineSegmentsGeometry.js";
import { LineMaterial } from "three/addons/lines/LineMaterial.js";
import type { BoneLocation } from "$lib/server_bindings";

export class SkeletonLineSegments extends LineSegments2 {
    constructor(bonesData: BoneDict) {
        const material = new LineMaterial({
            linewidth: 4,
            vertexColors: true,
        });

        const colors: number[] = [];

        Object.keys(bonesData).forEach((location) => {
            const color = getBoneColor(location as BoneLocation);
            colors.push(...color, ...color);
        });

        const geometry = new LineSegmentsGeometry();
        geometry.setColors(colors);
        geometry.setPositions(generatePoints(bonesData));

        super(geometry, material);
    }

    update(bonesData: BoneDict) {
        this.geometry.setPositions(generatePoints(bonesData));
    }
}

function generatePoints(bonesData: BoneDict): number[] {
    const points: number[] = [];

    Object.values(bonesData).forEach((boneData) => {
        if (boneData.parent) {
            points.push(
                ...bonesData[boneData.parent].tail_world_position,
                ...boneData.tail_world_position,
            );
        }
    });

    return points;
}

function getBoneColor(location: BoneLocation): [number, number, number] {
    // prettier-ignore
    switch (location) {
        case "Hip": return [1, 0, 0];
        case "LeftUpperLeg":
        case "RightUpperLeg": return [1, 1, 0]
        case "LeftLowerLeg": 
        case "RightLowerLeg": return [0, 1, 0];
        case "LeftFoot": 
        case "RightFoot": return [1, 0, 1];
        case "Waist": return [0, 0, 1];
        case "Chest": return [0, 1, 1];
        case "UpperChest": return [0, 1, 0.5];
        case "Neck": return [1, 0.4, 0.3];
        case "Head": return [0.5, 0.5, 1];
        case "LeftShoulder": 
        case "RightShoulder": return [0.6, 0.3, 1];
        case "LeftUpperArm": 
        case "RightUpperArm": return [0.2, 0.8, 0];
        case "LeftLowerArm": 
        case "RightLowerArm": return [1, 0, 0.2];
        case "LeftHand": 
        case "RightHand": return [1, 0.8, 0.1];
        case "LeftHip": 
        case "RightHip": return [0.2, 0.5, 0];
    }
}
