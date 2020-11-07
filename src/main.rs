extern crate glfw;
extern crate cglinalg;
extern crate cgperspective;
extern crate image;
extern crate log;
extern crate file_logger;
extern crate mini_obj;
extern crate assimp;


mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod backend;
mod camera;
mod light;
mod material;
mod lighting_map;


use material::{
    Material,
};
use cglinalg::{
    Angle,
    Degrees,
    Matrix4,
    Radians,
    Vector3,
    Unit,
};
use cgperspective::{
    SimpleCameraMovement,
    CameraMovement,
    CameraAttitudeSpec,
    PerspectiveFovSpec,
    FreeKinematicsSpec,
    Camera
};
use glfw::{
    Action, 
    Context, 
    Key
};
use gl::types::{
    GLfloat,
    GLint,
    GLuint, 
    GLvoid, 
    GLsizeiptr,
};
use log::{
    info
};
use mini_obj::{
    ObjMesh
};

use crate::backend::{
    OpenGLContext,
    ShaderSourceBuilder,
    ShaderSource,
    ShaderHandle,
};
use crate::camera::{
    PerspectiveFovCamera,
};
use crate::lighting_map::{
    LightingMap,
};
use crate::light::*;

use std::mem;
use std::ptr;


// Default value for the color buffer.
const CLEAR_COLOR: [f32; 4] = [0.1_f32, 0.1_f32, 0.1_f32, 1.0_f32];
// Default value for the depth buffer.
const CLEAR_DEPTH: [f32; 4] = [1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32];

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;


fn create_box_mesh() -> ObjMesh {
    let points: Vec<[f32; 3]> = vec![
        [-0.5, -0.5, -0.5], [ 0.5, -0.5, -0.5], [ 0.5,  0.5, -0.5],
        [ 0.5,  0.5, -0.5], [-0.5,  0.5, -0.5], [-0.5, -0.5, -0.5],
        [-0.5, -0.5,  0.5], [ 0.5, -0.5,  0.5], [ 0.5,  0.5,  0.5],  
        [ 0.5,  0.5,  0.5], [-0.5,  0.5,  0.5], [-0.5, -0.5,  0.5],
        [-0.5,  0.5,  0.5], [-0.5,  0.5, -0.5], [-0.5, -0.5, -0.5], 
        [-0.5, -0.5, -0.5], [-0.5, -0.5,  0.5], [-0.5,  0.5,  0.5], 
        [ 0.5,  0.5,  0.5], [ 0.5,  0.5, -0.5], [ 0.5, -0.5, -0.5], 
        [ 0.5, -0.5, -0.5], [ 0.5, -0.5,  0.5], [ 0.5,  0.5,  0.5],
        [-0.5, -0.5, -0.5], [ 0.5, -0.5, -0.5], [ 0.5, -0.5,  0.5],  
        [ 0.5, -0.5,  0.5], [-0.5, -0.5,  0.5], [-0.5, -0.5, -0.5],
        [-0.5,  0.5, -0.5], [ 0.5,  0.5, -0.5], [ 0.5,  0.5,  0.5], 
        [ 0.5,  0.5,  0.5], [-0.5,  0.5,  0.5], [-0.5,  0.5, -0.5],  
    ];
    let tex_coords = vec![
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0],
        [1.0, 1.0], [0.0, 1.0], [0.0, 0.0],
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0],
        [1.0, 1.0], [0.0, 1.0], [0.0, 0.0],
        [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        [0.0, 1.0], [0.0, 0.0], [1.0, 0.0],
        [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        [0.0, 1.0], [0.0, 0.0], [1.0, 0.0],
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0],
        [1.0, 0.0], [0.0, 0.0], [0.0, 1.0],
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0],
        [1.0, 0.0], [0.0, 0.0], [0.0, 1.0]
    ];
    let normals = vec![
        [ 0.0,  0.0, -1.0], [ 0.0,  0.0, -1.0], [ 0.0,  0.0, -1.0],
        [ 0.0,  0.0, -1.0], [ 0.0,  0.0, -1.0], [ 0.0,  0.0, -1.0],
        [ 0.0,  0.0,  1.0], [ 0.0,  0.0,  1.0], [ 0.0,  0.0,  1.0],
        [ 0.0,  0.0,  1.0], [ 0.0,  0.0,  1.0], [ 0.0,  0.0,  1.0],
        [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0],
        [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0],
        [ 1.0,  0.0,  0.0], [ 1.0,  0.0,  0.0], [ 1.0,  0.0,  0.0],
        [ 1.0,  0.0,  0.0], [ 1.0,  0.0,  0.0], [ 1.0,  0.0,  0.0],
        [ 0.0, -1.0,  0.0], [ 0.0, -1.0,  0.0], [ 0.0, -1.0,  0.0],
        [ 0.0, -1.0,  0.0], [ 0.0, -1.0,  0.0], [ 0.0, -1.0,  0.0],
        [ 0.0,  1.0,  0.0], [ 0.0,  1.0,  0.0], [ 0.0,  1.0,  0.0],
        [ 0.0,  1.0,  0.0], [ 0.0,  1.0,  0.0], [ 0.0,  1.0,  0.0],
    ];

    ObjMesh::new(points, tex_coords, normals)
}

