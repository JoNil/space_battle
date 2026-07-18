# Asterion Mk I

The Asterion Mk I is the initial engineering reference craft for Space Battle. It is a compact, two-person, long-duration combat spacecraft organized around a load-bearing open truss.

## Layout

| Station (X axis) | Subsystem | Rationale |
| --- | --- | --- |
| `+14.4 m` to `+21.9 m` | Crew pressure vessel and NDS-style docking nose | Maximum stand-off from the drive; approximately 58 m³ gross pressure volume for two people |
| `+7.2 m` to `+13.6 m` | Cryogenic fusion-propellant tank | Close to the crew to maximize drive separation; tank contents and surrounding water stores supplement the aft storm shelter |
| `-12.0 m` to `+8.0 m` | Octagonal aluminium/composite truss | Direct thrust and recoil load path with low panel mass |
| `X = 0 m`, dorsal | 120 mm L/55 gun, open slew bearing, and 14-round autoloader | Breech, recoil cradle, and ammunition carrier rotate together and react into the main truss |
| `-12.7 m` | Layered shadow shield | High-Z hot-side layer plus hydrogen-rich crew-side layer; protects only the forward shadow cone |
| `-20.5 m` to `-13.0 m` | Speculative fusion drive and magnetic nozzle | Radioactive machinery is kept at maximum practical stand-off |
| `-10.6 m` to `-4.2 m`, `Y = +/-2.7` to `+/-11.9 m` | Paired 500 K two-sided radiator wings | Radial XY planes make the vehicle edge-on to both emitting faces and provide 111.82 m² of active planform area |

The coordinate convention is `+X forward`, `+Y port`, and `+Z dorsal`. Dimensions are in metres.

## Gun

The model uses the dimensional envelope of a modern 120 mm L/55 smoothbore-class weapon, not a proprietary turret. The mount is deliberately skeletal: a flat rectangular-section slew bearing with captured bolts, two side yokes, trunnions, recoil rails, recuperator cylinders, a forged housing with a powered vertical sliding-wedge breech, barrel, and a rotating ready-ammunition carrier.

The carrier holds fourteen complete rounds close to the yaw axis. In the inspection state, thirteen are stored in the carrier and one is staged on the bore-aligned loading tray. Paired lift actuators raise an indexed round, and an axial rammer drives it into the chamber after the wedge opens. The nominal barrel length is 6.6 m and the modelled recoil travel is 0.48 m. No fictional recoil cancellation is assumed. Firing must change ship linear and angular momentum, and every fired complete round must reduce ship mass.

## Structure, docking, thermal control, and RCS

The truss bay interfaces use short node sleeves with alternating alignment cones and peripheral through-bolts, based on the cup/cone alignment and claw/bolt load path used by ISS truss attachment mechanisms. Equipment trays are tied to nearby primary longerons with triangulated feet instead of floating inside the truss.

The forward interface is an NDS-style docking assembly with an 800 mm clear pressure passage, pressure tunnel, hard-capture ring, replaceable pressure seal, six soft-capture links, three guide petals at 120 degrees, and twelve peripheral hard-capture hooks. The earlier unusable side hatch is omitted; crew transfer is through the axial docking tunnel.

The radiators are not vertical plates facing the vehicle. Each wing extends radially in the XY plane, so its two emitting-face normals point along `+Z` and `-Z`. The ship therefore presents only its narrow edge to the radiator surfaces, minimizing radiative exchange back into the truss. Both broad faces retain a nearly unobstructed deep-space view.

Each wing contains 24 replaceable channelized sandwich modules in a 4 by 6 grid. Bonded high-emissivity facesheets surround a structural honeycomb core. The module gaps carry radial and chordwise frame rails and permit differential thermal expansion. Four independently valved dual-pass circuits per wing connect separate root supply and return headers; a puncture can be isolated to one longitudinal strip. A continuous hinge beam, three four-bar support stations, and two locking actuators transfer deployed-wing loads directly into the primary truss longerons.

## 500 kW radiator calculation

The current reference design assumes a high-temperature power/drive cooling loop. It is not a direct 500 K crew-cabin coolant loop.

The two-sided Stefan-Boltzmann model is:

`Q = 2 * epsilon * eta * F * sigma * A * (T^4 - T_space^4)`

with:

