use crate::cube_slice::SlicePartQuad;

///! generate quads of a binary slice
///! lod not implemented atm
pub fn greedy_mesh_binary_plane(mut data: [u32; 32], lod_size: u32) -> Vec<SlicePartQuad> {
    let mut greedy_quads = vec![];
    for row in 0..data.len() {
        let mut y = 0;
        while y < lod_size {
            // find first solid, "air/zero's" could be first so skip
            y += (data[row] >> y).trailing_zeros();
            if y >= lod_size {
                // reached top
                continue;
            }
            let h = (data[row] >> y).trailing_ones();
            // convert height 'num' to positive bits repeated 'num' times aka:
            // 1 = 0b1, 2 = 0b11, 4 = 0b1111
            let h_as_mask = u32::checked_shl(1, h).map_or(!0, |v| v - 1);
            let mask = h_as_mask << y;
            // grow horizontally
            let mut w = 1;
            while row + w < lod_size as usize {
                // fetch bits spanning height, in the next row
                let next_row_h = (data[row + w] >> y) & h_as_mask;
                if next_row_h != h_as_mask {
                    break; // can no longer expand horizontally
                }

                // nuke the bits we expanded into
                data[row + w] &= !mask;

                w += 1;
            }
            greedy_quads.push(SlicePartQuad {
                y: y as usize,
                width: w,
                height: h as usize,
                x: row,
            });
            y += h;
        }
    }
    greedy_quads
}