fn create_box_positions() -> Vec<Vector3<f32>> {
    vec![
        Vector3::new( 0.0,  0.0,  0.0),
        Vector3::new( 2.0,  5.0, -15.0),
        Vector3::new(-1.5, -2.2, -2.5),
        Vector3::new(-3.8, -2.0, -12.3),
        Vector3::new( 2.4, -0.4, -3.5),
        Vector3::new(-1.7,  3.0, -7.5),
        Vector3::new( 1.3, -2.0, -2.5),
        Vector3::new( 1.5,  2.0, -2.5),
        Vector3::new( 1.5,  0.2, -1.5),
        Vector3::new(-1.3,  1.0, -1.5)
    ]
}

fn create_box_model_matrices(box_positions: &[Vector3<f32>]) -> Vec<Matrix4<f32>> {
    let mut box_model_matrices = vec![];
    for i in 0..box_positions.len() {
        let translation_i = Matrix4::from_affine_translation(&box_positions[i]);
        let angle_i = Radians(20.0 * (i as f32));
        let axis_i = Unit::from_value(Vector3::new(1.0, 0.3, 0.5));
        let rotation_i = Matrix4::from_affine_axis_angle(&axis_i, angle_i);
        let model_i = rotation_i * translation_i;
        box_model_matrices.push(model_i);
    }

    debug_assert_eq!(box_model_matrices.len(), box_positions.len());

    box_model_matrices
}

fn create_camera(width: u32, height: u32) -> PerspectiveFovCamera<f32> {
    let near = 0.1;
    let far = 100.0;
    let fovy = Degrees(72.0);
    let aspect = width as f32 / height as f32;
    let model_spec = PerspectiveFovSpec::new(
        fovy, 
        aspect, 
        near, 
        far
    );
    let position = Vector3::new(0.0, 0.0, 3.0);
    let forward = Vector3::new(0.0, 0.0, 1.0);
    let right = Vector3::new(1.0, 0.0, 0.0);
    let up  = Vector3::new(0.0, 1.0, 0.0);
    let axis = Vector3::new(0.0, 0.0, -1.0);
    let attitude_spec = CameraAttitudeSpec::new(
        position,
        forward,
        right,
        up,
        axis,
    );
    let movement_speed = 5.0;
    let rotation_speed = Degrees(50.0);
    let kinematics_spec = FreeKinematicsSpec::new(
        movement_speed, 
        rotation_speed
    );

    Camera::new(&model_spec, &attitude_spec, &kinematics_spec)
}

