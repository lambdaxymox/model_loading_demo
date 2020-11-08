use cglinalg::{
    Vector2,
    Vector3,
};
use wavefront_obj::obj;
use wavefront_obj::mtl;
use crate::texture;
use crate::texture::{
    TextureImage2D,
};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::{
    Read,
};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    tex_coords: Vector2<f32>,
    tangent: Vector3<f32>,
    bitangent: Vector3<f32>,
}

impl Vertex {
    pub fn zero() -> Vertex {
        Vertex {
            position: Vector3::zero(),
            normal: Vector3::zero(),
            tex_coords: Vector2::zero(),
            tangent: Vector3::zero(),
            bitangent: Vector3::zero(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TextureKind {
    Diffuse,
    Specular,
    Normal,
    Height,
    Emission,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextureID {
    id: u32,
}

#[derive(Clone)]
pub struct Texture {
    kind: TextureKind,
    name: String,
    data: TextureImage2D,
}


#[derive(Clone, Debug)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    textures: Vec<TextureID>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<TextureID>) -> Mesh {
        Mesh {
            vertices: vertices,
            indices: indices,
            textures: textures,
        }
    }
}

pub struct Model {
    textures_loaded: HashMap<TextureID, Texture>,
    meshes: Vec<Mesh>,
    name: String,
    gamma_correction: bool,
}

pub struct ModelLoadError {
    error: Box<dyn Error>,
}

pub fn load_from_memory(buffer: &[u8]) -> Result<Model, ModelLoadError> {
    let mut reader = io::Cursor::new(buffer);
    let mut zip_archive = zip::ZipArchive::new(reader).map_err(|e| {
        ModelLoadError {
            error: Box::new(e),
        }
    })?;
    let obj_set = {
        let obj_file_names = zip_archive
            .file_names()
            .filter(|file_name| { file_name.ends_with(".obj") })
            .collect::<Vec<&str>>();
        let obj_file_name = String::from(obj_file_names[0]);
        let mut obj_file = zip_archive.by_name(&obj_file_name).map_err(|e| {
            ModelLoadError {
                error: Box::new(e),
            }
        })?;
        let mut obj_buffer = String::new();
        obj_file.read_to_string(&mut obj_buffer).map_err(|e| {
            ModelLoadError {
                error: Box::new(e),
            }
        })?;
        let obj_set = obj::parse(&obj_buffer).map_err(|e| {
            ModelLoadError {
                error: Box::new(e),
            }
        })?;

        obj_set
    };
    let mtl_set = {
        let mtl_file_names = zip_archive
            .file_names()
            .filter(|file_name| { file_name.ends_with(".mtl") })
            .collect::<Vec<&str>>();
        let mtl_file_name = String::from(mtl_file_names[0]);
        let mut mtl_file = zip_archive.by_name(&mtl_file_name).map_err(|e| {
            ModelLoadError {
                error: Box::new(e),
            }
        })?;
        let mut mtl_buffer = String::new();
        mtl_file.read_to_string(&mut mtl_buffer).map_err(|e| {
            ModelLoadError {
                error: Box::new(e),
            }
        })?;
        let mtl_set = mtl::parse(&mtl_buffer).map_err(|e| {
            ModelLoadError {
                error: Box::new(e),
            }
        })?;

        mtl_set
    };
    unimplemented!();
}

