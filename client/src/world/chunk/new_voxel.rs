use std::sync::Mutex;

use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssetUsages;
use hashbrown::HashMap;
use rayon::prelude::*;

use mesher::b32::{build_mesh32, VoxelCubeOcclusionMatrix32};
use mesher::meshing::quads_to_mesh;

use crate::world::chunk::{SurfaceTextureIden, VoxelSurfaceData};

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum VoxelMeshStage {
    CreateMesh,
}

pub struct VoxelMeshPlugin;

impl Plugin for VoxelMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            build_meshes_system.in_set(VoxelMeshStage::CreateMesh),
        );
    }
}


fn build_meshes_system(
    commands: ParallelCommands,
    mesh_handler: ResMut<Assets<Mesh>>,
    voxel_chunks: Query<
        (Entity, &VoxelCubeOcclusionMatrix32, &VoxelSurfaceData, &Children),
        Changed<VoxelCubeOcclusionMatrix32>,
    >,
    surface_entities: Query<(Entity, &SurfaceTextureIden), With<SurfaceTextureIden>>,
) {
    let mesh_handler = Mutex::new(mesh_handler);

    voxel_chunks
        .par_iter()
        .for_each(|(kube_entity, occlusion_matrix, textures, children)| {
            let outcome = build_mesh32(occlusion_matrix, |x, y, z, face|
            Some(1_usize),
            );

            let meshes = outcome
                .into_par_iter()
                .flat_map(|(key, quads)| {
                    let mesh = quads_to_mesh(&quads, 1.0, RenderAssetUsages::RENDER_WORLD);
                    let bounding_box = mesh.compute_aabb();
                    if let Some(bounding_box) = bounding_box {
                        Some((
                            key,
                            mesh,
                            bounding_box
                        ))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let mut mesh_handler = mesh_handler.lock().expect("Failed to lock mesh handler");
            let meshes = meshes
                .into_iter()
                .map(|(key, mesh,aabb)| (key, mesh_handler.add(mesh),aabb))
                .collect::<Vec<_>>();
            drop(mesh_handler);
            let mut surfaces = children
                .into_iter()
                .flat_map(|child| surface_entities.get(*child))
                .map(|(entity, surface_id)| (surface_id.id, entity))
                .collect::<HashMap<_, _>>();

            commands.command_scope(|mut command| {
                for (key, mesh,aabb) in meshes {
                    let entity = surfaces.remove(&key);
                    if let Some(surface_entity) = entity {
                        command.entity(surface_entity).insert((mesh,aabb));
                    } else {
                        let surface_id = SurfaceTextureIden { id: key };
                        command.spawn(ChunkSurfaceBundle {
                            surface_id,
                            mesh,
                            aabb,
                            ..Default::default()
                        }).set_parent(kube_entity);
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
    pub surface_id: SurfaceTextureIden,
    pub mesh: Handle<Mesh>,
    pub aabb: Aabb,
    //https://docs.rs/bevy/latest/bevy/render/prelude/struct.SpatialBundle.html
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