fn create_cube_lights() -> [PointLight<f32>; 4] {
    let position_0 = Vector3::new(0.7, 0.2, 2.0);
    let ambient_0 = Vector3::new(0.2, 0.2, 0.2);
    let diffuse_0 = Vector3::new(0.5, 0.5, 0.5);
    let specular_0 = Vector3::new(1.0, 1.0, 1.0);
    let constant_0 = 1.0;
    let linear_0 = 0.09;
    let quadratic_0 = 0.032;
    let light_0 = PointLight::new(
        position_0,
        constant_0,
        linear_0,
        quadratic_0,
        ambient_0, 
        diffuse_0, 
        specular_0
    );

    let position_1 = Vector3::new(2.3, -3.3, -4.0);
    let ambient_1 = Vector3::new(0.2, 0.2, 0.2);   
    let diffuse_1 = Vector3::new(0.5, 0.5, 0.5);
    let specular_1 = Vector3::new(1.0, 1.0, 1.0);
    let constant_1 = 1.0;
    let linear_1 = 0.09;
    let quadratic_1 = 0.032;
    let light_1 = PointLight::new(
        position_1,
        constant_1,
        linear_1,
        quadratic_1,
        ambient_1,
        diffuse_1,
        specular_1
    );


    let position_2 = Vector3::new(-4.0, 2.0, -12.0);
    let ambient_2 = Vector3::new(0.2, 0.2, 0.2);
    let diffuse_2 = Vector3::new(0.5, 0.5, 0.5);
    let specular_2 = Vector3::new(1.0, 1.0, 1.0);
    let constant_2 = 1.0;
    let linear_2 = 0.09;
    let quadratic_2 = 0.032;
    let light_2 = PointLight::new(
        position_2,
        constant_2,
        linear_2,
        quadratic_2,
        ambient_2, 
        diffuse_2, 
        specular_2
    );

    let position_3 = Vector3::new(0.0, 0.0, -3.0);
    let ambient_3 = Vector3::new(0.05, 0.05, 0.05);
    let diffuse_3 = Vector3::new(0.8, 0.8, 0.8);
    let specular_3 = Vector3::new(1.0, 1.0, 1.0);
    let constant_3 = 1.0;
    let linear_3 = 0.09;
    let quadratic_3 = 0.032;
    let light_3 = PointLight::new(
        position_3,
        constant_3,
        linear_3,
        quadratic_3,
        ambient_3,
        diffuse_3,
        specular_3
    );

    [light_0, light_1, light_2, light_3]
}

fn create_directional_light() -> DirLight<f32> {
    let direction = Vector3::new(-0.2, -1.0, -0.3);
    let ambient = Vector3::new(0.05, 0.05, 0.05);
    let diffuse = Vector3::new(0.4, 0.4, 0.4);
    let specular = Vector3::new(0.5, 0.5, 0.5);

    DirLight::new(direction, ambient, diffuse, specular)
}

fn create_spotlight(camera: &PerspectiveFovCamera<f32>) -> SpotLight<f32> {
    let position = camera.position();
    let direction = camera.forward_axis();
    let ambient = Vector3::new(0.0, 0.0, 0.0);
    let diffuse = Vector3::new(1.0, 1.0, 1.0);
    let specular = Vector3::new(1.0, 1.0, 1.0);
    let constant = 1.0;
    let linear = 0.09;
    let quadratic = 0.032;
    let cutoff = Degrees(12.5).cos();
    let outer_cutoff = Degrees(15.0).cos();

    SpotLight::new(
        position,
        direction,
        cutoff,
        outer_cutoff,
        constant,
        linear,
        quadratic,
        ambient,
        diffuse,
        specular
    )
}

fn create_lighting_map() -> LightingMap {
    let diffuse_buffer = include_bytes!("../assets/container2_diffuse.png");
    let specular_buffer = include_bytes!("../assets/container2_specular.png");
    let emission_buffer = include_bytes!("../assets/container2_emission.png");
    
    lighting_map::load_lighting_map(diffuse_buffer, specular_buffer, emission_buffer)
}

fn send_to_gpu_uniforms_cube_light_mesh(shader: ShaderHandle, model_mat: &Matrix4<f32>) {
    shader.use_program();
    shader.set_mat4("model", model_mat);
}

fn send_to_gpu_uniforms_camera(shader: ShaderHandle, camera: &PerspectiveFovCamera<f32>) {
    shader.use_program();
    shader.set_mat4("camera.projection", &camera.projection());
    shader.set_mat4("camera.view", camera.view_matrix());    
}

fn send_to_gpu_uniforms_dir_light(shader: ShaderHandle, light: &DirLight<f32>) {
    shader.use_program();
    shader.set_vec3("dirLight.direction", &light.direction);
    shader.set_vec3("dirLight.ambient", &light.ambient);
    shader.set_vec3("dirLight.diffuse", &light.diffuse);
    shader.set_vec3("dirLight.specular", &light.specular);
}

