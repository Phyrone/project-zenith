use std::ops::Deref;

use bevy::app::App;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use futures_lite::future;
use rayon::prelude::*;

use game2::{WithFixedSizeExt, CHUNK_SIZE};

use crate::world::chunk::chunk_data::ClientChunkData;
use crate::world::chunk::grid::ChunkGrid;
use crate::world::chunk::voxel::{
    create_voxel_chunk, voxels_grouped_greedy_mesh, GroupedVoxelMeshes,
};
use crate::world::chunk::{ChunkRenderStage, TextureIden, VoxelWorldFixedChunkPosition};
use crate::world::material::MaterialRegistry;

//TODO cancel tasks when chunk is removed
//TODO cancel tasks when chunk is updated before the task is finished
//TODO lod support
#[derive(Default)]
pub struct ChunkRenderMeshPlugin;

impl Plugin for ChunkRenderMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (create_build_mesh_tasks, apply_calculated_meshes_system)
                .chain()
                .in_set(ChunkRenderStage::ConstructMesh)
                .after(ChunkRenderStage::ChunkPreData),
        );
        app.add_systems(Update, (position_chunks));
    }
}

#[derive(Debug, Default, Component)]
#[component(storage = "SparseSet")]
pub struct ChunkRenderErrand;

#[derive(Debug, Component, Clone, Default)]
pub struct VoxelChunkSurface {
    pub iden: TextureIden,
}

impl VoxelChunkSurface {
    
}

impl Deref for VoxelChunkSurface {
    type Target = TextureIden;

    fn deref(&self) -> &Self::Target {
        &self.iden
    }
}

//similar to PbrBundle but without resource but an identifier to gain the resource later
#[derive(Bundle, Clone, Default)]
pub struct ChunkSurfaceBundle {
    pub surface: VoxelChunkSurface,
    pub mesh: Handle<Mesh>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Inherited visibility of an entity.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub view_visibility: ViewVisibility,
    pub aabb: Aabb,
}

fn create_build_mesh_tasks(
    commands: ParallelCommands,
    chunk_grid: Res<ChunkGrid>,
    material_registry: Res<MaterialRegistry>,
    chunks: Query<
        (Entity, &ClientChunkData, &VoxelWorldFixedChunkPosition),
        (With<ChunkRenderErrand>),
    >,
) {
    let pool = AsyncComputeTaskPool::get();
    chunks.par_iter().for_each(|(entity, data, pos)| {
        let neighbors: [Option<ClientChunkData>; 6] = chunk_grid
            .neighbours(pos.x, pos.y, pos.z)
            .iter()
            .map(|entity| entity.and_then(|e| chunks.get(e).ok()))
            .map(|chunk| chunk.map(|(_, data, _)| data.clone()))
            .collect::<Vec<_>>()
            .into_fixed_size::<6>();

        let data = data.clone();
        let registry = material_registry.clone();

        let greedy_mesh_task = pool.spawn(async move {
            //rust borrow checker is a pain
            //TODO optimize this and above (maybe unecessary and the compiler will optimize it away for me)
            let mapped = neighbors
                .iter()
                .map(|data| {
                    if let Some(data) = data {
                        Some(data.deref())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .into_fixed_size::<6>();

            let voxels = create_voxel_chunk(&registry, data.deref(), &mapped, 1);
            //TODO meshes might be needed in main world later for physics
            //voxels_grouped_mesh(&voxels, resolution, RenderAssetUsages::RENDER_WORLD)
            //TODO replace with greedy meshing (once i made tiling textures work + shader for it)
            voxels_grouped_greedy_mesh(&voxels, 1, RenderAssetUsages::RENDER_WORLD)
        });

        let greedy_mesh_task = GreedyMeshTask::new(greedy_mesh_task);

        commands.command_scope(|mut commands| {
            commands
                .entity(entity)
                .insert(greedy_mesh_task)
                .remove::<ChunkRenderErrand>();
        });
    });
}

fn position_chunks(
    commands: ParallelCommands,
    mut chunks: Query<(
        Entity,
        &VoxelWorldFixedChunkPosition,
        Option<&mut Transform>,
    )>,
) {
    chunks.par_iter_mut().for_each(|(entity, pos, transform)| {
        let new_position = Vec3::new(
            pos.x as f32 * CHUNK_SIZE as f32,
            pos.y as f32 * CHUNK_SIZE as f32,
            pos.z as f32 * CHUNK_SIZE as f32,
        );
        if let Some(mut transform) = transform {
            transform.translation = new_position;
        } else {
            commands.command_scope(|mut commands| {
                commands.entity(entity).insert(Transform {
                    translation: new_position,
                    scale: Vec3::splat(1.0),
                    ..default()
                });
            });
        }
    });
}

fn apply_calculated_meshes_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    surface_entities: Query<(Entity), With<VoxelChunkSurface>>,
    mut tasks: Query<(Entity, &mut GreedyMeshTask, Option<&Children>)>,
) {
    block_on(async {
        for (chunk_entity, mut greedy_mesh_task, children) in tasks.iter_mut() {
            let mut greedy_mesh_task = &mut greedy_mesh_task.task;
            let got_greedy_mesh = future::poll_once(&mut greedy_mesh_task).await;

            if let Some(to_render) = got_greedy_mesh {
                let bundles = to_render
                    .into_iter()
                    .map(|(iden, mesh)| (iden, meshes.add(mesh)))
                    .par_bridge()
                    .collect::<Vec<_>>();

                let mut surface_childs = if let Some(children) = children {
                    children
                        .iter()
                        .filter_map(|child| surface_entities.get(*child).ok())
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                };

                for (texture_iden, mesh) in bundles {
                    let surface = VoxelChunkSurface { iden: texture_iden };
                    if let Some(entity) = surface_childs.pop() {
                        commands.entity(entity).insert(ChunkSurfaceBundle {
                            surface,
                            mesh,
                            aabb: Aabb::from_min_max(
                                Vec3::splat(-1.0),
                                Vec3::splat(1.0 + CHUNK_SIZE as f32),
                            ),
                            ..default()
                        });
                    } else {
                        commands.spawn((surface, mesh)).set_parent(chunk_entity);
                    }
                }
                commands
                    .entity(chunk_entity)
                    .remove_children(&surface_childs);
                for entity in surface_childs {
                    commands.entity(entity).despawn_recursive();
                }
                commands.entity(chunk_entity).remove::<GreedyMeshTask>();
            };
        }
    })
}

#[derive(Debug, Component)]
struct GreedyMeshTask {
    task: Task<GroupedVoxelMeshes>,
}

impl GreedyMeshTask {
    fn new(task: Task<GroupedVoxelMeshes>) -> Self {
        Self { task }
    }
}
