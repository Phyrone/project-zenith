use std::any::Any;

use image::imageops::FilterType;
use image::{DynamicImage, GenericImage};
use rayon::prelude::*;

pub trait VoxelSurfaceTexture: Any {}

pub struct BlockTextures {}

impl BlockTextures {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct BlockTexturePack {
    textures: Vec<(String, DynamicImage)>,
}

impl BlockTexturePack {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
        }
    }

    pub fn new_with_textures(textures: Vec<(String, DynamicImage)>) -> Self {
        Self { textures }
    }
    pub fn add_texture(&mut self, name: String, image: DynamicImage) {
        self.textures.push((name, image));
    }

    pub fn render(&self) -> (DynamicImage, Vec<(String, Rect)>) {
        let time = std::time::Instant::now();
        let items = self.create_pack();
        let time = time.elapsed();
        println!("Time create pack layout: {:?}", time);

        let max_x = items
            .par_iter()
            .map(|i| i.rect.x + i.rect.w)
            .max()
            .unwrap_or(0);
        let max_y = items
            .par_iter()
            .map(|i| i.rect.y + i.rect.h)
            .max()
            .unwrap_or(0);
        let mut pack_image = DynamicImage::new_rgba8(max_x as u32, max_y as u32);
        let mut pack_data = Vec::new();
        for PackedItem {
            data: (name, image),
            rect,
        } in items
        {
            pack_image
                .copy_from(image, rect.x as u32, rect.y as u32)
                //TODO: Handle error
                .expect("Failed to copy texture to pack");

            pack_data.push((name.clone(), rect));
        }

        (pack_image, pack_data)
    }

    pub fn render_array(&self, resolution: u32) -> DynamicImage {
        let height = resolution * (self.textures.len() as u32);
        let width = resolution;
        let mut array_texture_image = DynamicImage::new_rgba8(width, height);
        for (i, (name, image)) in self.textures.iter().enumerate() {
            let needs_resize = image.width() != resolution || image.height() != resolution;
            let start_y = i as u32 * resolution;
            //holds ownership of resized image when needed until it is copied to array_texture_image
            let holder: DynamicImage;
            let resized = if needs_resize {
                holder = image.resize_exact(resolution, resolution, FilterType::Lanczos3);
                &holder
            } else {
                image
            };
            array_texture_image
                .copy_from(resized, 0, start_y)
                .expect("Failed to copy texture to array");
        }
        array_texture_image
    }

    fn create_pack(&self) -> Vec<PackedItem<(&String, &DynamicImage)>> {
        let items = self.to_items();

        let mut size = 8 * 1024;
        let items = loop {
            let rect = Rect::of_size(size, size);
            let pack_result = pack(rect, items.clone());
            match pack_result {
                Ok(items) => break items,
                Err(_) => {
                    size += 1024;
                    //TODO remove debug print
                    println!("Failed to pack items, increasing size to {}", size);
                    continue;
                }
            }
        };
        return items;
    }

    fn to_items(&self) -> Vec<Item<(&String, &DynamicImage)>> {
        self.textures
            .par_iter()
            .map(|(name, image)| Item {
                data: (name, image),
                w: image.width() as usize,
                h: image.height() as usize,
                rot: Rotation::None,
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_packing() {
        let time = std::time::Instant::now();
        let mut pack = BlockTexturePack::new();
        let base_path = "../assets/textures/";
        let mut paths = vec![];
        let range = 1..13;
        let colors = ["dark", "green", "light", "orange", "purple", "red"];
        for color in colors.iter() {
            for i in range.clone() {
                let path = format!("prototype/{}/texture_{:02}.png", color, i);
                println!("{}", path);
                paths.push(path);
            }
        }
        let time = time.elapsed();
        println!("Time to load textures: {:?}", time);

        let paths = paths
            .iter()
            .map(|p| format!("{}{}", base_path, p))
            .collect::<Vec<_>>();
        let images = paths
            .iter()
            .map(|p| image::open(p).unwrap())
            .collect::<Vec<_>>();
        for (path, image) in paths.into_iter().zip(images.into_iter()) {
            pack.add_texture(path, image);
        }
        let (image, _) = pack.render();
        //let image = pack.render_array(1024);
        image.save("pack.png").unwrap();
    }
}
