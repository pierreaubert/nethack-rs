"""
Blender headless script for auto-rigging a humanoid mesh.

Called via:
    blender --background --python blender_rig.py -- input.obj output.fbx

This places a basic humanoid armature scaled to the mesh bounding box,
then uses Blender's "automatic weights" to skin it.
"""

import bpy
import sys
import os
from mathutils import Vector


def clear_scene():
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)
    for collection in bpy.data.collections:
        bpy.data.collections.remove(collection)


def import_mesh(filepath):
    ext = os.path.splitext(filepath)[1].lower()
    if ext == '.obj':
        bpy.ops.wm.obj_import(filepath=filepath)
    elif ext == '.glb' or ext == '.gltf':
        bpy.ops.import_scene.gltf(filepath=filepath)
    elif ext == '.fbx':
        bpy.ops.import_scene.fbx(filepath=filepath)
    elif ext == '.ply':
        bpy.ops.wm.ply_import(filepath=filepath)
    else:
        raise ValueError(f"Unsupported format: {ext}")

    # Find the imported mesh
    mesh_obj = None
    for obj in bpy.context.scene.objects:
        if obj.type == 'MESH':
            mesh_obj = obj
            break

    if mesh_obj is None:
        raise RuntimeError("No mesh found after import")

    return mesh_obj


