import bpy
import json
import math
import os
import random
import struct
import zlib
from mathutils import Vector


ASSET_DIR = r"C:\dev\space_battle\assets\asterion_mk1"
TEXTURE_DIR = os.path.join(ASSET_DIR, "textures")
RENDER_DIR = os.path.join(ASSET_DIR, "renders")
BLEND_PATH = os.path.join(ASSET_DIR, "asterion_mk1.blend")
GLB_PATH = os.path.join(ASSET_DIR, "asterion_mk1.glb")
STATS_PATH = os.path.join(ASSET_DIR, "asterion_mk1_stats.json")

FORWARD = Vector((1.0, 0.0, 0.0))
STEFAN_BOLTZMANN_W_M2_K4 = 5.670374419e-8
RADIATOR_DESIGN = {
    "heat_load_w": 500_000.0,
    "design_margin": 1.20,
    "surface_temperature_k": 500.0,
    "infrared_emissivity": 0.88,
    "panel_fin_efficiency": 0.90,
    "deep_space_view_factor": 0.98,
    "emitting_faces": 2,
    "wing_count": 2,
    "modules_x": 4,
    "modules_span": 6,
    "wing_x_min_m": -10.6,
    "wing_x_max_m": -4.2,
    "wing_root_radius_m": 2.7,
    "wing_tip_radius_m": 11.9,
    "module_gap_m": 0.04,
    "system_areal_density_kg_m2": 8.0,
}
random.seed(12055)

os.makedirs(TEXTURE_DIR, exist_ok=True)
os.makedirs(RENDER_DIR, exist_ok=True)


def radiator_sizing():
    design = dict(RADIATOR_DESIGN)
    gross_x = design["wing_x_max_m"] - design["wing_x_min_m"]
    gross_span = design["wing_tip_radius_m"] - design["wing_root_radius_m"]
    cell_x = gross_x / design["modules_x"]
    cell_span = gross_span / design["modules_span"]
    active_x = cell_x - design["module_gap_m"]
    active_span = cell_span - design["module_gap_m"]
    active_area = (
        design["wing_count"]
        * design["modules_x"]
        * design["modules_span"]
        * active_x
        * active_span
    )
    one_face_flux = (
        design["infrared_emissivity"]
        * design["panel_fin_efficiency"]
        * design["deep_space_view_factor"]
        * STEFAN_BOLTZMANN_W_M2_K4
        * design["surface_temperature_k"] ** 4
    )
    two_face_flux = design["emitting_faces"] * one_face_flux
    design["gross_x_m"] = gross_x
    design["gross_span_m"] = gross_span
    design["cell_x_m"] = cell_x
    design["cell_span_m"] = cell_span
    design["active_module_x_m"] = active_x
    design["active_module_span_m"] = active_span
    design["effective_two_face_flux_w_m2"] = two_face_flux
    design["required_area_nominal_m2"] = design["heat_load_w"] / two_face_flux
    design["required_area_with_margin_m2"] = (
        design["heat_load_w"] * design["design_margin"] / two_face_flux
    )
    design["modeled_active_area_m2"] = active_area
    design["modeled_capacity_w"] = active_area * two_face_flux
    design["estimated_system_mass_kg"] = (
        active_area * design["system_areal_density_kg_m2"]
    )
    return design


def write_png(path, width, height, pixel_fn):
    """Write an RGB PNG using only Blender's standard Python runtime."""
    rows = bytearray()
    for y in range(height):
        rows.append(0)
        for x in range(width):
            r, g, b = pixel_fn(x, y)
            rows.extend((
                max(0, min(255, int(r))),
                max(0, min(255, int(g))),
                max(0, min(255, int(b))),
            ))

    def chunk(kind, data):
        return (
            struct.pack(">I", len(data))
            + kind
            + data
            + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
        )

    payload = b"\x89PNG\r\n\x1a\n"
    payload += chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
    payload += chunk(b"IDAT", zlib.compress(bytes(rows), 9))
    payload += chunk(b"IEND", b"")
    with open(path, "wb") as handle:
        handle.write(payload)


