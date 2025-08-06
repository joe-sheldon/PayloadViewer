mod constants;

use std::ops::{Add, Mul};
use bevy::prelude::*;
use bevy::color::Color;
use bevy::reflect::List;
use bevy::tasks::futures_lite::StreamExt;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_obj::ObjPlugin;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use rfd::FileDialog;
use crate::constants::{SKY_COLOR, WELD_CENTER_COLOR, WELD_CENTER_SIZE, WELD_FACE_CENTER_COLOR, WELD_FACE_CENTER_SIZE};
use crate::weld_digest::{read_payload_digest, Joint, PayloadDigest};

mod weld_digest;

#[derive(Resource, Default)]
struct Game {
    loa: String,
    loa_rev: String,
    summary_path: String,
    weld_digest: PayloadDigest,
    selected_joint: Option<Joint>,
    selected_component_uid: Option<String>,
    selection_cooldown: Timer,
    ui_show_joint_list: bool,
    ui_show_component_list: bool,
    ui_show_settings_window: bool,
    vis_option_active_color: [f32; 3],
    vis_option_inactive_color: [f32; 3],
    vis_option_weld_center_color: [f32; 3],
    vis_option_pipe_face_center_color: [f32; 3],

    vis_option_active_opacity: i32,
    vis_option_inactive_opacity: i32,
    vis_option_locator_sphere_opacity: i32,
}

#[derive(Default)]
struct Player {
    entity: Option<Entity>,
    loc: Vec3,
    forward: Vec3,
}

#[derive(Component)]
struct WeldCenter;

#[derive(Component)]
struct ModeledComponent;

#[derive(Component)]
struct FaceCenter;

fn main() {
    let dir_path = FileDialog::new()
        .set_title("Load Payload Joint Summary Directory")
        .set_directory(".") // Set starting directory
        .pick_folder(); // Open the file dialog
    let dp = dir_path.unwrap().to_str().unwrap().to_string();

    App::new()
        .init_resource::<Game>()
        .insert_resource(ClearColor(SKY_COLOR))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Set your custom asset path here
            file_path: dp.to_string(),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins((
            PanOrbitCameraPlugin,
            ObjPlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut game: ResMut<Game>
) {
    // Default Visibility Options
    game.vis_option_active_color = [0.2, 0.8, 0.2];
    game.vis_option_inactive_color = [0.2, 0.2, 0.2];
    game.vis_option_weld_center_color = [0.2, 0.2, 0.6];
    game.vis_option_pipe_face_center_color = [0.6, 0.2, 0.2];

    game.vis_option_active_opacity = 100;
    game.vis_option_inactive_opacity = 100;
    game.vis_option_locator_sphere_opacity = 100;

    let dp = "sample_data/sample_0/".to_string();
    game.summary_path = dp.clone();

    game.weld_digest = read_payload_digest(dp.clone()).unwrap();;
    println!("Loaded Payload Summary");

    // Components
    for comp in game.weld_digest.components.values().clone() {
        // Actual Model
        let obj_path = comp.clone().geom_path;
        if obj_path.is_some() {
            commands.spawn((
                Mesh3d(asset_server.load_with_settings(
                    obj_path.unwrap(),
                    |settings: &mut bevy_obj::ObjSettings| {
                        settings.force_compute_normals = true;
                        settings.prefer_flat_normals = true;
                    })
                ),
                MeshMaterial3d(materials.add(Color::srgba(
                    game.vis_option_inactive_color[0],
                    game.vis_option_inactive_color[1],
                    game.vis_option_inactive_color[2],
                    (game.vis_option_inactive_opacity as f32 / 100.0)
                ))),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                ModeledComponent
            ));
        }

    // Joint Stuff
    for (joint_uuid, joint) in game.weld_digest.joints.clone() {
        println!("Spawning Joint {:?}: {:?}", joint_uuid, joint.joint_number);

        if(joint.center.is_some()) {
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(WELD_CENTER_SIZE))),
                MeshMaterial3d(materials.add(Color::srgba(
                    game.vis_option_weld_center_color[0],
                    game.vis_option_weld_center_color[1],
                    game.vis_option_weld_center_color[2],
                    (game.vis_option_locator_sphere_opacity as f32 / 100.0)
                ))),
                Transform::from_translation(joint.center.unwrap()),
                WeldCenter
            ));
        }


            // Weld Faces
            // for face in member.faces {
            //     commands.spawn((
            //         Mesh3d(meshes.add(Sphere::new(WELD_FACE_CENTER_SIZE))),
            //         MeshMaterial3d(materials.add(Color::srgba(
            //             game.vis_option_pipe_face_center_color[0],
            //             game.vis_option_pipe_face_center_color[1],
            //             game.vis_option_pipe_face_center_color[2],
            //             (game.vis_option_locator_sphere_opacity as f32 / 100.0)
            //         ))),
            //         Transform::from_translation(face),
            //         FaceCenter
            //     ));
            // }
        }
    }


    // Lighting
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(-50.0, 0.0, 0.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(0.0, 50.0, 0.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(0.0, -50.0, 0.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Pan-Orbit camera around active joint center
    commands.spawn((
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)).look_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));
}