/// Send the uniforms for the lighting data to the GPU for the mesh.
/// Note that in order to render multiple lights in the shader, we define an 
/// array of structs. In OpenGL, each elementary member of a struct is 
/// considered to be a uniform variable, and each struct is a struct of uniforms. 
/// Consequently, if every element of an array of struct uniforms is not used in 
/// the shader, OpenGL will optimize those uniform locations out at runtime. This
/// will cause OpenGL to return a `GL_INVALID_VALUE` on a call to 
/// `glGetUniformLocation`.
fn send_to_gpu_uniforms_point_lights(shader: ShaderHandle, lights: &[PointLight<f32>; 4]) {
    shader.use_program();

    shader.set_vec3("pointLights[0].position", &lights[0].position);
    shader.set_float("pointLights[0].constant", lights[0].constant);
    shader.set_float("pointLights[0].linear", lights[0].linear);
    shader.set_float("pointLights[0].quadratic", lights[0].quadratic);
    shader.set_vec3("pointLights[0].ambient", &lights[0].ambient);
    shader.set_vec3("pointLights[0].diffuse", &lights[0].diffuse);
    shader.set_vec3("pointLights[0].specular", &lights[0].specular);

    shader.set_vec3("pointLights[1].position", &lights[1].position);
    shader.set_float("pointLights[1].constant", lights[1].constant);
    shader.set_float("pointLights[1].linear", lights[1].linear);
    shader.set_float("pointLights[1].quadratic", lights[1].quadratic);
    shader.set_vec3("pointLights[1].ambient", &lights[1].ambient);
    shader.set_vec3("pointLights[1].diffuse", &lights[1].diffuse);
    shader.set_vec3("pointLights[1].specular", &lights[1].specular);

    shader.set_vec3("pointLights[2].position", &lights[2].position);
    shader.set_float("pointLights[2].constant", lights[2].constant);
    shader.set_float("pointLights[2].linear", lights[2].linear);
    shader.set_float("pointLights[2].quadratic", lights[2].quadratic);
    shader.set_vec3("pointLights[2].ambient", &lights[2].ambient);
    shader.set_vec3("pointLights[2].diffuse", &lights[2].diffuse);
    shader.set_vec3("pointLights[2].specular", &lights[2].specular);

    shader.set_vec3("pointLights[3].position", &lights[3].position);
    shader.set_float("pointLights[3].constant", lights[3].constant);
    shader.set_float("pointLights[3].linear", lights[3].linear);
    shader.set_float("pointLights[3].quadratic", lights[3].quadratic);
    shader.set_vec3("pointLights[3].ambient", &lights[3].ambient);
    shader.set_vec3("pointLights[3].diffuse", &lights[3].diffuse);
    shader.set_vec3("pointLights[3].specular", &lights[3].specular);
}

/// Send the uniforms for the lighting data to the GPU for the mesh.
/// Note that in order to render multiple lights in the shader, we define an 
/// array of structs. In OpenGL, each elementary member of a struct is 
/// considered to be a uniform variable, and each struct is a struct of uniforms. 
/// Consequently, if every element of an array of struct uniforms is not used in 
/// the shader, OpenGL will optimize those uniform locations out at runtime. This
/// will cause OpenGL to return a `GL_INVALID_VALUE` on a call to 
/// `glGetUniformLocation`.
fn send_to_gpu_uniforms_spotlight(shader: ShaderHandle, light: &SpotLight<f32>) {
    shader.use_program();
    shader.set_vec3("spotLight.position", &light.position);
    shader.set_vec3("spotLight.direction", &light.direction);
    shader.set_float("spotLight.cutOff", light.cutoff);
    shader.set_float("spotLight.outerCutOff", light.outer_cutoff);
    shader.set_vec3("spotLight.ambient", &light.ambient);
    shader.set_vec3("spotLight.diffuse", &light.diffuse);
    shader.set_vec3("spotLight.specular", &light.specular);
    shader.set_float("spotLight.constant", light.constant);
    shader.set_float("spotLight.linear", light.linear);
    shader.set_float("spotLight.quadratic", light.quadratic);
}

