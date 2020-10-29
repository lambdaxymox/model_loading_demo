use cglinalg::{
    Matrix4,
    Vector3,
    ScalarFloat,
};


pub struct PointLight<S> {
    pub position: Vector3<S>,
    pub constant: S,
    pub linear: S,
    pub quadratic: S,
    pub ambient: Vector3<S>,
    pub diffuse: Vector3<S>,
    pub specular: Vector3<S>,
}

impl<S> PointLight<S> where S: ScalarFloat {
    pub fn new(
        position: Vector3<S>,
        constant: S,
        linear: S,
        quadratic: S,
        ambient: Vector3<S>,
        diffuse: Vector3<S>,
        specular: Vector3<S>) -> Self 
    {
        Self {
            position: position,
            constant: constant,
            linear: linear,
            quadratic: quadratic,
            ambient: ambient,
            diffuse: diffuse,
            specular: specular,
        }
    }
    
    pub fn model_matrix(&self) -> Matrix4<S> {
        Matrix4::from_affine_translation(&self.position)
    }
}

pub struct DirLight<S> {
    pub direction: Vector3<S>,
    pub ambient: Vector3<S>,
    pub diffuse: Vector3<S>,
    pub specular: Vector3<S>,
}

impl<S> DirLight<S> where S: ScalarFloat {
    pub fn new(
        direction: Vector3<S>,
        ambient: Vector3<S>,
        diffuse: Vector3<S>,
        specular: Vector3<S>) -> Self 
    {
        Self {
            direction: direction,
            ambient: ambient,
            diffuse: diffuse,
            specular: specular,
        }
    }
}


pub struct SpotLight<S> {
    pub position: Vector3<S>,
    pub direction: Vector3<S>,
    pub cutoff: S,
    pub outer_cutoff: S,

    pub constant: S,
    pub linear: S,
    pub quadratic: S,

    pub ambient: Vector3<S>,
    pub diffuse: Vector3<S>,
    pub specular: Vector3<S>
}

impl<S> SpotLight<S> where S: ScalarFloat {
    pub fn new(
        position: Vector3<S>,
        direction: Vector3<S>,
        cutoff: S,
        outer_cutoff: S,
        constant: S,
        linear: S,
        quadratic: S,
        ambient: Vector3<S>,
        diffuse: Vector3<S>,
        specular: Vector3<S>) -> SpotLight<S> 
    {
        Self {
            position: position,
            direction: direction,
            cutoff: cutoff,
            outer_cutoff: outer_cutoff,
            constant: constant,
            linear: linear,
            quadratic: quadratic,
            ambient: ambient,
            diffuse: diffuse,
            specular: specular
        }
    }

    pub fn update(&mut self, position: &Vector3<S>, direction: &Vector3<S>) {
        self.position = position.clone();
        self.direction = direction.clone();
    }
}

