use bevy::prelude::{
    App, Changed, Entity, IntoSystemConfigs, Plugin, Query, RemovedComponents, ResMut, Resource,
    SystemSet, Update,
};
use bimap::BiHashMap;

use crate::world::chunk::RenderingWorldFixedChunk;

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, SystemSet)]
pub struct CreateChunkGridLabel;

pub struct ChunkGridPlugin;

pub const GRID_NEIGHBOUR_MAP: [(i32, i32, i32); 6] = [
    (0, 1, 0),
    (0, -1, 0),
    (-1, 0, 0),
    (1, 0, 0),
    (0, 0, 1),
    (0, 0, -1),
];

impl Plugin for ChunkGridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkGrid::default()).add_systems(
            Update,
            (update_updated_to_grid, update_removed_from_grid).in_set(CreateChunkGridLabel),
        );
    }
}

#[derive(Default, Debug, Clone, Resource)]
pub struct ChunkGrid {
    pub chunks: BiHashMap<(i64, i64, i64), Entity>,
}

impl ChunkGrid {
    #[inline]
    pub fn get(&self, x: i64, y: i64, z: i64) -> Option<Entity> {
        self.chunks.get_by_left(&(x, y, z)).copied()
    }

    ///returns an array of all 6 neighbours of the given chunk
    /// 1. up
    /// 2. down
    /// 3. left
    /// 4. right
    /// 5. front
    /// 6. back
    /// if a neighbour is not present in the grid it will be None
    /// the chunk given by x,y,z does not have to be in the grid
    pub fn neighbours(&self, x: i64, y: i64, z: i64) -> [Option<Entity>; 6] {
        [
            self.get(x, y + 1, z),
            self.get(x, y - 1, z),
            self.get(x - 1, y, z),
            self.get(x + 1, y, z),
            self.get(x, y, z + 1),
            self.get(x, y, z - 1),
        ]
    }
}

fn update_updated_to_grid(
    grid_res: ResMut<ChunkGrid>,
    chunks: Query<(Entity, &RenderingWorldFixedChunk), Changed<RenderingWorldFixedChunk>>,
) {
    if chunks.is_empty() {
        return;
    }
    let grid = grid_res.into_inner();
    for (entity_id, chunk) in chunks.iter() {
        grid.chunks.insert((chunk.x, chunk.y, chunk.z), entity_id);
    }
}

fn update_removed_from_grid(
    grid_res: ResMut<ChunkGrid>,
    mut chunks: RemovedComponents<RenderingWorldFixedChunk>,
) {
    if chunks.is_empty() {
        return;
    }
    let grid = grid_res.into_inner();
    for chunk in chunks.read() {
        grid.chunks.remove_by_right(&chunk);
    }
}