fn send_to_gpu_textures_material(lighting_map: &LightingMap) -> (GLuint, GLuint, GLuint) {
    let diffuse_tex = backend::send_to_gpu_texture(&lighting_map.diffuse, gl::REPEAT).unwrap();
    let specular_tex = backend::send_to_gpu_texture(&lighting_map.specular, gl::REPEAT).unwrap();
    let emission_tex = backend::send_to_gpu_texture(&lighting_map.emission, gl::REPEAT).unwrap();

    (diffuse_tex, specular_tex, emission_tex)
}

#[derive(Copy, Clone)]
struct MaterialUniforms<'a> {
    diffuse_index: i32,
    specular_index: i32,
    emission_index: i32,
    material: &'a Material<f32>,
}

impl<'a> MaterialUniforms<'a> {
    fn new(
        diffuse_index: i32,
        specular_index: i32,
        emission_index: i32,
        material: &'a Material<f32>) -> Self
    {
        Self {
            diffuse_index: diffuse_index,
            specular_index: specular_index,
            emission_index: emission_index,
            material: material,
        }
    }
}

fn send_to_gpu_uniforms_material(shader: ShaderHandle, uniforms: MaterialUniforms) {
    shader.use_program();
    shader.set_int("material.diffuse", uniforms.diffuse_index);
    shader.set_int("material.specular", uniforms.specular_index);
    shader.set_int("material.emission", uniforms.emission_index);
    shader.set_float("material.specular_exponent", uniforms.material.specular_exponent);
}

fn send_to_gpu_mesh(shader: ShaderHandle, mesh: &ObjMesh) -> (GLuint, GLuint, GLuint, GLuint) {
    let v_pos_loc = shader.get_attrib_location("aPos");
    let v_tex_loc = shader.get_attrib_location("aTexCoords");
    let v_norm_loc = shader.get_attrib_location("aNormal");

    let mut v_pos_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut v_pos_vbo);
    }
    debug_assert!(v_pos_vbo > 0);
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, v_pos_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mesh.points.len_bytes() as GLsizeiptr,
            mesh.points.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW
        );
    }

    let mut v_tex_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut v_tex_vbo);
    }
    debug_assert!(v_tex_vbo > 0);
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, v_tex_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mesh.tex_coords.len_bytes() as GLsizeiptr,
            mesh.tex_coords.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW
        )
    }

    let mut v_norm_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut v_norm_vbo);
    }
    debug_assert!(v_norm_vbo > 0);
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, v_norm_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mesh.normals.len_bytes() as GLsizeiptr,
            mesh.normals.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW
        );
    }

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, v_pos_vbo);
        gl::VertexAttribPointer(v_pos_loc, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::BindBuffer(gl::ARRAY_BUFFER, v_tex_vbo);
        gl::VertexAttribPointer(v_tex_loc, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::BindBuffer(gl::ARRAY_BUFFER, v_norm_vbo);
        gl::VertexAttribPointer(v_norm_loc, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(v_pos_loc);
        gl::EnableVertexAttribArray(v_tex_loc);
        gl::EnableVertexAttribArray(v_norm_loc);
    }
    debug_assert!(vao > 0);

    (vao, v_pos_vbo, v_tex_vbo, v_norm_vbo)
}

fn send_to_gpu_light_mesh(shader: ShaderHandle, mesh: &ObjMesh) -> (GLuint, GLuint) {
    let v_pos_loc = shader.get_attrib_location("v_pos");

    let mut v_pos_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut v_pos_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, v_pos_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (3 * mem::size_of::<GLfloat>() * mesh.points.len()) as GLsizeiptr,
            mesh.points.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW
        );
    }
    debug_assert!(v_pos_vbo > 0);

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, v_pos_vbo);
        gl::VertexAttribPointer(v_pos_loc, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(v_pos_loc);
    }
    debug_assert!(vao > 0);

    (vao, v_pos_vbo)
}

