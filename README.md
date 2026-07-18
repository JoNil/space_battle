# Space Battle

Space Battle is a hard-science spacecraft combat simulator. Its purpose is to explore how battles in space might actually unfold when ships, weapons, sensors, thermal systems, structures, and crews are constrained by currently understood physics and present-day engineering.

The project allows one speculative technology: a compact fusion drive with enough effective fuel and propellant performance to sustain roughly one Earth gravity (`9.80665 m/s²`) for several days. That exception is not intended to remove engineering constraints. The simulation must still account for:

- fuel and propellant mass as they are consumed;
- the energy density required by a chosen mission profile;
- thrust, exhaust power, and the thrust schedule required to average about `1 g` over a full tank;
- the changing mass, centre of mass, acceleration, and rotational inertia of the ship;
- waste heat, radiators, shielding, and radiation stand-off distance;
- electrical generation, cable length, conductor material, wire gauge, voltage, current, resistance, heat, and conversion losses;
- structural loads caused by acceleration, manoeuvring, weapon recoil, and equipment placement;
- ammunition mass and the momentum and mass lost when a weapon fires;
- realistic radar, optical, infrared, and passive-emission detection;
- target tracking, light-time, projectile time of flight, manoeuvre uncertainty, and physically plausible firing envelopes;
- laser aperture, wavelength, beam quality, diffraction, pointing accuracy, dwell time, target material, reflective coatings, and thermal damage;
- plausible alternatives such as kinetic guns, missiles, nuclear effects, particle beams, and electromagnetic or plasma phenomena, with their real limitations represented.

Mass is a primary design variable. A lighter ship accelerates and rotates faster for a given force or torque, but reduced structure, shielding, radiator area, fuel, ammunition, or redundancy must create corresponding consequences. The simulation should conserve mass, momentum, angular momentum, and energy wherever applicable and should expose assumptions rather than hiding them behind arbitrary balance numbers.

