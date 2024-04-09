use bevy::prelude::*;

use crate::world::chunk::chunk_render_mesh::VoxelChunkSurface;
use crate::world::chunk::ChunkRenderStage;

#[derive(Default)]
pub struct ChunkApplyStandardMaterialsPlugin;

impl Plugin for ChunkApplyStandardMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (apply_standard_materials_system).in_set(ChunkRenderStage::ApplyMaterials),
        );
    }
}

fn apply_standard_materials_system(
    mut commands: ParallelCommands,
    surfaces: Query<(Entity, &VoxelChunkSurface), (Changed<VoxelChunkSurface>)>,
) {
    surfaces.par_iter().for_each(|(entity, surface)| {
        let data = surface;
    })
}
