#![feature(const_collections_with_hasher)]
#![feature(allocator_api)]
#![feature(downcast_unchecked)]

use std::f32::consts::{FRAC_PI_2 as FRAC_TAU_4, TAU};
use std::sync::Arc;

use citro3d::buffer::Primitive;
use citro3d::macros::*;
use citro3d::math::{AspectRatio, ClipPlanes, Projection, StereoDisplacement};
use citro3d::render::{ClearFlags, DepthFormat, Target};
use citro3d::shader::{Library, Program};
use citro3d::texture::TextureFilterParam;
use citro3d::uniform::Index;
use citro3d::Instance;
use ctru::prelude::*;
use ctru::services::gfx::{RawFrameBuffer, Screen, TopScreen3D};

use glam::{Mat4, Quat, Vec2, Vec3};

use include_texture_macro::include_texture;
use vert_attr::VertAttrBuilder;

mod asset_server;
mod model;

use asset_server::add_asset;
use model::colour::Colour;
use model::material::Material;
use model::shape::Shape;
use model::texture::Texture;
use model::Model;

const DEADZONE: f32 = 0.01;
const CIRCLE_DEADZONE: f32 = 15.0;

const SHADER: &[u8] = include_shader!("../shader.pica");

const BOWSER: &[u8] = include_texture!("../bowser.png");
const PEACH: &[u8] = include_texture!("../diffuse.png");

const NORMAL: &[u8] = include_texture!("../normal.png");

pub struct Uniforms {
    pub model_matrix: Index,
    pub camera_matrix: Index,
    pub projection_matrix: Index,
}

#[derive(VertAttrBuilder, Clone, Debug)]
#[repr(C)]
struct Vert {
    pos: Vec3,
    tex: Vec2,
    norm: Vec3,
    tan: Vec3,
}

