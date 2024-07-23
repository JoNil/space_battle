use bevy::prelude::World;
use bevy_editor_pls::{
    editor_window::{EditorWindow, EditorWindowContext},
    egui,
};
use bevy_rapier3d::prelude::RapierContext;

pub struct PhysicsProfilingPanel;

impl EditorWindow for PhysicsProfilingPanel {
    type State = ();
    const NAME: &'static str = "Physics Profiling";

    fn ui(world: &mut World, _cx: EditorWindowContext, ui: &mut egui::Ui) {
        let context = world.resource::<RapierContext>();
        let counters = &context.pipeline.counters;

        ui.label(format!("Total: {:.2}ms", counters.step_time()));
        ui.label(format!(
            "Collision detection: {:.2}ms",
            counters.collision_detection_time()
        ));
        ui.indent("Collision detection", |ui| {
            ui.label(format!("Broad-phase: {:.2}ms", counters.broad_phase_time()));
            ui.label(format!(
                "Narrow-phase: {:.2}ms",
                counters.narrow_phase_time()
            ));
        });
        ui.label(format!(
            "Island computation: {:.2}ms",
            counters.island_construction_time()
        ));
        ui.label(format!("Solver: {:.2}ms", counters.solver_time()));
        ui.indent("Solver", |ui| {
            let solver = &counters.solver;
            ui.label(format!(
                "Velocity assembly: {:.2}ms",
                solver.velocity_assembly_time.time()
            ));
            ui.label(format!(
                "Velocity resolution: {:.2}ms",
                counters.velocity_resolution_time()
            ));
            ui.label(format!(
                "Velocity integration: {:.2}ms",
                solver.velocity_update_time.time()
            ));
        });
        ui.label(format!("CCD: {:.2}ms", counters.ccd_time()));
        ui.indent("CCD", |ui| {
            ui.label(format!("# of substeps: {}", counters.ccd.num_substeps));
            ui.label(format!(
                "TOI computation: {:.2}ms",
                counters.ccd.toi_computation_time.time()
            ));
            ui.label(format!(
                "Broad-phase: {:.2}ms",
                counters.ccd.broad_phase_time.time()
            ));
            ui.label(format!(
                "Narrow-phase: {:.2}ms",
                counters.ccd.narrow_phase_time.time()
            ));
            ui.label(format!("Solver: {:.2}ms", counters.ccd.solver_time.time()));
        });
    }
}
