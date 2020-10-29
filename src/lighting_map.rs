use crate::backend;
use crate::backend::{
    TextureImage2D
};


pub fn load_lighting_map(
    diffuse_buffer: &[u8], 
    specular_buffer: &[u8], 
    emission_buffer: &[u8]) -> LightingMap {

    let diffuse = backend::load_image(diffuse_buffer);
    let specular = backend::load_image(specular_buffer);
    let emission = backend::load_image(emission_buffer);

    LightingMap::new(diffuse, specular, emission)
}


pub struct LightingMap {
    pub diffuse: TextureImage2D,
    pub specular: TextureImage2D,
    pub emission: TextureImage2D,
}

impl LightingMap {
    pub fn new(diffuse: TextureImage2D, specular: TextureImage2D, emission: TextureImage2D) -> Self {
        Self {
            diffuse: diffuse,
            specular: specular,
            emission: emission,
        }
    }
}

