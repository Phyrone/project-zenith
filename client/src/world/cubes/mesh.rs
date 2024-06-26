use std::sync::Mutex;

use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssetUsages;
use common::CHUNK_SIZE;
use hashbrown::HashMap;
use rayon::prelude::*;

use mesher::b32::{build_mesh32, VoxelCubeOcclusionMatrix32};
use mesher::meshing::quads_to_mesh;

use crate::world::cubes::{ChunkRenderStage, SurfaceMaterial, VoxelCubeStore};

pub struct VoxelMeshPlugin;

impl Plugin for VoxelMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_static_neighbors.in_set(ChunkRenderStage::ComputeOccupied),
                build_meshes_system.in_set(ChunkRenderStage::ComputeMesh),
            ),
        );
    }
}

fn sync_static_neighbors() {}

#[allow(clippy::type_complexity)]
fn build_meshes_system(
    commands: ParallelCommands,
    mesh_handler: ResMut<Assets<Mesh>>,
    voxel_chunks: Query<
        (
            Entity,
            &VoxelCubeOcclusionMatrix32,
            &VoxelCubeStore,
            &Children,
        ),
        (Or<(Changed<VoxelCubeOcclusionMatrix32>, Changed<VoxelCubeStore>)>),
    >,
    surface_entities: Query<(Entity, &SurfaceMaterial), With<Parent>>,
) {
    let mesh_handler = Mutex::new(mesh_handler);

    voxel_chunks
        .par_iter()
        .for_each(|(kube_entity, occlusion_matrix, textures, children)| {
            let outcome = build_mesh32(occlusion_matrix, |x, y, z, face| {
                let index = x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE;
                //TODO: per direction textures
                textures.get(index).to_owned()
            });

            let meshes = outcome
                .into_par_iter()
                .flat_map(|(key, quads)| {
                    let mesh = quads_to_mesh(&quads, 1.0, RenderAssetUsages::RENDER_WORLD);
                    //build aabb for frustum culling
                    mesh.compute_aabb()
                        .map(|bounding_box| (key, mesh, bounding_box))
                })
                .collect::<Vec<_>>();

            let mut mesh_handler = mesh_handler.lock().expect("Failed to lock mesh handler");
            let meshes = meshes
                .into_iter()
                .map(|(key, mesh, aabb)| (key, mesh_handler.add(mesh), aabb))
                .collect::<Vec<_>>();
            drop(mesh_handler);
            let mut surfaces = children
                .into_iter()
                .flat_map(|child| surface_entities.get(*child))
                .map(|(entity, surface_id)| (surface_id, entity))
                .collect::<HashMap<_, _>>();

            commands.command_scope(|mut command| {
                for (surface_iden, mesh, aabb) in meshes {
                    let entity = surfaces.remove(&surface_iden);
                    if let Some(surface_entity) = entity {
                        command.entity(surface_entity).insert((mesh, aabb));
                    } else {
                        command
                            .spawn(ChunkSurfaceBundle {
                                surface_iden,
                                mesh,
                                aabb,
                                ..Default::default()
                            })
                            .set_parent(kube_entity);
                    }
                }
                for (_, entity) in surfaces {
                    command.entity(entity).despawn_recursive();
                }
            })
        });
}

#[derive(Bundle, Default)]
pub struct ChunkSurfaceBundle {
    pub surface_iden: SurfaceMaterial,
    pub mesh: Handle<Mesh>,
    pub aabb: Aabb,
    //https://docs.rs/bevy/latest/bevy/render/prelude/struct.SpatialBundle.html
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