fn ui_example_system(
    mut game: ResMut<Game>,
    mut camera_transforms: ParamSet<(Query<&mut Transform, With<Camera3d>>, Query<&Transform>)>,
    mut contexts: EguiContexts
) -> Result {
    let weld_digest = game.weld_digest.clone();

    let ui_title = format!(
        "Joint Summary for {}/{}",
        weld_digest.name,
        weld_digest.rev
    );

    egui::Window::new(ui_title).show(contexts.ctx_mut()?, |ui| {
        ui.checkbox(&mut game.ui_show_joint_list, "Show Joint List");
        ui.checkbox(&mut game.ui_show_component_list, "Show Component List");
        ui.checkbox(&mut game.ui_show_settings_window, "Show settings");
        ui.separator();
        if ui.button("Clear Selections").clicked() {
            game.selected_component_uid = None;
            game.selected_joint = None;
        }
    });

    if (game.ui_show_joint_list) {
        egui::Window::new("Joints List").show(contexts.ctx_mut()?, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if (game.selected_joint.is_some()) {
                    let active_joint = game.selected_joint.clone().unwrap();
                    ui.heading(format!("Selected Joint: {:?}", active_joint.joint_number));
                    ui.label(format!("Design: {:?}", active_joint.joint_design));
                    for member_uid in active_joint.members {
                        let comp = game.weld_digest.components.get(&member_uid).unwrap();
                        ui.label(format!("Member: {:?}", comp.part_number));
                    }
                    if (active_joint.center.is_some()) {
                        if ui.button("Fly To Selected Joint").clicked() {
                            println!("Fly to joint: {:?}: {:?}", active_joint.joint_number, active_joint.center);

                            let new_center = active_joint.center.unwrap();
                            for mut transform in camera_transforms.p0().iter_mut() {
                                *transform = transform.looking_at(new_center, Vec3::Y).with_translation(
                                    Vec3::new(
                                        new_center.x + 50.0,
                                        new_center.y,
                                        new_center.z
                                    )
                                );
                            }
                        };
                    }

                    ui.separator();
                }

                ui.heading("All Joints:");
                if (weld_digest.joints.len() == 0) {
                    ui.label("Payload has 0 Joints");
                } else {
                    let all_joints = weld_digest.joints.clone();
                    for (uuid, joint) in all_joints {
                        ui.horizontal(|hori| {
                            hori.label(joint.joint_number.clone());
                            if hori.button("Select").clicked() {
                                game.selected_joint = Some(joint);
                            };
                        });
                    }
                }

            });
        });
    }

    if (game.ui_show_component_list) {
        egui::Window::new("Components List").show(contexts.ctx_mut()?, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if (game.selected_component_uid.is_some()) {
                    let active_component = game.weld_digest.components
                        .get(&game.selected_component_uid.clone().unwrap()).unwrap().clone();

                    ui.heading(format!("Selected Component: {:?}", active_component.part_number));
                    ui.label(format!("Design ID: {:?}", active_component.design_id));
                    ui.label(format!("Description: {:?}", active_component.description));

                    if (active_component.loc.is_some()) {
                        if ui.button("Fly To Selected Component").clicked() {
                            println!("Fly to component: {:?}: {:?}", active_component.part_number, active_component.loc);
                            let new_center = active_component.loc.unwrap();
                            for mut transform in camera_transforms.p0().iter_mut() {
                                *transform = transform.looking_at(new_center, Vec3::Y).with_translation(
                                    Vec3::new(
                                        new_center.x + 50.0,
                                        new_center.y,
                                        new_center.z
                                    )
                                );
                            }
                        };
                    }
                    ui.separator();
                }

                ui.heading("All Components:");
                if (weld_digest.components.len() == 0) {
                    ui.label("Payload has 0 Components");
                } else {
                    let all_components = weld_digest.components.clone();
                    for (uid, comp) in all_components.clone() {
                        ui.horizontal(|hori| {
                            hori.label(comp.part_number.clone());
                            if hori.button("Select").clicked() {
                                game.selected_component_uid = Some(uid);
                            };
                        });
                    }
                }
            })
        });
    }

    if (game.ui_show_settings_window) {
        egui::Window::new("Settings").show(contexts.ctx_mut()?, |ui| {
            ui.heading("Colors");
            ui.horizontal(|hori| {
                hori.color_edit_button_rgb(&mut game.vis_option_weld_center_color);
                hori.color_edit_button_rgb(&mut game.vis_option_pipe_face_center_color);
                hori.label("Weld / Pipe Face Centers");
            });
            ui.horizontal(|hori| {
                hori.color_edit_button_rgb(&mut game.vis_option_active_color);
                hori.label("Active Component");
            });
            ui.horizontal(|hori| {
                hori.color_edit_button_rgb(&mut game.vis_option_inactive_color);
                hori.label("Inactive Components");
            });
            ui.separator();
            ui.heading("Opacity");
            ui.add(egui::Slider::new(&mut game.vis_option_active_opacity, 0..=100)
                .text("Active Component"));
            ui.add(egui::Slider::new(&mut game.vis_option_inactive_opacity, 0..=100)
                .text("Inactive Component"));
            ui.add(egui::Slider::new(&mut game.vis_option_locator_sphere_opacity, 0..=100)
                .text("Locator Spheres"));
        });
    }

    Ok(())
}