fn create_mesh_shader_source() -> ShaderSource<'static, 'static, 'static> {
    let vertex_name = "multiple_lights.vert.glsl";
    let vertex_source = include_str!("../shaders/multiple_lights.vert.glsl");
    let fragment_name = "multiple_lights.frag.glsl";
    let fragment_source = include_str!("../shaders/multiple_lights.frag.glsl");
    
    ShaderSourceBuilder::new(
        vertex_name,
        vertex_source,
        fragment_name,
        fragment_source)
    .build()
}

fn create_cube_light_shader_source() -> ShaderSource<'static, 'static, 'static> {
    let vertex_name = "lighting_cube.vert.glsl";
    let vertex_source = include_str!("../shaders/lighting_cube.vert.glsl");
    let fragment_name = "lighting_cube.frag.glsl";
    let fragment_source = include_str!("../shaders/lighting_cube.frag.glsl");

    ShaderSourceBuilder::new(
        vertex_name,
        vertex_source,
        fragment_name,
        fragment_source)
    .build()
}

fn send_to_gpu_shaders(_context: &mut OpenGLContext, source: &ShaderSource) -> ShaderHandle {
    backend::compile(source).unwrap()
}

/// Initialize the logger.
fn init_logger(log_file: &str) {
    file_logger::init(log_file).expect("Failed to initialize logger.");
}

/// Create and OpenGL context.
fn init_gl(width: u32, height: u32) -> backend::OpenGLContext {
    let context = match backend::start_opengl(width, height) {
        Ok(val) => val,
        Err(e) => {
            panic!("Failed to Initialize OpenGL context. Got error: {}", e);
        }
    };

    context
}

/// The GLFW frame buffer size callback function. This is normally set using 
/// the GLFW `glfwSetFramebufferSizeCallback` function, but instead we explicitly
/// handle window resizing in our state updates on the application side. Run this function 
/// whenever the size of the viewport changes.
fn framebuffer_size_callback(context: &mut OpenGLContext, width: u32, height: u32) {
    context.width = width;
    context.height = height;
    unsafe {
        gl::Viewport(0, 0, width as i32, height as i32);
    }
}

fn process_input(context: &mut OpenGLContext) -> CameraMovement {
    match context.window.get_key(Key::Escape) {
        Action::Press | Action::Repeat => {
            context.window.set_should_close(true);
        }
        _ => {}
    }

    let mut movement = CameraMovement::new();
    match context.window.get_key(Key::A) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveLeft;
        }
        _ => {}
        }
    match context.window.get_key(Key::D) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveRight;
        }
        _ => {}
    }
    match context.window.get_key(Key::Q) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveUp;
        }
        _ => {}
    }
    match context.window.get_key(Key::E) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveDown;
        }
        _ => {}
    }
    match context.window.get_key(Key::W) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveForward;
        }
        _ => {}
    }
    match context.window.get_key(Key::S) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveBackward;
        }
        _ => {}
    }
    match context.window.get_key(Key::Left) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::YawLeft;
        }
        _ => {}
    }
    match context.window.get_key(Key::Right) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::YawRight;
        }
        _ => {}
    }
    match context.window.get_key(Key::Up) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::PitchUp;
        }
        _ => {}
    }
    match context.window.get_key(Key::Down) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::PitchDown;
        }
        _ => {}
    }
    match context.window.get_key(Key::Z) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::RollCounterClockwise;
        }
        _ => {}
    }
    match context.window.get_key(Key::C) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::RollClockwise;
        }
        _ => {}
    }

    movement
}

