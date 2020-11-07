use cglinalg::{
    Vector2,
    Vector3,
};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
struct Vertex {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    tex_coords: Vector2<f32>,
    tangent: Vector3<f32>,
    bitangent: Vector3<f32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum TextureKind {
    Diffuse,
    Specular,
    Normal,
    Height,
    Emission,
}

struct Texture {
    kind: TextureKind,
    path: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct TextureID {
    id: u32,
}

impl TextureID {
    #[inline]
    pub const fn new(id: u32) -> TextureID {
        TextureID {
            id: id,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct BackendTextureID {
    id: u32,
}

impl BackendTextureID {
    #[inline]
    pub const fn new(id: u32) -> BackendTextureID {
        TextureHandle {
            id: id
        }
    }
}

#[derive(Clone, Debug)]
struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    textures: Vec<Texture>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Mesh {
        Mesh {
            vertices: vertices,
            indices: indices,
            textures: textures,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct MeshID {
    id: u32,
}

impl MeshID {
    #[inline]
    pub const fn new(id: u32) -> MeshID {
        MeshID {
            id: id,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct BackendMeshID {
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl BackendMeshID {
    #[inline]
    pub const fn new(vao: u32, vbo: u32, ebo: u32) -> BackendMeshID {
        BackendMeshID {
            vao: vao,
            vbo: vbo,
            ebo: ebo
        }
    }
}

