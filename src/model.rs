use crate::texture;
use crate::texture::{
    TextureImage2D,
};
use cglinalg::{
    Vector2,
    Vector3,
};
use wavefront_obj::obj;
use wavefront_obj::mtl;
use std::collections::{
    HashMap,
};
use std::error::{
    Error, 
};
use std::io;
use std::io::{
    Read,
};
use zip::{
    ZipArchive,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TextureKind {
    Ambient,
    Diffuse,
    Specular,
    Bump,
    Emission,
}

#[derive(Clone)]
pub struct Texture {
    name: String,
    kind: TextureKind,
    data: TextureImage2D,
}

impl Texture {
    fn new(name: String, kind: TextureKind, data: TextureImage2D) -> Texture {
        Texture {
            name: name,
            kind: kind,
            data: data,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    name: String,
    vertices: Vec<Vertex>,
    vertex_indices: Vec<u32>,
    texture_indices: HashMap<TextureKind, u32>,
}

impl Mesh {
    fn new(
        name: String, 
        vertices: Vec<Vertex>, 
        vertex_indices: Vec<u32>, 
        texture_indices: HashMap<TextureKind, u32>) -> Mesh {
        
        Mesh {
            name: name,
            vertices: vertices,
            vertex_indices: vertex_indices,
            texture_indices: texture_indices,
        }
    }
}

pub struct Model {
    name: String,
    meshes: Vec<Mesh>,
    textures_loaded: Vec<Texture>,
    pub gamma_correction: bool,
}

impl Model {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    pub fn textures_loaded(&self) -> &[Texture] {
        &self.textures_loaded
    }
}

impl Model {
    fn new(
        name: String, 
        meshes: Vec<Mesh>, 
        textures_loaded: Vec<Texture>, 
        gamma_correction: bool) -> Model 
    {
        Model {
            name: name,
            meshes: meshes,
            textures_loaded: textures_loaded,
            gamma_correction: gamma_correction,
        }
    }
}

#[derive(Debug)]
pub struct ModelLoadError {
    error: Option<Box<dyn Error>>,
}

fn search_material_sets<'a>(
    material_sets: &'a [mtl::MaterialSet], 
    material_name: &str) -> Option<&'a mtl::Material> 
{
    for material_set in material_sets.iter() {
        for material in material_set.materials.iter() {
            if material_name == material.name {
                return Some(&material)
            }
        }
    }

    None
}

// TODO: Calculate normals from vertex data in the case that they're missing?
fn load_mesh_vertices(object: &obj::Object) -> Vec<Vertex> {
    let mut vertices = vec![];
    for element in object.element_set.iter() {
        match element {
            obj::Element::Face(vtn1, vtn2, vtn3) => {
                let triples = [
                    object.get_vtn_triple(*vtn1).unwrap(),
                    object.get_vtn_triple(*vtn2).unwrap(),
                    object.get_vtn_triple(*vtn3).unwrap(),
                ];
             
                for triple in triples.iter() {
                    match triple {
                        obj::VTNTriple::V(vp) => {
                            vertices.push(Vertex {
                                position: Vector3::new(vp.x as f32, vp.y as f32, vp.z as f32),
                                normal: Vector3::zero(),
                                tex_coords: Vector2::zero(),
                                tangent: Vector3::zero(),
                                bitangent: Vector3::zero(),
                            });
                        }
                        obj::VTNTriple::VT(vp, vt) => {
                            vertices.push(Vertex {
                                position: Vector3::new(vp.x as f32, vp.y as f32, vp.z as f32),
                                normal: Vector3::zero(),
                                tex_coords: Vector2::new(vt.u as f32, vt.v as f32),
                                tangent: Vector3::zero(),
                                bitangent: Vector3::zero(),
                            });
                        }
                        obj::VTNTriple::VN(vp, vn) => {
                            vertices.push(Vertex {
                                position: Vector3::new(vp.x as f32, vp.y as f32, vp.z as f32),
                                normal: Vector3::new(vn.x as f32, vn.y as f32, vn.z as f32),
                                tex_coords: Vector2::zero(),
                                tangent: Vector3::zero(),
                                bitangent: Vector3::zero(),
                            });
                        }
                        obj::VTNTriple::VTN(vp, vt, vn) => {
                            vertices.push(Vertex {
                                position: Vector3::new(vp.x as f32, vp.y as f32, vp.z as f32),
                                normal: Vector3::new(vn.x as f32, vn.y as f32, vn.z as f32),
                                tex_coords: Vector2::new(vt.u as f32, vt.v as f32),
                                tangent: Vector3::zero(),
                                bitangent: Vector3::zero(),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    vertices
}

fn lookup_texture(textures_loaded: &[Texture], texture_name: &str) -> Option<u32> {
    for i in 0..textures_loaded.len() {
        if textures_loaded[i].name == texture_name {
            return Some(i as u32);
        }
    }

    None
}

fn load_texture_map<R: io::Read + io::Seek>(
    zip_archive: &mut ZipArchive<R>, 
    textures_loaded: &mut Vec<Texture>,
    mesh_textures: &mut HashMap<TextureKind, u32>,
    texture_kind: TextureKind,
    texture_name: Option<&str>) -> Option<u32>
{
    if let Some(file_name) = texture_name {
        // If the texture has already been loaded, return the index of the already loaded
        // texture to save parsing and loading redundant textures to the GPU.
        if let Some(texture_index) = lookup_texture(&textures_loaded, file_name) {
            mesh_textures.insert(texture_kind, texture_index);
            return Some(texture_index);
        }

        let mut file = zip_archive.by_name(&file_name).ok()?;
        let texture_image = if file_name.ends_with(".png") {
            texture::from_png_reader(&mut file)
        } else {
            texture::from_jpeg_reader(&mut file)
        };
        let texture_map = Texture::new(
            file_name.to_owned(), 
            texture_kind, 
            texture_image
        );
        textures_loaded.push(texture_map);
        let texture_index = (textures_loaded.len() - 1) as u32;
        mesh_textures.insert(texture_kind, texture_index);
            
        Some(texture_index)
    } else {
        None
    }
}

pub fn load_from_memory(
    buffer: &[u8], 
    model_name: &str, 
    gamma_correction: bool) -> Result<Model, ModelLoadError> 
{
    let reader = io::Cursor::new(buffer);
    let mut zip_archive = zip::ZipArchive::new(reader).map_err(|e| {
        ModelLoadError {
            error: Some(Box::new(e)),
        }
    })?;
    let obj_file = {
        let obj_file_names: Vec<&str> = zip_archive
            .file_names()
            .filter(|file_name| file_name.ends_with(".obj"))
            .collect();
        let file_name = obj_file_names[0].to_owned();
        let mut file = zip_archive.by_name(&file_name).map_err(|e| {
            ModelLoadError {
                error: Some(Box::new(e)),
            }
        })?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).map_err(|e| {
            ModelLoadError {
                error: Some(Box::new(e)),
            }
        })?;

        buffer
    };
    let obj_set = obj::parse(&obj_file).map_err(|e| {
        ModelLoadError {
            error: Some(Box::new(e)),
        }
    })?;
    let mut mtl_sets = vec![];
    for material_library in obj_set.material_libraries.iter() {
        let mut file = zip_archive.by_name(&material_library).map_err(|e| {
            ModelLoadError {
                error: Some(Box::new(e))
            }
        })?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).map_err(|e| {
            ModelLoadError {
                error: Some(Box::new(e))
            }
        })?;
        let mtl_set = mtl::parse(&buffer).map_err(|e| {
            ModelLoadError {
                error: Some(Box::new(e)),
            }
        })?;
        mtl_sets.push(mtl_set);
    }

    let mut textures_loaded = vec![];
    let mut meshes = vec![];
    for object in obj_set.objects.iter() {
        assert!(object.geometry_set.len() == 1);

        let material_name = &object.geometry_set[0].material_name
            .as_ref()
            .map(|s| s.as_str())
            .ok_or(
                ModelLoadError {
                    error: None,
                }
            )?;
        let material = search_material_sets(&mtl_sets, &material_name).ok_or(
            ModelLoadError {
                error: None,
            }
        )?;

        let mesh_name = object.name.clone();
        let vertices = load_mesh_vertices(&object);
        let vertex_indices: Vec<u32> = (0..vertices.len() as u32).collect();
        let mut texture_indices = HashMap::new();
        load_texture_map(
            &mut zip_archive, 
            &mut textures_loaded,
            &mut texture_indices,
            TextureKind::Ambient,
            material.map_ambient.as_ref().map(|s| s.as_str()), 
        );
        load_texture_map(
            &mut zip_archive, 
            &mut textures_loaded,
            &mut texture_indices,
            TextureKind::Diffuse,
            material.map_diffuse.as_ref().map(|s| s.as_str()), 
        );
        load_texture_map(
            &mut zip_archive, 
            &mut textures_loaded,
            &mut texture_indices,
            TextureKind::Specular,
            material.map_specular.as_ref().map(|s| s.as_str()), 
        );
        load_texture_map(
            &mut zip_archive, 
            &mut textures_loaded,
            &mut texture_indices,
            TextureKind::Bump,
            material.map_bump.as_ref().map(|s| s.as_str()), 
        );
        load_texture_map(
            &mut zip_archive, 
            &mut textures_loaded,
            &mut texture_indices,
            TextureKind::Emission,
            material.map_ambient.as_ref().map(|s| s.as_str()), 
        );

        let mesh = Mesh::new(mesh_name, vertices, vertex_indices, texture_indices);
        meshes.push(mesh);
    }

    Ok(Model::new(model_name.to_owned(), meshes, textures_loaded, gamma_correction))
}