fn main() {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx = Gfx::new().unwrap();
    let _console = Console::new(gfx.bottom_screen.borrow_mut());

    //let mut soc = Soc::new().unwrap();
    //soc.redirect_to_3dslink(true, true).unwrap();

    let mut gpu = Instance::new().unwrap();

    let top_screen = TopScreen3D::from(&gfx.top_screen);

    let (mut top_screen_left, mut top_screen_right) = top_screen.split_mut();

    let RawFrameBuffer { width, height, .. } = top_screen_left.raw_framebuffer();
    let mut top_left_target = Target::new(
        width,
        height,
        top_screen_left,
        Some(DepthFormat::Depth24Stencil8),
    )
    .expect("failed to create left render target");

    let RawFrameBuffer { width, height, .. } = top_screen_right.raw_framebuffer();
    let mut top_right_target = Target::new(
        width,
        height,
        top_screen_right,
        Some(DepthFormat::Depth24Stencil8),
    )
    .expect("failed to create right render target");

    let shader_lib = Library::from_bytes(SHADER).expect("failed to load shader");
    let vert_shader = shader_lib.get(0).unwrap();
    let vert_prog = Arc::pin(Program::new(vert_shader).unwrap());

    let model_uniform = vert_prog.get_uniform("modelMtx").unwrap();
    let cam_uniform = vert_prog.get_uniform("camMtx").unwrap();
    let proj_uniform = vert_prog.get_uniform("projMtx").unwrap();

    let uniforms = Uniforms {
        model_matrix: model_uniform,
        camera_matrix: cam_uniform,
        projection_matrix: proj_uniform,
    };

    gpu.bind_program(vert_prog);

    let mut light_env = gpu.light_env_mut();

    let light_index = light_env.as_mut().create_light().unwrap();
    let mut light = light_env.as_mut().light_mut(light_index).unwrap();
    let light_pos = Vec3::new(0.0, 0.0, -0.5);
    light.as_mut().set_color(1.0, 1.0, 1.0);

    let mut cam_pos = Vec3::new(0.0, 0.0, 0.0);

    // yaw, pitch, roll
    let mut cam_rot = Vec3::new(0.0, 0.0, 0.0);

    let peach = Texture::new(
        128,
        128,
        PEACH.to_vec(),
        TextureFilterParam::Linear,
        TextureFilterParam::Nearest,
    );
    let bowser = Texture::new(
        64,
        64,
        BOWSER.to_vec(),
        TextureFilterParam::Linear,
        TextureFilterParam::Nearest,
    );

    let gpu_peach = (&peach).into();
    let gpu_bowser = (&bowser).into();

    let peach_key = add_asset("peach_tex", gpu_peach);
    let bowser_key = add_asset("bowser_tex", gpu_bowser);

    let normal = Texture::new(
        128,
        128,
        NORMAL.to_vec(),
        TextureFilterParam::Linear,
        TextureFilterParam::Nearest,
    );
    let gpu_normal = (&normal).into();
    let normal_key = add_asset("normal_tex", gpu_normal);

    let specular = Colour::new(255, 255, 255, 255);
    let specular = add_asset("specular", specular);

    let red = Colour::new(255, 0, 0, 255);
    let red = add_asset("red", red);

    let blue = Colour::new(0, 0, 255, 255);
    let blue = add_asset("blue", blue);

    let ambient = Colour::new(127, 127, 127, 255);
    let ambient = add_asset("ambient", ambient);

    let diffuse_red = Colour::new(102, 0, 0, 255);
    let diffuse_red = add_asset("diffuse_red", diffuse_red);

    let diffuse_blue = Colour::new(0, 0, 102, 255);
    let diffuse_blue = add_asset("diffuse_blue", diffuse_blue);

    let peach_mat = Material::new(
        Some(peach_key),
        Some(normal_key),
        Some(ambient),
        Some(diffuse_blue),
        Some(specular),
        None,
        None,
        Some(100.0),
    );
    let bowser_mat = Material::new(
        Some(bowser_key),
        Some(normal_key),
        Some(ambient),
        Some(diffuse_red),
        Some(specular),
        None,
        Some(red),
        Some(100.0),
    );

    let peach_mat_key = add_asset("peach_mat", peach_mat);
    let bowser_mat_key = add_asset("bowser_mat", bowser_mat);

    let square_front = Shape::new(
        peach_mat_key,
        Primitive::TriangleFan,
        vec![
            Vert {
                pos: Vec3::new(-0.5, 0.5, -0.5),
                tex: Vec2::new(0.0, 1.0),
                norm: Vec3::new(0.0, 0.0, 1.0),
                tan: Vec3::new(1.0, 0.0, 0.0),
            },
            Vert {
                pos: Vec3::new(-0.5, -0.5, -0.5),
                tex: Vec2::new(0.0, 0.0),
                norm: Vec3::new(0.0, 0.0, 1.0),
                tan: Vec3::new(1.0, 0.0, 0.0),
            },
            Vert {
                pos: Vec3::new(0.5, -0.5, -0.5),
                tex: Vec2::new(1.0, 0.0),
                norm: Vec3::new(0.0, 0.0, 1.0),
                tan: Vec3::new(1.0, 0.0, 0.0),
            },
            Vert {
                pos: Vec3::new(0.5, 0.5, -0.5),
                tex: Vec2::new(1.0, 1.0),
                norm: Vec3::new(0.0, 0.0, 1.0),
                tan: Vec3::new(1.0, 0.0, 0.0),
            },
        ],
    );
    let square_back = Shape::new(
        bowser_mat_key,
        Primitive::TriangleFan,
        vec![
            Vert {
                pos: Vec3::new(0.5, 0.5, -0.5),
                tex: Vec2::new(1.0, 1.0),
                norm: Vec3::new(0.0, 0.0, 1.0),
                tan: Vec3::new(1.0, 0.0, 0.0),
            },
            Vert {
                pos: Vec3::new(0.5, -0.5, -0.5),
                tex: Vec2::new(1.0, 0.0),
                norm: Vec3::new(0.0, 0.0, 1.0),
                tan: Vec3::new(1.0, 0.0, 0.0),
            },
            Vert {
                pos: Vec3::new(-0.5, -0.5, -0.5),
                tex: Vec2::new(0.0, 0.0),
                norm: Vec3::new(0.0, 0.0, 1.0),
                tan: Vec3::new(1.0, 0.0, 0.0),
            },
            Vert {
                pos: Vec3::new(-0.5, 0.5, -0.5),
                tex: Vec2::new(0.0, 1.0),
                norm: Vec3::new(0.0, 0.0, 1.0),
                tan: Vec3::new(1.0, 0.0, 0.0),
            },
        ],
    );

    let front_key = add_asset("front_square", square_front);
    let back_key = add_asset("back_square", square_back);

    let mut mdl = Model::new(
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 0.0, 0.0),
        vec![front_key, back_key],
    );

    let mut last_touch = (0, 0);
    let mut last_angle = (0.0, 0.0);

    while apt.main_loop() {
        gfx.wait_for_vblank();

        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        let (x, y) = hid.circlepad_position();
        let (x, y) = (x as f32, y as f32);
        let x_move = if x.abs() > CIRCLE_DEADZONE {
            x / 1000.0
        } else {
            0.0
        };
        let y_move = if y.abs() > CIRCLE_DEADZONE {
            y / 1000.0
        } else {
            0.0
        };
        let cos = cam_rot.x.cos();
        let sin = -cam_rot.x.sin();
        cam_pos.x += -cos * x_move + sin * y_move;
        cam_pos.z += cos * y_move + sin * x_move;

        if hid.keys_held().contains(KeyPad::X) {
            cam_pos.y -= 0.01;
        }
        if hid.keys_held().contains(KeyPad::Y) {
            cam_pos.y += 0.01;
        }

        if hid.keys_down().contains(KeyPad::TOUCH) {
            last_touch = hid.touch_position();
            last_angle = (cam_rot.x, cam_rot.y);
        }

        if hid.keys_held().contains(KeyPad::TOUCH) {
            let (tx, ty) = hid.touch_position();
            let (sx, sy) = last_touch;
            let (tx, ty) = (tx as i16, ty as i16);
            let (sx, sy) = (sx as i16, sy as i16);
            let (dx, dy) = (tx - sx, -(ty - sy));
            let (ax, ay) = (dx as f32 / 320.0, dy as f32 / 240.0);
            cam_rot.x = (last_angle.0 + ax * TAU / 2.0).rem_euclid(TAU);
            cam_rot.y = (last_angle.1 + ay * TAU / 2.0).clamp(-FRAC_TAU_4, FRAC_TAU_4);
        }

        gpu.render_frame_with(|inst| {
            let scale = Vec3::new(1.0, 1.0, 1.0);

            let rotation = Quat::from_axis_angle(Vec3::Y, -cam_rot.x)
                * Quat::from_axis_angle(Vec3::X, cam_rot.y);

            let camera_matrix =
                Mat4::from_scale_rotation_translation(scale, -rotation, -cam_pos).inverse();

            inst.bind_vertex_uniform(uniforms.camera_matrix, camera_matrix);

            let light_pos = camera_matrix.transform_point3(light_pos);

            inst.light_env_mut()
                .light_mut(light_index)
                .unwrap()
                .set_position(light_pos.into());

            let mut render_to = |target: &mut Target, projection| {
                target.clear(ClearFlags::ALL, 0xFF00FFFF, 0);
                inst.select_render_target(target).unwrap();

                inst.bind_vertex_uniform(uniforms.projection_matrix, projection);

                mdl.draw(inst, &uniforms);
            };

            let Projections {
                left_eye,
                right_eye,
                ..
            } = calculate_projections();

            render_to(&mut top_left_target, left_eye);
            render_to(&mut top_right_target, right_eye);
        })
    }
}

#[derive(Debug)]
struct Projections {
    left_eye: Mat4,
    right_eye: Mat4,
    center: Mat4,
}

fn calculate_projections() -> Projections {
    // TODO: it would be cool to allow playing around with these parameters on
    // the fly with D-pad, etc.
    let slider_val = ctru::os::current_3d_slider_state();
    let interocular_distance = slider_val / 2.0;

    let vertical_fov = 40.0_f32.to_radians();
    let screen_depth = 2.0;

    let clip_planes = ClipPlanes {
        near: 0.01,
        far: 100.0,
    };

    let (left, right) = StereoDisplacement::new(interocular_distance, screen_depth);

    let (left_eye, right_eye) =
        Projection::perspective(vertical_fov, AspectRatio::TopScreen, clip_planes)
            .stereo_matrices(left, right);

    let center: citro3d::math::Matrix4 =
        Projection::perspective(vertical_fov, AspectRatio::BottomScreen, clip_planes).into();

    Projections {
        left_eye: left_eye.into(),
        right_eye: right_eye.into(),
        center: center.into(),
    }
}
