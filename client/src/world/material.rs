use block_mesh::VoxelVisibility;

use game2::material::Material;

pub trait MaterialClientData {
    fn get_visibility(&self) -> VoxelVisibility;
}

impl MaterialClientData for Material {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            Material::AIR => VoxelVisibility::Empty,
            _ => VoxelVisibility::Opaque,
        }
    }
}
