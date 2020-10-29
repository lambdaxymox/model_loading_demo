use cglinalg::{
    Vector3,
    ScalarFloat,
};
use std::collections::hash_map::HashMap;


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Material<S> {
    pub ambient: Vector3<S>,
    pub diffuse: Vector3<S>,
    pub specular: Vector3<S>,
    pub specular_exponent: S,
}

impl<S> Material<S> where S: ScalarFloat {
    fn new(ambient: Vector3<S>, diffuse: Vector3<S>, specular: Vector3<S>, specular_exponent: S) -> Material<S> {
        Material {
            ambient: ambient,
            diffuse: diffuse,
            specular: specular,
            specular_exponent: specular_exponent,
        }
    }
}

/// A table of materials for the Blinn-Phong shading model.
/// There material parameters are derived from the OpenGL `teapots.c` demo, 
/// c.f. `Silicon Graphics, Inc., 1994, Mark J. Kilgard` and the table found
/// (here)[http://devernay.free.fr/cours/opengl/materials.html]
fn raw_sgi_material_table() -> HashMap<&'static str, Material<f32>> {
    let materials = [
        ("emerald", Material::new(
            Vector3::new(0.0215, 0.1745, 0.0215),
            Vector3::new(0.07568, 0.61424, 0.07568),
            Vector3::new(0.5, 0.5, 0.4),
            0.6
        )),
        ("jade", Material::new(
            Vector3::new(0.135, 0.2225, 0.1575), 
            Vector3::new(0.54, 0.89, 0.63), 
            Vector3::new(0.316228, 0.316228, 0.316228), 
            0.1
        )),
        ("obsidian", Material::new(
            Vector3::new(0.05375, 0.05, 0.06625),
            Vector3::new(0.18275, 0.17, 0.22525),
            Vector3::new(0.332741, 0.328634, 0.346435),
            0.3
        )),
        ("pearl", Material::new(
            Vector3::new(0.25, 0.20725, 0.20725),
            Vector3::new(1.0, 0.829, 0.829),
            Vector3::new(0.296648, 0.296648, 0.296648),
            0.088
        )),
        ("ruby", Material::new(
            Vector3::new(0.1745, 0.01175, 0.01175),
            Vector3::new(0.61424, 0.04136, 0.04136),
            Vector3::new(0.727811, 0.626959, 0.626959),
            0.6
        )),
        ("turquoise", Material::new(
            Vector3::new(0.1, 0.18725, 0.1745),
            Vector3::new(0.396, 0.74151, 0.69102),
            Vector3::new(0.297254, 0.30829, 0.306678),
            0.1
        )),
        ("brass", Material::new(
            Vector3::new(0.1, 0.18725, 0.1745),
            Vector3::new(0.780392, 0.568627, 0.113725),
            Vector3::new(0.992157, 0.941176, 0.807843),
            0.21794872
        )),
        ("bronze", Material::new(
            Vector3::new(0.2125, 0.1275, 0.054),
            Vector3::new(0.714, 0.4284, 0.18144),
            Vector3::new(0.393548, 0.271906, 0.166721),
            0.2
        )),
        ("chrome", Material::new(
            Vector3::new(0.25, 0.25, 0.25),
            Vector3::new(0.4, 0.4, 0.4),
            Vector3::new(0.774597, 0.774597, 0.774597),
            0.6
        )),
        ("copper", Material::new(
            Vector3::new(0.0735, 0.0225, 0.7038),
            Vector3::new(0.7038, 0.27048, 0.0828),
            Vector3::new(0.256777, 0.137622, 0.086014),
            0.1
        )),
        ("gold", Material::new(
            Vector3::new(0.1995, 0.0745, 0.75164),
            Vector3::new(0.75164, 0.60648, 0.22648),
            Vector3::new(0.628281, 0.555802, 0.366065),
            0.4
        )),
        ("silver", Material::new(
            Vector3::new(0.19225, 0.19225, 0.50754),
            Vector3::new(0.50754, 0.50754, 0.50754),
            Vector3::new(0.508273, 0.508273, 0.508273),
            0.4
        )),
        ("black plastic", Material::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.01, 0.01, 0.01),
            Vector3::new(0.5, 0.5, 0.5),
            0.25
        )),
        ("cyan plastic", Material::new(
            Vector3::new(0.0, 0.1, 0.06),
            Vector3::new(0.0, 0.50980392, 0.50980392),
            Vector3::new(0.50196078, 0.50196078, 0.50196078),
            0.25
        )),
        ("green plastic", Material::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.1, 0.35, 0.1),
            Vector3::new(0.45, 0.55, 0.45),
            0.25
        )),
        ("red plastic", Material::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.5, 0.0, 0.0),
            Vector3::new(0.7, 0.6, 0.6),
            0.25
        )),
        ("white plastic", Material::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.55, 0.55, 0.55),
            Vector3::new(0.70, 0.70, 0.70),
            0.25
        )),
        ("yellow plastic", Material::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.5, 0.5, 0.0),
            Vector3::new(0.6, 0.6, 0.5),
            0.25
        )),
        ("black rubber", Material::new(
            Vector3::new(0.02, 0.02, 0.02),
            Vector3::new(0.01, 0.01, 0.01),
            Vector3::new(0.4, 0.4, 0.4),
            0.078125
        )),
        ("cyan rubber", Material::new(
            Vector3::new(0.0,0.05, 0.05),
            Vector3::new(0.4, 0.5, 0.5),
            Vector3::new(0.04, 0.7, 0.7),
            0.078125
        )),
        ("green rubber", Material::new(
            Vector3::new(0.0, 0.05, 0.0),
            Vector3::new(0.4, 0.5, 0.4),
            Vector3::new(0.04, 0.7, 0.04),
            0.078125
        )),
        ("red rubber", Material::new(
            Vector3::new(0.0, 0.05, 0.0),
            Vector3::new(0.5, 0.4, 0.4),
            Vector3::new(0.7, 0.04, 0.04),
            0.078125
        )),
        ("white rubber", Material::new(
            Vector3::new(0.05, 0.05, 0.05),
            Vector3::new( 	0.5, 0.5, 0.5),
            Vector3::new(0.7, 0.7, 0.7),
            0.078125
        )),
        ("yellow rubber", Material::new(
            Vector3::new(0.05, 0.05, 0.0),
            Vector3::new(0.5, 0.5, 0.4),
            Vector3::new(0.7, 0.7, 0.04),
            0.078125
        ))
    ].iter().map(|p| *p).collect();
    
    materials
}

/// Create a table of materials for the Blinn-Phong shading model that can be sent to the 
/// GPU directly derived from the original Silicon Graphics Institute data.
pub fn sgi_material_table() -> HashMap<&'static str, Material<f32>> {
    raw_sgi_material_table()
        .iter()
        .map(|(name, material)| { (*name, Material::new(
            material.ambient, 
            material.diffuse, 
            material.specular, 
            128.0 * material.specular_exponent
        ))})
        .collect::<HashMap<&'static str, Material<f32>>>()
}