- required heat rejection: `500,000 W`
- design margin: `20%`, giving `600,000 W`
- radiator surface temperature: `500 K` (`227 C`)
- infrared emissivity `epsilon = 0.88`
- panel/fin efficiency `eta = 0.90`
- unobstructed deep-space view factor `F = 0.98`
- `sigma = 5.670374419e-8 W m^-2 K^-4`
- deep-space sink temperature treated as negligible

This gives `2,750.70 W/m²` per effective emitting face and `5,501.40 W/m²` per square metre of two-sided planform. The nominal 500 kW area is `90.89 m²`; including the 20% design margin requires `109.06 m²`. The modeled active area is `111.82 m²`, giving approximately `615.17 kW` at 500 K.

Radiator sizing is extremely temperature-sensitive:

| Surface temperature | Area for 500 kW | Area with 20% margin |
| --- | ---: | ---: |
| `400 K` | `221.9 m²` | `266.3 m²` |
| `450 K` | `138.5 m²` | `166.2 m²` |
| `500 K` | `90.9 m²` | `109.1 m²` |
| `550 K` | `62.1 m²` | `74.5 m²` |
| `600 K` | `43.8 m²` | `52.6 m²` |

An `8 kg/m²` system-level areal-density allowance gives an initial radiator-system mass estimate of approximately `895 kg`, including panels, embedded tubes, coolant allowance, frames, headers, valves, and deployment hardware. This is deliberately more conservative than lightweight NASA prototype values and remains a placeholder until the subsystem mass model is derived.

The forward and aft RCS installations each use four compact two-axis 100 N-class pods. Every pod has two structural feet attached to a local load-distribution collar, two scaled nozzles, and separate fuel and oxidizer branches. Each cluster has paired diaphragm tanks on a triangulated tray; the forward mains route around the outside of the large cryogenic tank.

## Propellant and drive assumptions

The tank is placed forward, not next to the engine. This reduces radiation and heat exposure, limits the length of habitable plumbing, and keeps the crew within the shadow shield cone. It does require long, duplicated vacuum-jacketed feed lines running along the truss. The model includes two 120 mm outside-diameter feed trunks and two separated high-voltage DC cable runs.

The drive itself is a geometric placeholder for the one speculative technology allowed by the project. Fuel energy density, reaction mass, exhaust velocity, engine efficiency, radiator demand, and thrust schedule are intentionally left as simulation parameters rather than asserted by the mesh.

## Design references

- [NASA Docking System architecture](https://ntrs.nasa.gov/api/citations/20110011626/downloads/20110011626.pdf): deployable soft-capture ring, three guide petals at 120 degrees, capture before hard mate
- [ISS truss attachment mechanisms](https://ntrs.nasa.gov/api/citations/20110010964/downloads/20110010964.pdf?attachment=true): cup/cone alignment, ready-to-latch capture, and peripheral bolt load paths
- [U.S. Navy Mk 45 fact file](https://www.navy.mil/Resources/Fact-Files/Display-FactFiles/Article/2167864/mk-45-5-inch-5462-caliber-guns/mk-45-5-inch-5462-caliber-guns/): rotating ready-service ammunition storage and powered feed
- [NASA spacecraft thermal-control guidance](https://www.nasa.gov/smallsat-institute/sst-soa/thermal-control/): radiator view factor and deployable heat-rejection area
- [NASA multi-megawatt radiator design study](https://ntrs.nasa.gov/api/citations/20220019167/downloads/FINAL%20REV3%20-%20Considerations%20for%20Radiator%20Design%20in%20Multi-Megawatt%20Nuclear%20Electric%20Propulsion%20Applications.pdf): emissivity, fin efficiency, panel structure, parallel loops, view factor, and system areal density
- [NASA Apollo RCS design data](https://ntrs.nasa.gov/api/citations/19700078804/downloads/19700078804.pdf): distributed small-thrust pods supplied from dedicated propellant tanks

## Asset contents

- `asterion_mk1.blend` — native Blender source with named collections, origins, custom engineering properties, materials, UV maps, cameras, and lights
- `asterion_mk1.glb` — real-time export with material and custom-property data
- `asterion_mk1_spec.json` — machine-readable baseline dimensions and assumptions
- `asterion_mk1_stats.json` — generated mesh, triangle, and UV audit
- `textures/` — generated PBR albedo, roughness, and radiator maps
- `renders/` — verification renders
- `build_asterion_mk1.py` — reproducible Blender construction script

Every mesh object receives a UV map before materials are assigned. Round sections use deliberately modest segment counts and the truss uses eight-sided members to keep the real-time triangle count controlled.