def make_texture_maps():
    size = 512

    def hull_albedo(x, y):
        n = random.randint(-7, 7)
        panel = (x % 96 < 3) or (y % 80 < 3)
        secondary = (x % 32 == 0 and 80 < y % 160 < 83)
        scratch = ((x * 17 + y * 29) % 997) < 2
        base = 122 + n
        if panel:
            base -= 36
        if secondary:
            base += 18
        if scratch:
            base += 42
        warm = 3 if ((x // 96 + y // 80) % 3 == 0) else 0
        return base + warm, base + 2, base + 4

    def hull_roughness(x, y):
        noise = random.randint(-14, 14)
        seam = (x % 96 < 3) or (y % 80 < 3)
        value = 158 + noise + (35 if seam else 0)
        return value, value, value

    def radiator_albedo(x, y):
        tile_x = x % 32
        tile_y = y % 24
        seam = tile_x < 2 or tile_y < 2
        base = 18 + random.randint(-3, 4)
        if seam:
            return 66, 43, 28
        trace = 18 if (tile_x in (8, 9, 23, 24)) else 0
        return base + trace, base + 8, base + 10

    write_png(os.path.join(TEXTURE_DIR, "hull_albedo.png"), size, size, hull_albedo)
    write_png(os.path.join(TEXTURE_DIR, "hull_roughness.png"), size, size, hull_roughness)
    write_png(os.path.join(TEXTURE_DIR, "radiator_albedo.png"), size, size, radiator_albedo)


def clear_scene():
    # Direct data removal works from Blender's Python Console even if the
    # startup cube was left in Edit Mode; object operators do not.
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    for datablocks in (
        bpy.data.meshes,
        bpy.data.curves,
        bpy.data.materials,
        bpy.data.cameras,
        bpy.data.lights,
    ):
        for block in list(datablocks):
            if block.users == 0:
                datablocks.remove(block)
    for collection in list(bpy.data.collections):
        if collection.name != "Collection":
            bpy.data.collections.remove(collection)
    base = bpy.data.collections.get("Collection")
    if base:
        base.name = "00_MASTER"


def collection(name):
    coll = bpy.data.collections.get(name)
    if coll is None:
        coll = bpy.data.collections.new(name)
        bpy.context.scene.collection.children.link(coll)
    return coll


COLLECTIONS = {}


def move_to_collection(obj, coll_name):
    coll = COLLECTIONS[coll_name]
    for owner in list(obj.users_collection):
        owner.objects.unlink(obj)
    coll.objects.link(obj)


def tag(obj, system, material_key, description="", mass_status="pending"):
    obj["system"] = system
    obj["material_key"] = material_key
    obj["description"] = description
    obj["mass_status"] = mass_status
    return obj


def bevel(obj, width=0.06, segments=1):
    modifier = obj.modifiers.new("Edge softening", "BEVEL")
    modifier.width = width
    modifier.segments = segments
    return obj


def box(name, loc, dims, coll, mat, description="", bevel_width=0.04):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.dimensions = dims
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if bevel_width:
        bevel(obj, bevel_width, 1)
    move_to_collection(obj, coll)
    return tag(obj, coll, mat, description)


def cylinder_between(name, a, b, radius, coll, mat, vertices=8, description=""):
    a = Vector(a)
    b = Vector(b)
    delta = b - a
    midpoint = (a + b) * 0.5
    bpy.ops.mesh.primitive_cylinder_add(
        vertices=vertices,
        radius=radius,
        depth=delta.length,
        end_fill_type="NGON",
        location=midpoint,
    )
    obj = bpy.context.object
    obj.name = name
    obj.rotation_mode = "QUATERNION"
    obj.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(delta.normalized())
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    move_to_collection(obj, coll)
    for poly in obj.data.polygons:
        poly.use_smooth = True
    return tag(obj, coll, mat, description)


def axial_cylinder(name, x, length, radius, coll, mat, vertices=16, description=""):
    return cylinder_between(
        name,
        (x - length * 0.5, 0, 0),
        (x + length * 0.5, 0, 0),
        radius,
        coll,
        mat,
        vertices,
        description,
    )


def sphere(name, loc, radius, coll, mat, segments=16, rings=8, scale=(1, 1, 1), description=""):
    bpy.ops.mesh.primitive_uv_sphere_add(
        segments=segments,
        ring_count=rings,
        radius=radius,
        location=loc,
    )
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    move_to_collection(obj, coll)
    for poly in obj.data.polygons:
        poly.use_smooth = True
    return tag(obj, coll, mat, description)


def frustum(
    name,
    x,
    length,
    radius_front,
    radius_aft,
    coll,
    mat,
    vertices=16,
    description="",
    capped=True,
):
    bpy.ops.mesh.primitive_cone_add(
        vertices=vertices,
        radius1=radius_aft,
        radius2=radius_front,
        depth=length,
        end_fill_type="NGON" if capped else "NOTHING",
        location=(x, 0, 0),
        rotation=(0, math.pi * 0.5, 0),
    )
    obj = bpy.context.object
    obj.name = name
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    move_to_collection(obj, coll)
    for poly in obj.data.polygons:
        poly.use_smooth = True
    return tag(obj, coll, mat, description)


def cone_between(
    name,
    a,
    b,
    radius_at_a,
    radius_at_b,
    coll,
    mat,
    vertices=10,
    description="",
    capped=False,
):
    a = Vector(a)
    b = Vector(b)
    delta = b - a
    midpoint = (a + b) * 0.5
    bpy.ops.mesh.primitive_cone_add(
        vertices=vertices,
        radius1=radius_at_a,
        radius2=radius_at_b,
        depth=delta.length,
        end_fill_type="NGON" if capped else "NOTHING",
        location=midpoint,
    )
    obj = bpy.context.object
    obj.name = name
    obj.rotation_mode = "QUATERNION"
    obj.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(delta.normalized())
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    move_to_collection(obj, coll)
    for poly in obj.data.polygons:
        poly.use_smooth = True
    return tag(obj, coll, mat, description)


def torus(name, loc, major_radius, minor_radius, coll, mat, rotation=(0, 0, 0), description=""):
    bpy.ops.mesh.primitive_torus_add(
        major_radius=major_radius,
        minor_radius=minor_radius,
        major_segments=20,
        minor_segments=6,
        location=loc,
        rotation=rotation,
    )
    obj = bpy.context.object
    obj.name = name
    move_to_collection(obj, coll)
    for poly in obj.data.polygons:
        poly.use_smooth = True
    return tag(obj, coll, mat, description)


def annulus(
    name,
    loc,
    inner_radius,
    outer_radius,
    depth,
    coll,
    mat,
    axis="Z",
    segments=24,
    description="",
):
    """Low-poly rectangular-section ring, avoiding a decorative torus profile."""
    vertices = []
    for axial in (-depth * 0.5, depth * 0.5):
        for radius in (outer_radius, inner_radius):
            for i in range(segments):
                angle = math.tau * i / segments
                radial_a = radius * math.cos(angle)
                radial_b = radius * math.sin(angle)
                if axis == "X":
                    vertices.append((loc[0] + axial, loc[1] + radial_a, loc[2] + radial_b))
                else:
                    vertices.append((loc[0] + radial_a, loc[1] + radial_b, loc[2] + axial))

    def vid(layer, radius_index, index):
        return layer * segments * 2 + radius_index * segments + (index % segments)

    faces = []
    for i in range(segments):
        j = (i + 1) % segments
        # Outer and inner cylindrical walls.
        faces.append((vid(0, 0, i), vid(0, 0, j), vid(1, 0, j), vid(1, 0, i)))
        faces.append((vid(0, 1, j), vid(0, 1, i), vid(1, 1, i), vid(1, 1, j)))
        # Two flat annular faces.
        faces.append((vid(1, 0, i), vid(1, 0, j), vid(1, 1, j), vid(1, 1, i)))
        faces.append((vid(0, 0, j), vid(0, 0, i), vid(0, 1, i), vid(0, 1, j)))

    mesh = bpy.data.meshes.new(name + "_Mesh")
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    COLLECTIONS[coll].objects.link(obj)
    bevel(obj, min(0.025, depth * 0.18), 1)
    return tag(obj, coll, mat, description)


def docking_capture_petal(name, angle, coll, mat):
    """Tapered NDS-style guide petal projecting forward from the soft-capture ring."""
    base_x, tip_x = 21.58, 21.98
    base_r, tip_r = 0.61, 0.91
    base_half_t, tip_half_t = 0.15, 0.085
    half_thickness = 0.045
    vertices = []
    for x, radius, half_tangent in (
        (base_x, base_r, base_half_t),
        (tip_x, tip_r, tip_half_t),
    ):
        for radial_offset, tangent_offset in (
            (-half_thickness, -half_tangent),
            (-half_thickness, half_tangent),
            (half_thickness, half_tangent),
            (half_thickness, -half_tangent),
        ):
            r = radius + radial_offset
            y = r * math.cos(angle) - tangent_offset * math.sin(angle)
            z = r * math.sin(angle) + tangent_offset * math.cos(angle)
            vertices.append((x, y, z))
    faces = [
        (0, 1, 2, 3),
        (4, 7, 6, 5),
        (0, 4, 5, 1),
        (1, 5, 6, 2),
        (2, 6, 7, 3),
        (3, 7, 4, 0),
    ]
    mesh = bpy.data.meshes.new(name + "_Mesh")
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    COLLECTIONS[coll].objects.link(obj)
    bevel(obj, 0.018, 1)
    return tag(
        obj,
        coll,
        mat,
        "One of three 120-degree soft-capture guide petals; funnels initial contact into the capture ring.",
    )


def pipe_curve(name, points, radius, coll, mat, description=""):
    curve = bpy.data.curves.new(name + "_Curve", "CURVE")
    curve.dimensions = "3D"
    curve.resolution_u = 1
    curve.bevel_depth = radius
    curve.bevel_resolution = 1
    curve.resolution_u = 2
    spline = curve.splines.new("BEZIER")
    spline.bezier_points.add(len(points) - 1)
    for bp, co in zip(spline.bezier_points, points):
        bp.co = co
        bp.handle_left_type = "AUTO"
        bp.handle_right_type = "AUTO"
    obj = bpy.data.objects.new(name, curve)
    COLLECTIONS[coll].objects.link(obj)
    return tag(obj, coll, mat, description)


def create_empty(name, coll, loc=(0, 0, 0), description=""):
    obj = bpy.data.objects.new(name, None)
    obj.location = loc
    COLLECTIONS[coll].objects.link(obj)
    obj["system"] = coll
    obj["description"] = description
    return obj


def build_master_reference():
    radiator = radiator_sizing()
    master = create_empty(
        "ASM_ASTERION_MK1",
        "00_MASTER",
        description="Asterion Mk I engineering reference assembly; +X forward, +Y port, +Z dorsal.",
    )
    master["asset_name"] = "Asterion Mk I"
    master["revision"] = 3
    master["units"] = "metres"
    master["overall_length_m"] = 43.0
    master["crew_count"] = 2
    master["gross_pressurized_volume_m3"] = 58.0
    master["drive_assumption"] = "Compact fusion drive capable of mission-average 1 g for several days; performance remains parameterized."
    master["mass_model_status"] = "Subsystem mass model pending"
    master["coordinate_convention"] = "+X forward, +Y port, +Z dorsal"
    master["radiator_heat_load_w"] = radiator["heat_load_w"]
    master["radiator_surface_temperature_k"] = radiator["surface_temperature_k"]
    master["radiator_modeled_active_area_m2"] = round(radiator["modeled_active_area_m2"], 3)
    master["radiator_modeled_capacity_w"] = round(radiator["modeled_capacity_w"], 1)
    master["radiator_orientation"] = "Radial XY wings; emitting-face normals are +Z and -Z so the vehicle is edge-on."
    master["radiator_estimated_system_mass_kg"] = round(radiator["estimated_system_mass_kg"], 1)
    return master


def build_truss():
    ring_x = [-12.0, -8.0, -4.0, 0.0, 4.0, 8.0]
    radius = 2.35
    nodes = {}
    for bay_index, x in enumerate(ring_x):
        ring_nodes = []
        for i in range(8):
            angle = math.radians(22.5 + i * 45.0)
            node = Vector((x, radius * math.cos(angle), radius * math.sin(angle)))
            ring_nodes.append(node)
            cylinder_between(
                f"TRUSS_NodeSleeve_{bay_index:02d}_{i:02d}",
                (x - 0.17, node.y, node.z),
                (x + 0.17, node.y, node.z),
                0.15,
                "01_STRUCTURE",
                "truss",
                vertices=10,
                description="Machined truss-node sleeve and local equipment hardpoint.",
            )
            if 0 < bay_index < len(ring_x) - 1:
                if i % 2 == 0:
                    cylinder_between(
                        f"TRUSS_SSASPeripheralBolt_{bay_index:02d}_{i:02d}",
                        (x - 0.245, node.y, node.z),
                        (x + 0.245, node.y, node.z),
                        0.032,
                        "01_STRUCTURE",
                        "dark_metal",
                        vertices=8,
                        description="Captive peripheral segment bolt based on the ISS segment-to-segment attachment load path.",
                    )
                    for side in (-1, 1):
                        cylinder_between(
                            f"TRUSS_SSASBoltHead_{bay_index:02d}_{i:02d}_{side:+d}",
                            (x + side * 0.19, node.y, node.z),
                            (x + side * 0.25, node.y, node.z),
                            0.072,
                            "01_STRUCTURE",
                            "dark_metal",
                            vertices=8,
                            description="External captive bolt head and washer for a replaceable truss segment joint.",
                        )
                else:
                    cone_between(
                        f"TRUSS_SSASAlignmentCone_{bay_index:02d}_{i:02d}",
                        (x - 0.21, node.y, node.z),
                        (x + 0.21, node.y, node.z),
                        0.105,
                        0.060,
                        "01_STRUCTURE",
                        "dark_metal",
                        vertices=10,
                        description="Cup-and-cone alignment feature inspired by the ISS segment-to-segment attachment system.",
                        capped=True,
                    )
        nodes[x] = ring_nodes
        for i in range(8):
            cylinder_between(
                f"TRUSS_Ring_{bay_index:02d}_{i:02d}",
                ring_nodes[i],
                ring_nodes[(i + 1) % 8],
                0.085,
                "01_STRUCTURE",
                "truss",
                vertices=8,
                description="Octagonal transverse truss member.",
            )

    for bay in range(len(ring_x) - 1):
        x0, x1 = ring_x[bay], ring_x[bay + 1]
        for i in range(8):
            cylinder_between(
                f"TRUSS_Longeron_{bay:02d}_{i:02d}",
                nodes[x0][i],
                nodes[x1][i],
                0.105,
                "01_STRUCTURE",
                "truss",
                vertices=8,
                description="Primary axial thrust and recoil longeron.",
            )
            diagonal_target = (i + (1 if bay % 2 == 0 else -1)) % 8
            cylinder_between(
                f"TRUSS_Diagonal_{bay:02d}_{i:02d}",
                nodes[x0][i],
                nodes[x1][diagonal_target],
                0.064,
                "01_STRUCTURE",
                "truss",
                vertices=8,
                description="Alternating shear diagonal.",
            )

    cylinder_between(
        "STRUCTURE_CentralSpine",
        (-12.6, 0, 0),
        (8.8, 0, 0),
        0.22,
        "01_STRUCTURE",
        "truss",
        vertices=12,
        description="Central compression/tension spine and principal alignment datum.",
    )
    for x in ring_x:
        for i in (1, 3, 5, 7):
            cylinder_between(
                f"TRUSS_Radial_{int(x + 12):02d}_{i}",
                (x, 0, 0),
                nodes[x][i],
                0.06,
                "01_STRUCTURE",
                "truss",
                vertices=8,
                description="Radial equipment and load-transfer brace.",
            )


def build_habitat():
    axial_cylinder(
        "HAB_PressureCylinder",
        17.2,
        4.8,
        2.08,
        "03_HABITAT",
        "hull",
        vertices=20,
        description="Two-person pressure vessel; approximately 58 cubic metres gross internal volume.",
    )
    sphere(
        "HAB_AftDome",
        (14.8, 0, 0),
        2.08,
        "03_HABITAT",
        "hull",
        segments=20,
        rings=10,
        scale=(0.42, 1.0, 1.0),
        description="Pressure-vessel aft dome and storm-shelter boundary.",
    )
    sphere(
        "HAB_ForwardDome",
        (19.6, 0, 0),
        2.08,
        "03_HABITAT",
        "hull",
        segments=20,
        rings=10,
        scale=(0.42, 1.0, 1.0),
        description="Pressure-vessel forward dome.",
    )
    frustum(
        "HAB_DockingPressureTunnel",
        20.67,
        1.28,
        0.48,
        1.35,
        "03_HABITAT",
        "hull",
        vertices=20,
        description="Pressure tunnel tapering from the habitat dome to an 0.8 m docking passage.",
    )
    hard_ring = annulus(
        "HAB_NDS_HardCaptureRing",
        (21.31, 0, 0),
        0.40,
        0.80,
        0.18,
        "03_HABITAT",
        "dark_metal",
        axis="X",
        description="Rectangular-section NDS/IDSS-style hard-capture ring transmitting docking loads into the pressure tunnel.",
    )
    hard_ring["interface_family"] = "NASA Docking System / IDSS-like"
    hard_ring["passage_diameter_m"] = 0.8
    annulus(
        "HAB_NDS_SoftCaptureRing",
        (21.62, 0, 0),
        0.43,
        0.68,
        0.10,
        "03_HABITAT",
        "light_metal",
        axis="X",
        description="Deployable soft-capture ring used for initial alignment before hard mate.",
    )
    torus(
        "HAB_NDS_PressureSeal",
        (21.41, 0, 0),
        0.43,
        0.026,
        "03_HABITAT",
        "seal",
        rotation=(0, math.pi * 0.5, 0),
        description="Replaceable dual-pressure-seal datum inside the hard-capture ring.",
    )
    for i in range(6):
        angle = math.radians(i * 60.0 + 30.0)
        y0, z0 = 0.68 * math.cos(angle), 0.68 * math.sin(angle)
        y1, z1 = 0.58 * math.cos(angle), 0.58 * math.sin(angle)
        cylinder_between(
            f"HAB_NDS_SoftCaptureLink_{i:02d}",
            (21.31, y0, z0),
            (21.62, y1, z1),
            0.026,
            "03_HABITAT",
            "hydraulic",
            vertices=8,
            description="Compliant soft-capture ring support and retraction linkage.",
        )
    for i in range(3):
        docking_capture_petal(
            f"HAB_NDS_GuidePetal_{i + 1:02d}",
            math.radians(i * 120.0),
            "03_HABITAT",
            "light_metal",
        )
    for i in range(12):
        angle = math.radians(i * 30.0 + 15.0)
        y = 0.70 * math.cos(angle)
        z = 0.70 * math.sin(angle)
        hook = box(
            f"HAB_NDS_HardCaptureHook_{i + 1:02d}",
            (21.31, y, z),
            (0.20, 0.10, 0.14),
            "03_HABITAT",
            "dark_metal",
            description="Peripheral hard-capture hook/bolt transmitting axial and shear docking loads.",
            bevel_width=0.018,
        )
        hook.rotation_euler.x = angle

    for x in (15.45, 16.65, 17.85, 19.05):
        torus(
            f"HAB_WhippleFrame_{x:.2f}",
            (x, 0, 0),
            2.16,
            0.055,
            "03_HABITAT",
            "light_metal",
            rotation=(0, math.pi * 0.5, 0),
            description="Stand-off frame for segmented Whipple shielding.",
        )
    for side in (-1, 1):
        for x in (15.9, 17.3, 18.7):
            box(
                f"HAB_WhipplePanel_{side:+d}_{x:.1f}",
                (x, side * 2.16, 0.15),
                (1.22, 0.055, 1.25),
                "03_HABITAT",
                "light_metal",
                description="Replaceable thin external micrometeoroid bumper panel.",
                bevel_width=0.025,
            )
    for y in (-1.15, 1.15):
        sphere(
            f"HAB_OpticalPort_{y:+.2f}",
            (20.42, y, 0.45),
            0.18,
            "07_SENSORS",
            "glass",
            segments=12,
            rings=6,
            scale=(0.35, 1.0, 1.0),
            description="Small protected optical navigation and situational-awareness port.",
        )
    box(
        "HAB_WaterStormShelter",
        (14.55, 0, 0),
        (0.42, 2.9, 2.9),
        "03_HABITAT",
        "water",
        description="Aft water and consumables wall surrounding the two-person storm shelter.",
        bevel_width=0.15,
    )
    for side in (-1, 1):
        cylinder_between(
            f"HAB_EVAHandrail_{side:+d}",
            (15.45, side * 1.73, 1.28),
            (19.15, side * 1.73, 1.28),
            0.025,
            "03_HABITAT",
            "light_metal",
            vertices=8,
            description="Stand-off EVA translation handrail.",
        )


def build_propellant():
    axial_cylinder(
        "FUEL_MainTankCylinder",
        10.35,
        4.75,
        2.28,
        "04_FUEL",
        "tank",
        vertices=20,
        description="Vacuum-insulated cryogenic reaction-mass/fusion-fuel tank; contents are a simulation parameter.",
    )
    sphere(
        "FUEL_AftDome",
        (7.98, 0, 0),
        2.28,
        "04_FUEL",
        "tank",
        segments=20,
        rings=10,
        scale=(0.42, 1, 1),
        description="Tank aft pressure dome.",
    )
    sphere(
        "FUEL_ForwardDome",
        (12.72, 0, 0),
        2.28,
        "04_FUEL",
        "tank",
        segments=20,
        rings=10,
        scale=(0.42, 1, 1),
        description="Tank forward pressure dome.",
    )
    for x in (8.25, 9.55, 10.85, 12.15):
        torus(
            f"FUEL_SupportBand_{x:.2f}",
            (x, 0, 0),
            2.33,
            0.075,
            "04_FUEL",
            "dark_metal",
            rotation=(0, math.pi * 0.5, 0),
            description="Tank support band and MLI restraint.",
        )
    for y in (-0.55, 0.55):
        pipe_curve(
            f"FUEL_Manifold_{y:+.2f}",
            [
                (7.9, y, -1.55),
                (7.5, y * 1.8, -2.05),
                (6.6, y * 2.8, -2.12),
            ],
            0.075,
            "06_UTILITIES",
            "fuel_line",
            description="Redundant tank outlet and isolation manifold.",
        )


def build_shadow_shield_and_engine():
    frustum(
        "SHIELD_ForwardHydride",
        -12.42,
        0.45,
        2.72,
        2.48,
        "02_PROPULSION",
        "shield",
        vertices=20,
        description="Hydrogen-rich crew-side neutron shield in the drive shadow cone.",
    )
    frustum(
        "SHIELD_HotSideHighZ",
        -12.72,
        0.18,
        2.48,
        2.32,
        "02_PROPULSION",
        "dark_metal",
        vertices=20,
        description="High-Z hot-side gamma attenuation layer.",
    )
    for i in range(8):
        angle = math.radians(22.5 + i * 45)
        y = 2.55 * math.cos(angle)
        z = 2.55 * math.sin(angle)
        cylinder_between(
            f"ENGINE_ShieldStay_{i:02d}",
            (-12.0, y * 0.92, z * 0.92),
            (-13.45, y * 0.66, z * 0.66),
            0.075,
            "02_PROPULSION",
            "truss",
            description="Shield-to-drive load-transfer stay.",
        )

    axial_cylinder(
        "ENGINE_ReactorChamber",
        -14.65,
        2.25,
        1.48,
        "02_PROPULSION",
        "engine",
        vertices=20,
        description="Speculative fusion reaction chamber and superconducting magnet envelope.",
    )
    for x in (-13.65, -14.25, -14.85, -15.45):
        torus(
            f"ENGINE_MagnetCoil_{abs(x):.2f}",
            (x, 0, 0),
            1.55,
            0.13,
            "02_PROPULSION",
            "copper",
            rotation=(0, math.pi * 0.5, 0),
            description="External superconducting/magnetic confinement coil casing.",
        )
    frustum(
        "ENGINE_MagneticNozzle",
        -17.35,
        4.2,
        1.25,
        2.62,
        "02_PROPULSION",
        "engine",
        vertices=20,
        description="Magnetic nozzle and radiation-facing aft structure.",
        capped=False,
    )
    frustum(
        "ENGINE_NozzleInterior",
        -17.62,
        3.85,
        0.96,
        2.28,
        "02_PROPULSION",
        "nozzle",
        vertices=20,
        description="Refractory nozzle interior; kept visually dark except for residual thermal emission.",
        capped=False,
    )
    torus(
        "ENGINE_NozzleLip",
        (-19.47, 0, 0),
        2.44,
        0.14,
        "02_PROPULSION",
        "dark_metal",
        rotation=(0, math.pi * 0.5, 0),
        description="Aft magnetic-nozzle support ring.",
    )
    torus(
        "ENGINE_NozzleThroat",
        (-15.78, 0, 0),
        1.02,
        0.09,
        "02_PROPULSION",
        "nozzle",
        rotation=(0, math.pi * 0.5, 0),
        description="Visible magnetic-nozzle throat and aft field-coil datum.",
    )
    for i in range(4):
        angle = math.radians(45.0 + i * 90.0)
        y0 = 1.38 * math.cos(angle)
        z0 = 1.38 * math.sin(angle)
        y1 = 2.08 * math.cos(angle)
        z1 = 2.08 * math.sin(angle)
        cylinder_between(
            f"ENGINE_GimbalActuator_{i:02d}",
            (-14.25, y0, z0),
            (-13.05, y1, z1),
            0.095,
            "02_PROPULSION",
            "hydraulic",
            vertices=12,
            description="Drive alignment/gimbal actuator transmitting thrust into the shadow-shield ring.",
        )
        cylinder_between(
            f"ENGINE_GimbalStay_{i:02d}",
            (-15.25, y0 * 0.92, z0 * 0.92),
            (-13.05, y1, z1),
            0.065,
            "02_PROPULSION",
            "truss",
            vertices=8,
            description="Triangulated drive thrust stay.",
        )

    radiator = radiator_sizing()
    x_min = radiator["wing_x_min_m"]
    x_max = radiator["wing_x_max_m"]
    root_radius = radiator["wing_root_radius_m"]
    tip_radius = radiator["wing_tip_radius_m"]
    cell_x = radiator["cell_x_m"]
    cell_span = radiator["cell_span_m"]
    active_x = radiator["active_module_x_m"]
    active_span = radiator["active_module_span_m"]
    core_thickness = 0.025
    face_thickness = 0.004
    face_z = core_thickness * 0.5 + face_thickness * 0.5

    for side in (-1, 1):
        side_code = "P" if side > 0 else "S"
        root_y = side * root_radius
        tip_y = side * tip_radius
        inner_header_y = side * (root_radius - 0.16)
        wing_root = create_empty(
            f"THERMAL_RadiatorWing_{side_code}",
            "02_PROPULSION",
            (0, 0, 0),
            description="Two-sided radial-plane radiator wing. Its +Z/-Z face normals keep the vehicle nearly edge-on to the emitting surfaces.",
        )
        wing_root["heat_load_share_w"] = radiator["heat_load_w"] / radiator["wing_count"]
        wing_root["active_area_m2"] = radiator["modeled_active_area_m2"] / radiator["wing_count"]
        wing_root["modeled_capacity_w"] = radiator["modeled_capacity_w"] / radiator["wing_count"]
        wing_root["surface_temperature_k"] = radiator["surface_temperature_k"]
        wing_root["infrared_emissivity"] = radiator["infrared_emissivity"]
        wing_root["panel_fin_efficiency"] = radiator["panel_fin_efficiency"]
        wing_root["deep_space_view_factor"] = radiator["deep_space_view_factor"]
        wing_root["estimated_system_mass_kg"] = radiator["estimated_system_mass_kg"] / radiator["wing_count"]
        wing_root["orientation"] = "XY radial plane; broad-face normals +Z/-Z"
        wing_root["coolant_architecture"] = "Four independently isolated dual-pass circuits with root supply/return headers"

        # Twenty-four separate channelized sandwich modules per wing. The gaps
        # contain frame rails and permit thermal expansion without buckling one
        # monolithic sheet.
        for ix in range(radiator["modules_x"]):
            x = x_min + (ix + 0.5) * cell_x
            for iy in range(radiator["modules_span"]):
                radius = root_radius + (iy + 0.5) * cell_span
                y = side * radius
                module_name = f"THERMAL_{side_code}_Module_{ix + 1:02d}_{iy + 1:02d}"
                core = box(
                    f"{module_name}_HoneycombCore",
                    (x, y, 0),
                    (active_x, active_span, core_thickness),
                    "02_PROPULSION",
                    "light_metal",
                    description="Bonded aluminium/composite honeycomb core carrying in-plane thrust loads and spacing the two radiator facesheets.",
                    bevel_width=0.006,
                )
                core.parent = wing_root
                core["circuit_id"] = ix + 1
                core["module_row"] = iy + 1
                core["active_planform_area_m2"] = active_x * active_span
                for face_sign in (-1, 1):
                    face = box(
                        f"{module_name}_{'Dorsal' if face_sign > 0 else 'Ventral'}Face",
                        (x, y, face_sign * face_z),
                        (active_x, active_span, face_thickness),
                        "02_PROPULSION",
                        "radiator",
                        description="High-emissivity, low-solar-absorptivity facesheet bonded to an embedded coolant circuit and emitting directly to deep space.",
                        bevel_width=0.003,
                    )
                    face.parent = wing_root
                    face["emitting_normal"] = "+Z" if face_sign > 0 else "-Z"
                    face["infrared_emissivity"] = radiator["infrared_emissivity"]
                    face["solar_absorptivity_design"] = 0.12
                    face["one_face_active_area_m2"] = active_x * active_span

        # Grid rails sit in the module gaps, so they provide a continuous load
        # path without masking useful emitting area.
        for ix in range(radiator["modules_x"] + 1):
            x = x_min + ix * cell_x
            rail = box(
                f"THERMAL_{side_code}_RadialFrame_{ix + 1:02d}",
                (x, side * (root_radius + tip_radius) * 0.5, 0),
                (0.035, radiator["gross_span_m"] + 0.08, 0.075),
                "02_PROPULSION",
                "dark_metal",
                description="Radial wing rail located in a thermal-expansion gap between radiator modules.",
                bevel_width=0.008,
            )
            rail.parent = wing_root
        for iy in range(radiator["modules_span"] + 1):
            y = side * (root_radius + iy * cell_span)
            rail = box(
                f"THERMAL_{side_code}_ChordFrame_{iy + 1:02d}",
                ((x_min + x_max) * 0.5, y, 0),
                (radiator["gross_x_m"] + 0.08, 0.035, 0.075),
                "02_PROPULSION",
                "dark_metal",
                description="Chordwise wing rail carrying module edge loads and providing a replaceable bolted interface.",
                bevel_width=0.008,
            )
            rail.parent = wing_root

        hinge_beam = cylinder_between(
            f"THERMAL_{side_code}_RootHingeBeam",
            (x_min - 0.06, root_y, 0),
            (x_max + 0.06, root_y, 0),
            0.085,
            "02_PROPULSION",
            "dark_metal",
            vertices=12,
            description="Continuous root hinge beam carrying deployed-wing shear into three truss-mounted support stations.",
        )
        hinge_beam.parent = wing_root
        tip_beam = cylinder_between(
            f"THERMAL_{side_code}_TipCloseoutBeam",
            (x_min - 0.04, tip_y, 0),
            (x_max + 0.04, tip_y, 0),
            0.065,
            "02_PROPULSION",
            "dark_metal",
            vertices=10,
            description="Outboard closeout beam tying the radial frame rails together.",
        )
        tip_beam.parent = wing_root

        # Separate supply/return root headers feed four independently valved
        # U-flow circuits. A puncture therefore costs at most one quarter of a
        # wing instead of draining the complete radiator.
        for role, z in (("Supply", 0.08), ("Return", -0.08)):
            header = cylinder_between(
                f"THERMAL_{side_code}_{role}Header",
                (x_min, inner_header_y, z),
                (x_max, inner_header_y, z),
                0.035,
                "02_PROPULSION",
                "coolant",
                vertices=10,
                description=f"Root {role.lower()} manifold for the four parallel high-temperature radiator circuits.",
            )
            header.parent = wing_root
            header["nominal_outer_diameter_m"] = 0.07

        for ix in range(radiator["modules_x"]):
            x = x_min + (ix + 0.5) * cell_x
            circuit = pipe_curve(
                f"THERMAL_{side_code}_EmbeddedCircuit_{ix + 1:02d}",
                [
                    (x - 0.22, inner_header_y, 0.08),
                    (x - 0.22, root_y, 0),
                    (x - 0.22, side * (tip_radius - 0.06), 0),
                    (x + 0.22, side * (tip_radius - 0.06), 0),
                    (x + 0.22, root_y, 0),
                    (x + 0.22, inner_header_y, -0.08),
                ],
                0.012,
                "02_PROPULSION",
                "coolant",
                description="Embedded dual-pass coolant tube bonded through six modular condenser panels; independent isolation limits puncture losses.",
            )
            circuit.parent = wing_root
            circuit["circuit_id"] = ix + 1
            for role, x_offset, z in (("Supply", -0.22, 0.08), ("Return", 0.22, -0.08)):
                valve = box(
                    f"THERMAL_{side_code}_Circuit{ix + 1:02d}_{role}Valve",
                    (x + x_offset, inner_header_y, z),
                    (0.13, 0.16, 0.12),
                    "02_PROPULSION",
                    "dark_metal",
                    description=f"Normally open remotely actuated {role.lower()} isolation valve for radiator circuit {ix + 1}.",
                    bevel_width=0.018,
                )
                valve.parent = wing_root

        # The root beam is not simply attached to a skin panel. Three four-bar
        # support stations terminate on the existing upper/lower primary
        # longerons and provide both in-plane and out-of-plane restraint.
        for station_index, x in enumerate((-10.15, -7.40, -4.65), start=1):
            bracket = box(
                f"THERMAL_{side_code}_RootBracket_{station_index:02d}",
                (x, root_y, 0),
                (0.30, 0.18, 0.34),
                "02_PROPULSION",
                "dark_metal",
                description="Bolted radiator hinge bracket joining the continuous root beam to a four-bar truss support.",
                bevel_width=0.025,
            )
            bracket.parent = wing_root
            for z_sign in (-1, 1):
                for x_offset in (-0.32, 0.32):
                    support = cylinder_between(
                        f"THERMAL_{side_code}_RootSupport_{station_index:02d}_{z_sign:+d}_{x_offset:+.2f}",
                        (x + x_offset, side * 2.17, z_sign * 0.90),
                        (x, root_y, z_sign * 0.11),
                        0.055,
                        "02_PROPULSION",
                        "truss",
                        vertices=8,
                        description="Triangulated root member terminating on a primary truss longeron and the radiator hinge bracket.",
                    )
                    support.parent = wing_root
        for actuator_index, x in enumerate((-9.65, -5.15), start=1):
            actuator = cylinder_between(
                f"THERMAL_{side_code}_HingeActuator_{actuator_index:02d}",
                (x - 0.16, root_y, 0),
                (x + 0.16, root_y, 0),
                0.125,
                "02_PROPULSION",
                "hydraulic",
                vertices=12,
                description="Motor/brake sleeve on the radiator deployment hinge; locked after deployment.",
            )
            actuator.parent = wing_root


def build_gun():
    gun_root = create_empty(
        "GUN_120MM_RotatingAssembly",
        "05_ARMAMENT",
        (0, 0, 0),
        description="Yawing 120 mm weapon assembly. Its Z axis is coincident with the dorsal ring axis; the physical ring plane is recorded separately.",
    )
    gun_root["calibre_mm"] = 120
    gun_root["barrel_class"] = "L/55"
    gun_root["barrel_length_m"] = 6.6
    gun_root["nominal_recoil_travel_m"] = 0.48
    gun_root["physical_yaw_ring_z_m"] = 2.55
    gun_root["trunnion_and_bore_axis_z_m"] = 3.7
    gun_root["ready_magazine_capacity_rounds"] = 14
    gun_root["feed_architecture"] = "Rotary carrier drum, powered elevator, loading tray, and axial rammer"
    gun_root["momentum_conservation_required"] = True

    ring = annulus(
        "GUN_SlewBearing",
        (0, 0, 2.50),
        0.88,
        1.38,
        0.16,
        "05_ARMAMENT",
        "dark_metal",
        axis="Z",
        description="Flat rectangular-section slew bearing; the previous bowl-like torus has been removed.",
    )
    ring.parent = gun_root
    for i in range(16):
        angle = math.radians(i * 22.5)
        x = 1.16 * math.cos(angle)
        y = 1.16 * math.sin(angle)
        bolt = cylinder_between(
            f"GUN_SlewBearingBolt_{i + 1:02d}",
            (x, y, 2.56),
            (x, y, 2.66),
            0.038,
            "05_ARMAMENT",
            "light_metal",
            vertices=8,
            description="Captive slew-bearing bolt transferring recoil and yaw loads into the dorsal structure.",
        )
        bolt.parent = gun_root

    for y in (-0.86, 0.86):
        upright = box(
            f"GUN_YokeUpright_{y:+.2f}",
            (-0.25, y, 3.28),
            (0.23, 0.16, 1.42),
            "05_ARMAMENT",
            "gunmetal",
            description="Skeletal elevation-yoke upright.",
            bevel_width=0.035,
        )
        upright.parent = gun_root
        for x, z in ((-0.8, 2.7), (0.35, 2.7)):
            brace = cylinder_between(
                f"GUN_YokeBrace_{y:+.2f}_{x:+.2f}",
                (x, y, z),
                (-0.25, y, 3.85),
                0.07,
                "05_ARMAMENT",
                "gunmetal",
                description="Open truss brace for elevation yoke.",
            )
            brace.parent = gun_root

    trunnion = cylinder_between(
        "GUN_Trunnion",
        (-0.18, -1.08, 3.7),
        (-0.18, 1.08, 3.7),
        0.19,
        "05_ARMAMENT",
        "gunmetal",
        vertices=16,
        description="Elevation trunnion and bearing spindle.",
    )
    trunnion.parent = gun_root
    breech = box(
        "GUN_BreechHousing",
        (-0.95, 0, 3.7),
        (1.35, 1.05, 0.88),
        "05_ARMAMENT",
        "gunmetal",
        description="Forged 120 mm breech housing carrying chamber pressure, trunnion, wedge guides, and extractors.",
        bevel_width=0.065,
    )
    breech.parent = gun_root
    chamber_ring = annulus(
        "GUN_ChamberReinforcementRing",
        (-0.29, 0, 3.70),
        0.078,
        0.29,
        0.20,
        "05_ARMAMENT",
        "barrel",
        axis="X",
        segments=20,
        description="Reinforced chamber/barrel interface immediately forward of the breech housing.",
    )
    chamber_ring.parent = gun_root
    wedge = box(
        "GUN_VerticalSlidingBreechWedge",
        (-1.58, 0, 3.68),
        (0.18, 0.82, 0.80),
        "05_ARMAMENT",
        "barrel",
        description="Replaceable vertical sliding wedge that closes the chamber and carries firing pressure into the breech housing.",
        bevel_width=0.025,
    )
    wedge.parent = gun_root
    for y in (-0.47, 0.47):
        guide = box(
            f"GUN_BreechWedgeGuide_{y:+.2f}",
            (-1.57, y, 3.70),
            (0.24, 0.08, 0.98),
            "05_ARMAMENT",
            "gunmetal",
            description="Machined guide rail constraining vertical travel of the breech wedge.",
            bevel_width=0.018,
        )
        guide.parent = gun_root
    for y in (-0.29, 0.29):
        actuator = cylinder_between(
            f"GUN_BreechOperatingRod_{y:+.2f}",
            (-1.44, y, 4.02),
            (-1.44, y, 4.48),
            0.045,
            "05_ARMAMENT",
            "hydraulic",
            vertices=10,
            description="Redundant powered operating rod raising and lowering the sliding breech wedge.",
        )
        actuator.parent = gun_root
        extractor = box(
            f"GUN_CaseExtractor_{y:+.2f}",
            (-1.70, y, 3.70),
            (0.08, 0.13, 0.24),
            "05_ARMAMENT",
            "dark_metal",
            description="Spring-loaded extractor claw acting on the cartridge rim during breech opening.",
            bevel_width=0.012,
        )
        extractor.parent = gun_root
    operating_bridge = box(
        "GUN_BreechOperatingBridge",
        (-1.43, 0, 4.46),
        (0.42, 0.72, 0.16),
        "05_ARMAMENT",
        "gunmetal",
        description="Crosshead synchronizing the two breech operating rods.",
        bevel_width=0.025,
    )
    operating_bridge.parent = gun_root

    barrel = cylinder_between(
        "GUN_Barrel_L55",
        (-0.25, 0, 3.7),
        (6.35, 0, 3.7),
        0.112,
        "05_ARMAMENT",
        "barrel",
        vertices=16,
        description="120 mm L/55-class smoothbore barrel; bore represented at muzzle.",
    )
    barrel.parent = gun_root
    sleeve = cylinder_between(
        "GUN_ThermalSleeve",
        (0.05, 0, 3.7),
        (3.25, 0, 3.7),
        0.145,
        "05_ARMAMENT",
        "light_metal",
        vertices=16,
        description="Low-mass barrel thermal sleeve and alignment shroud.",
    )
    sleeve.parent = gun_root
    muzzle = torus(
        "GUN_MuzzleCollar",
        (6.35, 0, 3.7),
        0.115,
        0.035,
        "05_ARMAMENT",
        "dark_metal",
        rotation=(0, math.pi * 0.5, 0),
        description="Muzzle reference collar; no fictional recoil compensator.",
    )
    muzzle.parent = gun_root
    bore = cylinder_between(
        "GUN_Bore",
        (6.345, 0, 3.7),
        (6.37, 0, 3.7),
        0.06,
        "05_ARMAMENT",
        "nozzle",
        vertices=16,
        description="120 mm bore opening.",
    )
    bore.parent = gun_root

    for y in (-0.34, 0.34):
        recoil = cylinder_between(
            f"GUN_RecoilCylinder_{y:+.2f}",
            (-1.25, y, 3.28),
            (0.42, y, 3.28),
            0.105,
            "05_ARMAMENT",
            "hydraulic",
            vertices=12,
            description="Hydropneumatic recoil cylinder; nominal recoil travel 0.48 m.",
        )
        recoil.parent = gun_root
        rail = cylinder_between(
            f"GUN_RecoilRail_{y:+.2f}",
            (-1.65, y, 3.52),
            (0.55, y, 3.52),
            0.055,
            "05_ARMAMENT",
            "gunmetal",
            vertices=8,
            description="Linear recoil guide reacting into the open yoke.",
        )
        rail.parent = gun_root

    # A low-mounted rotating carrier keeps ready ammunition close to the yaw
    # axis. It is inspired by naval automatic-gun carrier drums, but scaled for
    # complete 120 mm rounds and paired with a vertical transfer shuttle.
    for x in (-3.08, -1.76):
        track = torus(
            f"GUN_MagazineCarrierTrack_{x:+.2f}",
            (x, 0, 2.12),
            0.84,
            0.055,
            "05_ARMAMENT",
            "gunmetal",
            rotation=(0, math.pi * 0.5, 0),
            description="Rotary ready-magazine carrier track supporting individual cartridge cradles.",
        )
        track.parent = gun_root
    magazine_axle = cylinder_between(
        "GUN_MagazineAxle",
        (-3.18, 0, 2.12),
        (-1.66, 0, 2.12),
        0.105,
        "05_ARMAMENT",
        "dark_metal",
        vertices=12,
        description="Central rotary-magazine axle and torque tube.",
    )
    magazine_axle.parent = gun_root
    for i in range(14):
        angle = math.radians(90.0 + i * (360.0 / 14.0))
        y = 0.72 * math.cos(angle)
        z = 2.12 + 0.72 * math.sin(angle)
        # Leave the indexed top cradle empty and show that same round on the
        # loading tray. This keeps the ready inventory at fourteen while making
        # the carrier -> elevator -> tray -> chamber sequence visually explicit.
        if i == 0:
            case_start, case_end = (-2.98, 0, 3.70), (-2.05, 0, 3.70)
            projectile_end = (-1.78, 0, 3.70)
            state = "loading_tray"
            name = "GUN_StagedRound"
        else:
            case_start, case_end = (-3.04, y, z), (-2.05, y, z)
            projectile_end = (-1.80, y, z)
            state = "ready_carrier"
            name = f"GUN_ReadyRound_{i + 1:02d}"
        case = cylinder_between(
            f"{name}_Case",
            case_start,
            case_end,
            0.06,
            "05_ARMAMENT",
            "ammunition",
            vertices=12,
            description="120 mm complete-round cartridge case; its consumable mass is removed when the round is fired.",
        )
        case["feed_state"] = state
        case.parent = gun_root
        projectile = cone_between(
            f"{name}_Projectile",
            case_end,
            projectile_end,
            0.055,
            0.022,
            "05_ARMAMENT",
            "barrel",
            vertices=12,
            description="Tapered projectile/neck section of a stored 120 mm complete round.",
        )
        projectile["feed_state"] = state
        projectile.parent = gun_root
    for y in (-0.91, 0.91):
        drive = box(
            f"GUN_MagazineChainDrive_{y:+.2f}",
            (-2.42, y, 2.12),
            (0.42, 0.18, 0.42),
            "05_ARMAMENT",
            "gunmetal",
            description="Redundant electric carrier-chain drive and indexing gearbox.",
            bevel_width=0.055,
        )
        drive.parent = gun_root
        suspension = cylinder_between(
            f"GUN_MagazineSuspension_{y:+.2f}",
            (-1.78, y * 0.88, 2.12),
            (-0.58, y * 1.12, 2.50),
            0.060,
            "05_ARMAMENT",
            "gunmetal",
            vertices=8,
            description="Magazine suspension member carrying cartridge mass and feed reaction into the slew bearing.",
        )
        suspension.parent = gun_root
    blast_floor = box(
        "GUN_MagazineBlastFloor",
        (-2.42, 0, 1.23),
        (1.55, 1.86, 0.08),
        "05_ARMAMENT",
        "light_metal",
        description="Thin vented blast and fragment plate beneath the ready magazine.",
        bevel_width=0.018,
    )
    blast_floor.parent = gun_root

    transfer_shuttle = box(
        "GUN_AmmunitionTransferShuttle",
        (-2.42, 0, 3.08),
        (1.32, 0.34, 0.18),
        "05_ARMAMENT",
        "gunmetal",
        description="Powered cartridge cradle accepting the indexed top round and lifting it to the bore line.",
        bevel_width=0.025,
    )
    transfer_shuttle.parent = gun_root
    for x in (-3.04, -1.80):
        for y in (-0.22, 0.22):
            rail = cylinder_between(
                f"GUN_TransferGuide_{x:+.2f}_{y:+.2f}",
                (x, y, 2.72),
                (x, y, 3.58),
                0.032,
                "05_ARMAMENT",
                "gunmetal",
                vertices=8,
                description="Vertical guide rail constraining the ammunition transfer shuttle.",
            )
            rail.parent = gun_root
    for y in (-0.31, 0.31):
        lift_actuator = cylinder_between(
            f"GUN_TransferLiftActuator_{y:+.2f}",
            (-2.42, y, 2.47),
            (-2.42, y, 3.42),
            0.055,
            "05_ARMAMENT",
            "hydraulic",
            vertices=10,
            description="Paired electromechanical lift actuator driving the cartridge shuttle between carrier and bore line.",
        )
        lift_actuator.parent = gun_root
    loading_tray = box(
        "GUN_LoadingTray",
        (-2.30, 0, 3.54),
        (1.45, 0.40, 0.10),
        "05_ARMAMENT",
        "gunmetal",
        description="Bore-aligned loading tray receiving a round from the transfer shuttle.",
        bevel_width=0.018,
    )
    loading_tray.parent = gun_root
    rammer = cylinder_between(
        "GUN_PoweredRammerCylinder",
        (-3.78, 0, 3.70),
        (-2.58, 0, 3.70),
        0.072,
        "05_ARMAMENT",
        "hydraulic",
        vertices=12,
        description="Powered axial rammer that seats a complete round in the chamber after the wedge opens.",
    )
    rammer.parent = gun_root
    ram_head = box(
        "GUN_RammerHead",
        (-2.50, 0, 3.70),
        (0.14, 0.25, 0.25),
        "05_ARMAMENT",
        "dark_metal",
        description="Rammer head contacting the cartridge base during chambering.",
        bevel_width=0.018,
    )
    ram_head.parent = gun_root

    for y in (-1.47, 1.47):
        yaw_drive = box(
            f"GUN_YawDrive_{y:+.2f}",
            (0.18, y, 2.58),
            (0.62, 0.32, 0.34),
            "05_ARMAMENT",
            "gunmetal",
            description="Redundant electric yaw drive and reduction gearbox.",
            bevel_width=0.06,
        )
        yaw_drive.parent = gun_root
        elevation_drive = cylinder_between(
            f"GUN_ElevationDrive_{y:+.2f}",
            (-0.4, y * 0.72, 3.05),
            (-0.18, y * 0.72, 3.62),
            0.09,
            "05_ARMAMENT",
            "hydraulic",
            vertices=12,
            description="Redundant elevation actuator with mechanical hold brake.",
        )
        elevation_drive.parent = gun_root

    for y in (-1.15, 1.15):
        cylinder_between(
            f"GUN_RingLoadPath_{y:+.2f}",
            (0, y, 2.45),
            (0, y * 1.65, 1.55),
            0.095,
            "01_STRUCTURE",
            "truss",
            description="Gun reaction load path into dorsal truss longerons.",
        )


def build_utilities():
    for side in (-1, 1):
        fuel_y = side * 1.82
        pipe_curve(
            f"UTIL_FuelFeed_{'P' if side > 0 else 'S'}",
            [
                (7.0, fuel_y * 0.9, -1.65),
                (5.8, fuel_y, -1.85),
                (2.0, fuel_y, -1.85),
                (-2.0, fuel_y, -1.85),
                (-6.0, fuel_y, -1.72),
                (-10.5, fuel_y * 0.72, -1.25),
                (-13.5, side * 0.72, -0.7),
                (-14.8, side * 0.6, -0.48),
            ],
            0.06,
            "06_UTILITIES",
            "fuel_line",
            description="120 mm OD redundant vacuum-jacketed cryogenic feed trunk.",
        )
        power_y = side * 2.05
        pipe_curve(
            f"UTIL_HVDCBus_{'P' if side > 0 else 'S'}",
            [
                (14.2, power_y * 0.7, 0.75),
                (11.8, power_y, 1.25),
                (8.0, power_y, 1.25),
                (4.0, power_y, 1.25),
                (0.0, power_y, 1.18),
                (-4.0, power_y, 1.15),
                (-8.0, power_y * 0.92, 1.05),
                (-12.8, side * 1.2, 0.62),
                (-14.5, side * 0.8, 0.42),
            ],
            0.0325,
            "06_UTILITIES",
            "power",
            description="Separated 65 mm OD shielded HVDC bus; conductor gauge remains a simulation-derived parameter.",
        )

    for side in (-1, 1):
        side_code = "P" if side > 0 else "S"
        for role, z in (("Supply", 0.08), ("Return", -0.08)):
            pipe_curve(
                f"UTIL_HighTempRadiator{role}_{side_code}",
                [
                    (6.2, side * 1.55, z),
                    (2.0, side * 1.82, z),
                    (-2.0, side * 1.92, z),
                    (-5.2, side * 2.05, z),
                    (-7.4, side * 2.54, z),
                ],
                0.027,
                "06_UTILITIES",
                "coolant",
                description=f"54 mm OD high-temperature power-system radiator {role.lower()} line feeding the {side_code} wing root manifold.",
            )
    for x in (-9.8, -5.8, -1.8, 2.2, 6.2):
        for side in (-1, 1):
            box(
                f"UTIL_LineClamp_{x:+.1f}_{side:+d}",
                (x, side * 1.82, -1.85),
                (0.12, 0.28, 0.18),
                "06_UTILITIES",
                "dark_metal",
                description="Service-line stand-off and micrometeoroid isolation clamp.",
                bevel_width=0.02,
            )


def build_polish_details():
    # Removable avionics and power-conversion boxes are kept inside the truss
    # shadow and mounted close to primary nodes to avoid bending long members.
    equipment = [
        (-8.0, 0.85, 0.75, "DriveControl"),
        (-5.9, -0.9, 0.65, "PowerConversion"),
        (2.0, 0.9, -0.55, "FireControl"),
        (5.7, -0.85, 0.55, "TankController"),
    ]
    for x, y, z, label in equipment:
        enclosure = box(
            f"EQUIP_{label}",
            (x, y, z),
            (1.15, 0.62, 0.48),
            "06_UTILITIES",
            "dark_metal",
            description=f"Replaceable {label} electronics enclosure with conductive/radiative mounting feet.",
            bevel_width=0.065,
        )
        enclosure["functional_role"] = label
        tray = box(
            f"EQUIP_{label}_Tray",
            (x, y, z - 0.29),
            (1.30, 0.78, 0.08),
            "06_UTILITIES",
            "light_metal",
            description="Bolted equipment tray touching the enclosure and distributing its 1 g inertial load into four radial struts.",
            bevel_width=0.018,
        )
        radial = Vector((0.0, y, z - 0.34))
        radial_length = math.sqrt(radial.y * radial.y + radial.z * radial.z)
        if radial_length < 0.1:
            radial = Vector((0.0, 1.0, 0.0))
            radial_length = 1.0
        target_y = radial.y / radial_length * 2.28
        target_z = radial.z / radial_length * 2.28
        for dx in (-0.46, 0.46):
            for lateral in (-1, 1):
                tray_y = y + lateral * 0.28
                tray_z = z - 0.31
                target_offset = lateral * 0.10
                target_tangent_y = target_y - target_offset * target_z / 2.28
                target_tangent_z = target_z + target_offset * target_y / 2.28
                cylinder_between(
                    f"EQUIP_{label}_Mount_{dx:+.2f}_{lateral:+d}",
                    (x + dx, tray_y, tray_z),
                    (x + dx, target_tangent_y, target_tangent_z),
                    0.032,
                    "06_UTILITIES",
                    "truss",
                    vertices=8,
                    description="Short triangulated mounting foot terminating on the adjacent primary truss longeron.",
                )
        for dx in (-0.46, 0.46):
            cylinder_between(
                f"EQUIP_{label}_RetentionBolt_{dx:+.2f}",
                (x + dx, y, z - 0.34),
                (x + dx, y, z - 0.22),
                0.022,
                "06_UTILITIES",
                "dark_metal",
                vertices=8,
                description="Captive enclosure-to-tray retention bolt.",
            )

    # Tank aft support ring and eight short struts leave a visible thermal gap
    # while reacting the tank's 1 g axial load into the forward truss bay.
    torus(
        "FUEL_AftSupportRing",
        (7.72, 0, 0),
        2.34,
        0.095,
        "04_FUEL",
        "dark_metal",
        rotation=(0, math.pi * 0.5, 0),
        description="Tank aft support ring with low-conductivity stand-offs.",
    )
    for i in range(8):
        angle = math.radians(22.5 + i * 45.0)
        y0 = 2.35 * math.cos(angle)
        z0 = 2.35 * math.sin(angle)
        cylinder_between(
            f"FUEL_AftSupportStrut_{i:02d}",
            (8.0, y0, z0),
            (7.72, y0 * 0.96, z0 * 0.96),
            0.055,
            "04_FUEL",
            "truss",
            vertices=8,
            description="Axial tank support and thermal-isolation strut.",
        )

    # A clearly visible service umbilical keeps the habitat independent from
    # the tank structure across their narrow forward interface gap.
    for side in (-1, 1):
        pipe_curve(
            f"HAB_ServiceUmbilical_{side:+d}",
            [
                (14.12, side * 1.15, -1.28),
                (13.78, side * 1.35, -1.42),
                (13.42, side * 1.48, -1.55),
            ],
            0.026,
            "06_UTILITIES",
            "coolant",
            description="Flexible habitat service umbilical across the pressure-vessel/tank structural gap.",
        )


def build_sensors_and_rcs():
    sensor_root = create_empty(
        "SENSORS_ForwardCluster",
        "07_SENSORS",
        (20.2, 0, 0),
        description="Forward passive optical/IR and active radar apertures.",
    )
    for angle_deg in (45, 135, 225, 315):
        angle = math.radians(angle_deg)
        y = 1.62 * math.cos(angle)
        z = 1.62 * math.sin(angle)
        panel = box(
            f"SENSORS_AESA_{angle_deg:03d}",
            (19.55, y, z),
            (0.08, 0.72, 0.72),
            "07_SENSORS",
            "sensor",
            description="Conformal active electronically scanned array panel; performance derived in simulation.",
            bevel_width=0.035,
        )
        panel.rotation_euler.x = angle
        panel.parent = sensor_root

    for z in (-0.52, 0.0, 0.52):
        sphere(
            f"SENSORS_OpticalHead_{z:+.2f}",
            (21.05, 0, z),
            0.16,
            "07_SENSORS",
            "glass",
            segments=12,
            rings=6,
            scale=(0.4, 1, 1),
            description="Protected optical/IR telescope aperture.",
        )
    aft_bench = box(
        "SENSORS_AftOpticalBench",
        (-11.45, 0, 2.26),
        (0.52, 1.62, 0.12),
        "07_SENSORS",
        "light_metal",
        description="Low-profile aft sensor bench spanning two dorsal truss longerons; replaces the unsupported mast and spherical head.",
        bevel_width=0.025,
    )
    aft_module = box(
        "SENSORS_AftOpticalModule",
        (-11.67, 0, 2.39),
        (0.34, 0.44, 0.22),
        "07_SENSORS",
        "sensor",
        description="Compact aft-looking optical/IR tracker enclosure bolted directly to its structural bench.",
        bevel_width=0.04,
    )
    cylinder_between(
        "SENSORS_AftOpticalWindow",
        (-11.88, 0, 2.39),
        (-11.77, 0, 2.39),
        0.095,
        "07_SENSORS",
        "glass",
        vertices=12,
        description="Recessed aft-facing optical/IR aperture.",
    )
    for y_sign in (-1, 1):
        cylinder_between(
            f"SENSORS_AftBenchMount_{y_sign:+d}",
            (-11.45, y_sign * 0.68, 2.20),
            (-11.45, y_sign * 0.90, 2.17),
            0.035,
            "07_SENSORS",
            "truss",
            vertices=8,
            description="Short sensor-bench foot terminating on a dorsal primary longeron.",
        )

    cluster_specs = (
        ("Aft", -10.50, -8.80),
        ("Forward", 13.50, 6.35),
    )
    for cluster_name, pod_x, tank_x in cluster_specs:
        collar = annulus(
            f"RCS_{cluster_name}StructuralCollar",
            (pod_x, 0, 0),
            2.12,
            2.30,
            0.14,
            "08_RCS",
            "truss",
            axis="X",
            description="Local RCS collar distributing pod thrust into the primary structure at four quadrants.",
        )
        collar["nominal_thruster_class_N"] = 100

        tank_locs = {}
        for propellant_index, (propellant, y) in enumerate(
            (("Fuel", -0.42), ("Oxidizer", 0.42))
        ):
            tank = sphere(
                f"RCS_{cluster_name}{propellant}Tank",
                (tank_x, y, 0.28),
                0.31,
                "08_RCS",
                "tank",
                segments=14,
                rings=7,
                scale=(1.16, 1.0, 1.0),
                description=f"Small diaphragm {propellant.lower()} tank for the {cluster_name.lower()} 100 N-class bipropellant RCS cluster.",
            )
            tank["propellant_role"] = propellant.lower()
            tank["consumable_mass_required"] = True
            tank_locs[propellant] = Vector((tank_x, y, 0.28))

        box(
            f"RCS_{cluster_name}TankTray",
            (tank_x, 0, -0.08),
            (1.02, 1.18, 0.10),
            "08_RCS",
            "light_metal",
            description="Bolted paired-tank tray carrying acceleration loads into the central spine.",
            bevel_width=0.025,
        )
        for y_sign in (-1, 1):
            cylinder_between(
                f"RCS_{cluster_name}TankStay_{y_sign:+d}",
                (tank_x, y_sign * 0.48, -0.10),
                (tank_x, y_sign * 1.66, -1.55),
                0.035,
                "08_RCS",
                "truss",
                vertices=8,
                description="Triangulated RCS tank-tray stay terminating on a lower primary longeron.",
            )

        # Two separately routed lines avoid a common-mode leak and make the
        # propellant source visible. The forward pair routes around the outside
        # of the large cryogenic tank rather than through it.
        for propellant_index, propellant in enumerate(("Fuel", "Oxidizer")):
            offset = -1 if propellant == "Fuel" else 1
            start = tank_locs[propellant]
            if cluster_name == "Forward":
                points = [
                    tuple(start),
                    (7.05, offset * 1.35, 1.55),
                    (8.05, offset * 0.12, offset * 2.43),
                    (12.70, offset * 0.12, offset * 2.43),
                    (13.25, offset * 0.12, offset * 2.10),
                    (pod_x, offset * 0.12, 0.0),
                ]
            else:
                points = [
                    tuple(start),
                    (-9.25, offset * 0.70, 0.48),
                    (-10.15, offset * 0.35, 0.38),
                    (pod_x, offset * 0.12, 0.0),
                ]
            pipe_curve(
                f"RCS_{cluster_name}{propellant}Main",
                points,
                0.014,
                "08_RCS",
                "fuel_line",
                description=f"Isolated {propellant.lower()} main from the paired tank module to the {cluster_name.lower()} RCS manifold.",
            )

        for y_sign, z_sign in ((1, 1), (-1, 1), (1, -1), (-1, -1)):
            loc = Vector((pod_x, y_sign * 1.90, z_sign * 1.40))
            pod = box(
                f"RCS_Pod_{pod_x:+.1f}_{y_sign:+d}_{z_sign:+d}",
                loc,
                (0.36, 0.24, 0.24),
                "08_RCS",
                "dark_metal",
                description="Compact two-axis 100 N-class bipropellant RCS pod with a visible structural foot and dual feed lines.",
                bevel_width=0.04,
            )
            pod["nominal_thruster_class_N"] = 100
            radial = Vector((0, loc.y, loc.z)).normalized()
            for x_offset in (-0.11, 0.11):
                cylinder_between(
                    f"RCS_PodMount_{pod_x:+.1f}_{y_sign:+d}_{z_sign:+d}_{x_offset:+.2f}",
                    (pod_x + x_offset, radial.y * 2.27, radial.z * 2.27),
                    (
                        pod_x + x_offset,
                        loc.y - radial.y * 0.09,
                        loc.z - radial.z * 0.09,
                    ),
                    0.032,
                    "08_RCS",
                    "truss",
                    vertices=8,
                    description="Twin pod mounting feet transmitting RCS thrust into the local structural collar.",
                )
            for propellant_index, propellant in enumerate(("Fuel", "Oxidizer")):
                offset = -0.045 if propellant == "Fuel" else 0.045
                pipe_curve(
                    f"RCS_{propellant}Branch_{pod_x:+.1f}_{y_sign:+d}_{z_sign:+d}",
                    [
                        (pod_x, offset, 0.0),
                        (
                            pod_x,
                            y_sign * 0.95 + offset,
                            z_sign * 0.70,
                        ),
                        (
                            pod_x,
                            loc.y - radial.y * 0.13 + offset,
                            loc.z - radial.z * 0.13,
                        ),
                    ],
                    0.009,
                    "08_RCS",
                    "fuel_line",
                    description=f"{propellant} branch from the collar manifold to the pod isolation valve.",
                )
            for axis, direction in (
                ("Y", Vector((0, y_sign, 0))),
                ("Z", Vector((0, 0, z_sign))),
            ):
                throat = loc + direction * 0.11
                lip = loc + direction * 0.20
                cone_between(
                    f"RCS_Nozzle_{pod_x:+.1f}_{y_sign:+d}_{z_sign:+d}_{axis}",
                    throat,
                    lip,
                    0.022,
                    0.052,
                    "08_RCS",
                    "nozzle",
                    vertices=10,
                    description="Scaled 100 N-class attitude-control nozzle with a 52 mm exit cup.",
                    capped=False,
                )


def convert_curves_to_mesh():
    for obj in list(bpy.data.objects):
        if obj.type == "CURVE":
            bpy.context.view_layer.objects.active = obj
            obj.select_set(True)
            bpy.ops.object.convert(target="MESH")
            obj.select_set(False)


def unwrap_all_meshes():
    mesh_count = 0
    for obj in bpy.data.objects:
        if obj.type != "MESH" or len(obj.data.polygons) == 0:
            continue
        bpy.ops.object.select_all(action="DESELECT")
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj
        try:
            bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
            bpy.ops.object.mode_set(mode="EDIT")
            bpy.ops.mesh.select_all(action="SELECT")
            bpy.ops.uv.smart_project(angle_limit=math.radians(66.0), island_margin=0.025)
            bpy.ops.object.mode_set(mode="OBJECT")
        except Exception:
            if bpy.context.object and bpy.context.object.mode != "OBJECT":
                bpy.ops.object.mode_set(mode="OBJECT")
            if not obj.data.uv_layers:
                obj.data.uv_layers.new(name="UVMap")
        if obj.data.uv_layers:
            obj.data.uv_layers.active.name = "UVMap"
        mesh_count += 1
    return mesh_count


def material_principled(name, base_color, metallic, roughness, emission=None):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    bsdf = nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*base_color, 1.0)
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness
    if emission is not None:
        if "Emission Color" in bsdf.inputs:
            bsdf.inputs["Emission Color"].default_value = (*emission[0], 1.0)
            bsdf.inputs["Emission Strength"].default_value = emission[1]
        elif "Emission" in bsdf.inputs:
            bsdf.inputs["Emission"].default_value = (*emission[0], 1.0)
    return mat


def material_textured(name, albedo_path, roughness_path, metallic=0.0, roughness_default=0.5):
    mat = material_principled(name, (0.5, 0.5, 0.5), metallic, roughness_default)
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    bsdf = nodes.get("Principled BSDF")
    tex = nodes.new("ShaderNodeTexImage")
    tex.name = "Albedo"
    tex.image = bpy.data.images.load(albedo_path, check_existing=True)
    tex.interpolation = "Linear"
    links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    if roughness_path:
        rough = nodes.new("ShaderNodeTexImage")
        rough.name = "Roughness"
        rough.image = bpy.data.images.load(roughness_path, check_existing=True)
        rough.image.colorspace_settings.name = "Non-Color"
        links.new(rough.outputs["Color"], bsdf.inputs["Roughness"])
    return mat


def build_materials():
    materials = {
        "hull": material_textured(
            "MAT_HullPanelled",
            os.path.join(TEXTURE_DIR, "hull_albedo.png"),
            os.path.join(TEXTURE_DIR, "hull_roughness.png"),
            metallic=0.72,
        ),
        "truss": material_principled("MAT_TrussCarbon", (0.055, 0.065, 0.072), 0.68, 0.34),
        "light_metal": material_principled("MAT_LightMetal", (0.34, 0.38, 0.42), 0.78, 0.29),
        "dark_metal": material_principled("MAT_DarkMetal", (0.035, 0.04, 0.046), 0.82, 0.24),
        "seal": material_principled("MAT_DockingSeal", (0.018, 0.022, 0.024), 0.02, 0.72),
        "tank": material_principled("MAT_TankMLI", (0.68, 0.70, 0.66), 0.58, 0.46),
        "shield": material_principled("MAT_ShadowShield", (0.22, 0.18, 0.13), 0.46, 0.65),
        "engine": material_principled("MAT_EngineCasing", (0.12, 0.14, 0.16), 0.84, 0.26),
        "nozzle": material_principled(
            "MAT_NozzleInterior",
            (0.008, 0.009, 0.011),
            0.74,
            0.42,
            ((0.035, 0.006, 0.002), 0.12),
        ),
        "copper": material_principled("MAT_Copper", (0.38, 0.12, 0.045), 0.9, 0.28),
        "gunmetal": material_principled("MAT_GunMechanism", (0.11, 0.12, 0.13), 0.87, 0.25),
        "barrel": material_principled("MAT_Barrel", (0.055, 0.06, 0.065), 0.9, 0.19),
        "hydraulic": material_principled("MAT_Hydraulic", (0.16, 0.18, 0.19), 0.75, 0.17),
        "ammunition": material_principled("MAT_Ammunition", (0.42, 0.28, 0.07), 0.72, 0.27),
        "fuel_line": material_principled("MAT_FuelLine", (0.66, 0.72, 0.78), 0.58, 0.33),
        "power": material_principled("MAT_PowerBus", (0.72, 0.22, 0.035), 0.48, 0.39),
        "coolant": material_principled("MAT_CoolantLine", (0.025, 0.28, 0.32), 0.55, 0.32),
        "glass": material_principled("MAT_OpticalGlass", (0.015, 0.10, 0.16), 0.42, 0.10, ((0.01, 0.08, 0.12), 0.5)),
        "sensor": material_principled("MAT_SensorPanel", (0.025, 0.075, 0.095), 0.63, 0.22),
        "water": material_principled("MAT_WaterStore", (0.12, 0.28, 0.33), 0.25, 0.44),
        "radiator": material_textured(
            "MAT_Radiator",
            os.path.join(TEXTURE_DIR, "radiator_albedo.png"),
            None,
            metallic=0.55,
            roughness_default=0.42,
        ),
    }
    for obj in bpy.data.objects:
        if obj.type != "MESH":
            continue
        key = obj.get("material_key", "hull")
        mat = materials.get(key, materials["hull"])
        obj.data.materials.clear()
        obj.data.materials.append(mat)
    return materials


def add_markings():
    hazard = material_principled("MAT_RadiationWarning", (0.88, 0.48, 0.015), 0.2, 0.34, ((0.18, 0.07, 0.0), 0.3))
    white = material_principled("MAT_StencilWhite", (0.72, 0.74, 0.72), 0.2, 0.56)
    black = bpy.data.materials["MAT_NozzleInterior"]

    for side in (-1, 1):
        for z in (-0.62, 0.0, 0.62):
            stripe = box(
                f"MARKING_RadiationStripe_{side:+d}_{z:+.2f}",
                (-13.78, side * 1.2, z),
                (0.34, 0.025, 0.22),
                "09_MARKINGS",
                "hull",
                description="High-radiation service-zone warning marker.",
                bevel_width=0.01,
            )
            stripe.data.materials.clear()
            stripe.data.materials.append(hazard if int((z + 0.62) * 10) % 2 == 0 else black)

    for x in (15.2, 17.2, 19.2):
        plate = box(
            f"MARKING_HabitatBand_{x:.1f}",
            (x, -2.095, 0),
            (0.12, 0.018, 1.05),
            "09_MARKINGS",
            "hull",
            description="High-visibility pressure-vessel alignment stencil.",
            bevel_width=0.004,
        )
        plate.data.materials.clear()
        plate.data.materials.append(white)


def camera_look_at(camera, target):
    direction = Vector(target) - camera.location
    camera.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()


def setup_presentation():
    scene = bpy.context.scene
    scene.unit_settings.system = "METRIC"
    scene.unit_settings.scale_length = 1.0
    # Blender 5.2 exposes the current Eevee engine under this enum.
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = 960
    scene.render.resolution_y = 540
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.film_transparent = False
    scene.render.image_settings.color_mode = "RGBA"
    for candidate_look in ("AgX - Medium Low Contrast", "AgX - Medium High Contrast"):
        try:
            scene.view_settings.look = candidate_look
            break
        except TypeError:
            continue

    world = bpy.data.worlds.get("World") or bpy.data.worlds.new("World")
    scene.world = world
    world.use_nodes = True
    bg = world.node_tree.nodes.get("Background")
    bg.inputs["Color"].default_value = (0.002, 0.004, 0.008, 1.0)
    bg.inputs["Strength"].default_value = 0.28

    camera_data = bpy.data.cameras.new("CAM_Engineering")
    camera = bpy.data.objects.new("CAM_Engineering", camera_data)
    COLLECTIONS["10_PRESENTATION"].objects.link(camera)
    camera_data.lens = 62
    camera_data.sensor_width = 36
    camera.location = (46, -62, 32)
    camera_look_at(camera, (0.5, 0, 0))
    scene.camera = camera

    def area_light(name, loc, energy, size, color):
        data = bpy.data.lights.new(name, "AREA")
        data.energy = energy
        data.shape = "DISK"
        data.size = size
        data.color = color
        obj = bpy.data.objects.new(name, data)
        obj.location = loc
        camera_look_at(obj, (0, 0, 0))
        COLLECTIONS["10_PRESENTATION"].objects.link(obj)
        return obj

    area_light("LIGHT_Key", (8, -18, 24), 6200, 12, (0.82, 0.90, 1.0))
    area_light("LIGHT_Fill", (10, 20, 9), 4300, 14, (1.0, 0.58, 0.36))
    area_light("LIGHT_Rim", (-25, -8, 15), 5600, 11, (0.38, 0.60, 1.0))

    return camera


def render_views(camera):
    scene = bpy.context.scene
    views = [
        ("asterion_mk1_hero.png", (46, -62, 32), (0.0, 0, 0)),
        ("asterion_mk1_side.png", (2, -74, 8), (0.0, 0, 0)),
        ("asterion_mk1_top.png", (0, -2, 74), (0, 0, 0)),
        ("asterion_mk1_gun_detail.png", (13, -18, 12), (0.0, 0, 2.5)),
        ("asterion_mk1_gun_feed_detail.png", (-12, -15, 9), (-1.7, 0, 3.2)),
        ("asterion_mk1_aft_nozzle.png", (-42, -16, 9), (-16.8, 0, 0)),
        ("asterion_mk1_docking_detail.png", (31, -13, 7), (20.9, 0, 0)),
        ("asterion_mk1_rcs_radiator_detail.png", (-1, -24, 10), (-5.4, 0, 0)),
        ("asterion_mk1_aft_rcs_detail.png", (-4, -10, -7), (-10.5, -1.8, -1.3)),
        ("asterion_mk1_forward_rcs_detail.png", (13.5, -11, -5), (13.5, -1.8, -1.3)),
    ]
    for filename, loc, target in views:
        camera.location = loc
        camera_look_at(camera, target)
        scene.render.filepath = os.path.join(RENDER_DIR, filename)
        bpy.ops.render.render(write_still=True)


def export_glb():
    bpy.ops.object.select_all(action="DESELECT")
    export_objects = [
        obj
        for obj in bpy.data.objects
        if not obj.get("exclude_from_export", False)
        and obj.type in {"MESH", "EMPTY"}
        and not obj.name.startswith("LIGHT_")
        and not obj.name.startswith("CAM_")
    ]
    for obj in export_objects:
        obj.select_set(True)
    bpy.ops.export_scene.gltf(
        filepath=GLB_PATH,
        export_format="GLB",
        use_selection=True,
        export_extras=True,
        export_apply=True,
    )
    bpy.ops.object.select_all(action="DESELECT")


def write_stats():
    totals = {
        "asset": "Asterion Mk I",
        "mesh_objects": 0,
        "vertices": 0,
        "polygons": 0,
        "triangles": 0,
        "mesh_objects_with_uv": 0,
        "mesh_objects_without_uv": [],
        "collections": {},
    }
    for coll in bpy.data.collections:
        totals["collections"][coll.name] = len(coll.objects)
    for obj in bpy.data.objects:
        if obj.type != "MESH":
            continue
        totals["mesh_objects"] += 1
        totals["vertices"] += len(obj.data.vertices)
        totals["polygons"] += len(obj.data.polygons)
        obj.data.calc_loop_triangles()
        totals["triangles"] += len(obj.data.loop_triangles)
        if obj.data.uv_layers:
            totals["mesh_objects_with_uv"] += 1
        else:
            totals["mesh_objects_without_uv"].append(obj.name)
    totals["uv_audit_pass"] = len(totals["mesh_objects_without_uv"]) == 0
    with open(STATS_PATH, "w", encoding="utf-8") as handle:
        json.dump(totals, handle, indent=2)
    return totals


def main():
    clear_scene()
    for name in (
        "00_MASTER",
        "01_STRUCTURE",
        "02_PROPULSION",
        "03_HABITAT",
        "04_FUEL",
        "05_ARMAMENT",
        "06_UTILITIES",
        "07_SENSORS",
        "08_RCS",
        "09_MARKINGS",
        "10_PRESENTATION",
    ):
        COLLECTIONS[name] = collection(name)

    make_texture_maps()
    build_master_reference()
    build_truss()
    build_habitat()
    build_propellant()
    build_shadow_shield_and_engine()
    build_gun()
    build_utilities()
    build_polish_details()
    build_sensors_and_rcs()
    convert_curves_to_mesh()

    # UV creation deliberately precedes material assignment.
    unwrap_all_meshes()
    build_materials()
    add_markings()
    # Marking meshes are created after the main pass, so unwrap them separately.
    unwrap_all_meshes()

    camera = setup_presentation()
    totals = write_stats()
    bpy.context.scene["asset_statistics"] = json.dumps(totals)
    bpy.context.scene["engineering_note"] = (
        "One speculative subsystem: compact fusion drive. Ordinary subsystems require conservation-based simulation."
    )

    bpy.ops.wm.save_as_mainfile(filepath=BLEND_PATH, check_existing=False)
    render_views(camera)
    export_glb()
    bpy.ops.wm.save_as_mainfile(filepath=BLEND_PATH, check_existing=False)
    print("ASTERION_BUILD_COMPLETE")
    print(json.dumps(totals, indent=2))


if __name__ == "__main__":
    main()