fn main() {
    let mesh = create_box_mesh();
    let box_positions = create_box_positions();
    let box_model_matrices = create_box_model_matrices(&box_positions);
    let light_mesh = create_box_mesh();
    init_logger("opengl_demo.log");
    info!("BEGIN LOG");
    let mut camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
    let cube_lights= create_cube_lights();
    let mut spotlight = create_spotlight(&camera);
    let dir_light = create_directional_light();
    let material_diffuse_index = 0;
    let material_specular_index = 1;
    let material_emission_index = 2;
    let material = material::sgi_material_table()["chrome"];
    let material_uniforms = MaterialUniforms::new( 
        material_diffuse_index, 
        material_specular_index,
        material_emission_index,
        &material,
    );
    let lighting_map = create_lighting_map();
    let mut context = init_gl(SCREEN_WIDTH, SCREEN_HEIGHT);

    // The model matrix for the cube light shader and the flashlight shader.
    let mesh_model_mat = Matrix4::identity();

    // Load the lighting maps data for the flashlight and cubelight shaders.
    let (
        diffuse_tex, 
        specular_tex, 
        emission_tex) = send_to_gpu_textures_material(&lighting_map);

    //  Load the model data for the cube light shader..
    let mesh_shader_source = create_mesh_shader_source();
    let mesh_shader = send_to_gpu_shaders(&mut context, &mesh_shader_source);
    let (
        mesh_vao, 
        _mesh_v_pos_vbo,
        _mesh_v_tex_vbo,
        _mesh_v_norm_vbo) = send_to_gpu_mesh(mesh_shader, &mesh);
    send_to_gpu_uniforms_cube_light_mesh(mesh_shader, &mesh_model_mat);
    send_to_gpu_uniforms_camera(mesh_shader, &camera);
    send_to_gpu_uniforms_material(mesh_shader, material_uniforms);
    send_to_gpu_uniforms_point_lights(mesh_shader, &cube_lights);
    send_to_gpu_uniforms_spotlight(mesh_shader, &spotlight);
    send_to_gpu_uniforms_dir_light(mesh_shader, &dir_light);


    // Load the lighting cube model.
    let light_shader_source = create_cube_light_shader_source();
    let light_shader = send_to_gpu_shaders(&mut context, &light_shader_source);
    let (
        light_vao,
        _light_v_pos_vbo) = send_to_gpu_light_mesh(light_shader, &light_mesh);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::ClearBufferfv(gl::COLOR, 0, &CLEAR_COLOR[0] as *const GLfloat);
        gl::ClearBufferfv(gl::DEPTH, 0, &CLEAR_DEPTH[0] as *const GLfloat);
        gl::Viewport(0, 0, context.width as GLint, context.height as GLint);
    }

    while !context.window.should_close() {
        let elapsed_seconds = context.update_timers();
        context.update_fps_counter();
        context.glfw.poll_events();
        let (width, height) = context.window.get_framebuffer_size();
        if (width != context.width as i32) && (height != context.height as i32) {
            camera.update_viewport(width as usize, height as usize);
            framebuffer_size_callback(&mut context, width as u32, height as u32);
        }

        let delta_movement = process_input(&mut context);
        camera.update_movement(delta_movement, elapsed_seconds as f32);
        spotlight.update(&camera.position(), &camera.forward_axis());

        send_to_gpu_uniforms_camera(mesh_shader, &camera);
        send_to_gpu_uniforms_camera(light_shader, &camera);
        send_to_gpu_uniforms_point_lights(mesh_shader, &cube_lights);
        send_to_gpu_uniforms_spotlight(mesh_shader, &spotlight);
        send_to_gpu_uniforms_dir_light(mesh_shader, &dir_light);
    
        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, &CLEAR_COLOR[0] as *const GLfloat);
            gl::ClearBufferfv(gl::DEPTH, 0, &CLEAR_DEPTH[0] as *const GLfloat);
            gl::Viewport(0, 0, context.width as GLint, context.height as GLint);
            gl::UseProgram(mesh_shader.id);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, diffuse_tex);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, specular_tex);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, emission_tex);
            gl::BindVertexArray(mesh_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, mesh.len() as i32);
        }

        // Illuminate the boxes.
        for model_matrix in box_model_matrices.iter() {
            unsafe {
                send_to_gpu_uniforms_cube_light_mesh(mesh_shader, &model_matrix);
                gl::DrawArrays(gl::TRIANGLES, 0, mesh.len() as i32);
            }
        }

        let scale_matrix = Matrix4::from_affine_scale(0.2);

        // Render the cube lights.
        for cube_light in cube_lights.iter() {
            let light_model_mat = cube_light.model_matrix() * &scale_matrix;
            send_to_gpu_uniforms_cube_light_mesh(light_shader, &light_model_mat);
            unsafe {
                gl::UseProgram(light_shader.id);
                gl::BindVertexArray(light_vao);
                gl::DrawArrays(gl::TRIANGLES, 0, light_mesh.len() as i32);
            }
        }

        context.window.swap_buffers();
    }

    info!("END LOG");
}