def get_mesh_bounds(obj):
    """Get world-space bounding box of mesh."""
    bbox = [obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
    xs = [v.x for v in bbox]
    ys = [v.y for v in bbox]
    zs = [v.z for v in bbox]
    return (
        Vector((min(xs), min(ys), min(zs))),
        Vector((max(xs), max(ys), max(zs))),
    )


def create_humanoid_armature(mesh_obj):
    """
    Create a basic humanoid armature scaled to fit the mesh.
    Assumes the mesh is roughly upright along Z axis.
    """
    bb_min, bb_max = get_mesh_bounds(mesh_obj)
    height = bb_max.z - bb_min.z
    width = bb_max.x - bb_min.x
    center_x = (bb_min.x + bb_max.x) / 2
    center_y = (bb_min.y + bb_max.y) / 2
    base_z = bb_min.z

    # Proportions relative to total height (approximate humanoid)
    foot_z = base_z
    knee_z = base_z + height * 0.25
    hip_z = base_z + height * 0.47
    spine_z = base_z + height * 0.55
    chest_z = base_z + height * 0.65
    neck_z = base_z + height * 0.82
    head_z = base_z + height * 1.0

    shoulder_offset = width * 0.22
    elbow_offset = width * 0.40
    hand_offset = width * 0.50
    hip_offset = width * 0.10

    bpy.ops.object.armature_add(enter_editmode=True, location=(center_x, center_y, 0))
    armature_obj = bpy.context.object
    armature = armature_obj.data
    armature.name = "Humanoid"

    # Remove default bone
    bpy.ops.armature.select_all(action='SELECT')
    bpy.ops.armature.delete()

    def add_bone(name, head, tail, parent_name=None):
        bone = armature.edit_bones.new(name)
        bone.head = Vector(head)
        bone.tail = Vector(tail)
        if parent_name:
            bone.parent = armature.edit_bones[parent_name]
            bone.use_connect = (bone.head - bone.parent.tail).length < 0.001
        return bone

    cx, cy = center_x, center_y

    # Spine chain
    add_bone("Hips", (cx, cy, hip_z), (cx, cy, spine_z))
    add_bone("Spine", (cx, cy, spine_z), (cx, cy, chest_z), "Hips")
    add_bone("Chest", (cx, cy, chest_z), (cx, cy, neck_z), "Spine")
    add_bone("Neck", (cx, cy, neck_z), (cx, cy, neck_z + (head_z - neck_z) * 0.4), "Chest")
    add_bone("Head", (cx, cy, neck_z + (head_z - neck_z) * 0.4), (cx, cy, head_z), "Neck")

    # Left leg
    add_bone("UpperLeg.L", (cx + hip_offset, cy, hip_z), (cx + hip_offset, cy, knee_z), "Hips")
    add_bone("LowerLeg.L", (cx + hip_offset, cy, knee_z), (cx + hip_offset, cy, foot_z + height * 0.05), "UpperLeg.L")
    add_bone("Foot.L", (cx + hip_offset, cy, foot_z + height * 0.05), (cx + hip_offset, cy - height * 0.06, foot_z), "LowerLeg.L")

    # Right leg
    add_bone("UpperLeg.R", (cx - hip_offset, cy, hip_z), (cx - hip_offset, cy, knee_z), "Hips")
    add_bone("LowerLeg.R", (cx - hip_offset, cy, knee_z), (cx - hip_offset, cy, foot_z + height * 0.05), "UpperLeg.R")
    add_bone("Foot.R", (cx - hip_offset, cy, foot_z + height * 0.05), (cx - hip_offset, cy - height * 0.06, foot_z), "LowerLeg.R")

    # Left arm
    add_bone("Shoulder.L", (cx, cy, neck_z - height * 0.02), (cx + shoulder_offset, cy, neck_z - height * 0.02), "Chest")
    add_bone("UpperArm.L", (cx + shoulder_offset, cy, neck_z - height * 0.02), (cx + elbow_offset, cy, chest_z - height * 0.02), "Shoulder.L")
    add_bone("LowerArm.L", (cx + elbow_offset, cy, chest_z - height * 0.02), (cx + hand_offset, cy, spine_z), "UpperArm.L")
    add_bone("Hand.L", (cx + hand_offset, cy, spine_z), (cx + hand_offset, cy, spine_z - height * 0.04), "LowerArm.L")

    # Right arm
    add_bone("Shoulder.R", (cx, cy, neck_z - height * 0.02), (cx - shoulder_offset, cy, neck_z - height * 0.02), "Chest")
    add_bone("UpperArm.R", (cx - shoulder_offset, cy, neck_z - height * 0.02), (cx - elbow_offset, cy, chest_z - height * 0.02), "Shoulder.R")
    add_bone("LowerArm.R", (cx - elbow_offset, cy, chest_z - height * 0.02), (cx - hand_offset, cy, spine_z), "UpperArm.R")
    add_bone("Hand.R", (cx - hand_offset, cy, spine_z), (cx - hand_offset, cy, spine_z - height * 0.04), "LowerArm.R")

    bpy.ops.object.mode_set(mode='OBJECT')
    return armature_obj


def parent_with_automatic_weights(mesh_obj, armature_obj):
    """Parent mesh to armature with automatic weight painting."""
    bpy.ops.object.select_all(action='DESELECT')
    mesh_obj.select_set(True)
    armature_obj.select_set(True)
    bpy.context.view_layer.objects.active = armature_obj
    bpy.ops.object.parent_set(type='ARMATURE_AUTO')


def export_rigged(armature_obj, mesh_obj, output_path):
    """Export the rigged character."""
    bpy.ops.object.select_all(action='DESELECT')
    armature_obj.select_set(True)
    mesh_obj.select_set(True)

    ext = os.path.splitext(output_path)[1].lower()
    if ext == '.fbx':
        bpy.ops.export_scene.fbx(
            filepath=output_path,
            use_selection=True,
            add_leaf_bones=False,
            bake_anim=False,
        )
    elif ext == '.glb' or ext == '.gltf':
        bpy.ops.export_scene.gltf(
            filepath=output_path,
            use_selection=True,
            export_format='GLB' if ext == '.glb' else 'GLTF_SEPARATE',
        )
    else:
        raise ValueError(f"Unsupported export format: {ext}")


def main():
    # Parse args after "--"
    argv = sys.argv
    if "--" in argv:
        argv = argv[argv.index("--") + 1:]
    else:
        print("Usage: blender --background --python blender_rig.py -- input.obj output.fbx")
        sys.exit(1)

    if len(argv) < 2:
        print("Need input and output paths")
        sys.exit(1)

    input_path = argv[0]
    output_path = argv[1]

    print(f"[rig] Input:  {input_path}")
    print(f"[rig] Output: {output_path}")

    clear_scene()

    print("[rig] Importing mesh...")
    mesh_obj = import_mesh(input_path)
    print(f"[rig] Mesh: {mesh_obj.name}, verts={len(mesh_obj.data.vertices)}")

    print("[rig] Creating humanoid armature...")
    armature_obj = create_humanoid_armature(mesh_obj)

    print("[rig] Applying automatic weights...")
    try:
        parent_with_automatic_weights(mesh_obj, armature_obj)
    except RuntimeError as e:
        print(f"[rig] WARNING: Automatic weights failed ({e})")
        print("[rig] Falling back to envelope weights...")
        bpy.ops.object.select_all(action='DESELECT')
        mesh_obj.select_set(True)
        armature_obj.select_set(True)
        bpy.context.view_layer.objects.active = armature_obj
        bpy.ops.object.parent_set(type='ARMATURE_ENVELOPE')

    print(f"[rig] Exporting to {output_path}...")
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    export_rigged(armature_obj, mesh_obj, output_path)
    print("[rig] Done.")


if __name__ == "__main__":
    main()
