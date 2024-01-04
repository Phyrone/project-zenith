use block_mesh::RIGHT_HANDED_Y_UP_CONFIG;

use game2::Face;

trait FaceExt {}

impl FaceExt for Face {}

fn a() {
    RIGHT_HANDED_Y_UP_CONFIG.faces[face.get_face_index() as usize]
}
