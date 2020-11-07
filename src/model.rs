use crate::mesh::{
    Mesh,
    Texture,
};


pub fn load_string(data: &str) -> Model {

}

struct Model {
    textures_loaded: Vec<Texture>,
    meshes: Vec<Mesh>,
    directory: Option<String>,
    gamma_correction: bool,
}

impl Model {
    fn new() -> Model {
        Model {
            textures_loaded: vec![],
            meshes: vec![],
            director: None,
            gamma_correction: false,
        }
    }
}

struct ModelBuilder {
    model: Model,
    directory: Option<String>,
    gamma_correction: Option<bool>,
}

impl ModelBuilder {
    pub fn new() -> ModelBuilder {
        ModelBuilder {
            model: Model::new(),
            directory: None,
            gamma_correction: None,
        }
    }

    pub fn with_directory(&mut self, directory: &str) {
        self.directory = Some(String::from(directory));
    }

    pub fn with_gamma_correction(&mut self, gamma_correction: bool) {
        self.gamma_correction = Some(gamma_correction);
    }

    fn process_node(&mut self, node: &assimp::Node, scene: &assimp::Scene) {
        
    }

    pub fn with_model(&mut self, data: &str) {
        fn smooth_normals() -> assimp::GenerateNormals {
            assimp::GenerateNormals {
                enable: true,
                smooth: true,
                max_smoothing_angle: 175.0,
            }
        }

        fn calc_tangent_space_args() -> assimp::CalcTangentSpace {
            assimp::CalcTangentSpace {
                enable: true,
                max_smoothing_angle: 45.0,
                texture_channel: 0,
            }
        }

        let mut importer = assimp::Importer::new();
        let possible_scene = importer
            .triangulate(true)
            .generate_normals(smooth_normals())
            .flip_uvs(true)
            .calc_tangent_space(calc_tangent_space_args())
            .read_string(data);

        if let Ok(scene) = possible_scene {
            self.process_node(&scene.root_node(), &scene);
        } else {
            panic!("Could not parse scene");
        }
    }
}

