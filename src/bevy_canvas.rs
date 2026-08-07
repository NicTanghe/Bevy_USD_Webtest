use leptos::prelude::*;

use crate::model::{StagePrimInfo, ViewerCommand};

#[cfg(feature = "hydrate")]
use crate::model::{FrameStage, PendingStage, ViewerEvent};
#[cfg(feature = "hydrate")]
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
#[cfg(feature = "hydrate")]
use bevy::prelude::*;
#[cfg(feature = "hydrate")]
use leptos_bevy_canvas::prelude::*;
#[cfg(feature = "hydrate")]
use openusd::sdf::Value;
#[cfg(feature = "hydrate")]
use openusd::usd::Stage;
#[cfg(feature = "hydrate")]
use usd_bevy::live::{LiveStage, LiveStagePlugin, PrimEntities};
#[cfg(feature = "hydrate")]
use usd_bevy::route::DisplayPurposes;
#[cfg(feature = "hydrate")]
use usd_bevy::{UsdPlugin, UsdPrimRef};

const CANVAS_ID: &str = "usd_viewport";
#[cfg(feature = "hydrate")]
const MIN_CAMERA_RADIUS: f32 = 0.001;
#[cfg(feature = "hydrate")]
const MAX_CAMERA_RADIUS: f32 = 1_000_000.0;
#[cfg(feature = "hydrate")]
const CAMERA_FAR: f32 = 10_000_000.0;

#[cfg(feature = "hydrate")]
#[derive(Component)]
struct OrbitCamera {
    focus: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    auto_orbit: bool,
}

#[cfg(feature = "hydrate")]
#[derive(Component)]
struct SceneStyled;

#[cfg(feature = "hydrate")]
#[derive(Clone)]
pub(crate) struct ViewerBridge(LeptosMessageSender<ViewerCommand>);

#[cfg(not(feature = "hydrate"))]
#[derive(Clone)]
pub(crate) struct ViewerBridge;

#[cfg(feature = "hydrate")]
pub(crate) type CanvasReceiver = BevyMessageReceiver<ViewerCommand>;

#[cfg(feature = "hydrate")]
pub(crate) type StageEventReceiver = LeptosMessageReceiver<ViewerEvent>;

#[cfg(feature = "hydrate")]
pub(crate) type CanvasEventSender = BevyMessageSender<ViewerEvent>;

#[cfg(not(feature = "hydrate"))]
pub(crate) struct CanvasReceiver;

#[cfg(not(feature = "hydrate"))]
pub(crate) struct StageEventReceiver;

#[cfg(not(feature = "hydrate"))]
pub(crate) struct CanvasEventSender;

#[cfg(feature = "hydrate")]
pub(crate) fn viewer_bridge() -> (
    ViewerBridge,
    CanvasReceiver,
    StageEventReceiver,
    CanvasEventSender,
) {
    let (sender, receiver) = message_l2b::<ViewerCommand>();
    let (event_receiver, event_sender) = message_b2l::<ViewerEvent>();
    (ViewerBridge(sender), receiver, event_receiver, event_sender)
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn viewer_bridge() -> (
    ViewerBridge,
    CanvasReceiver,
    StageEventReceiver,
    CanvasEventSender,
) {
    (
        ViewerBridge,
        CanvasReceiver,
        StageEventReceiver,
        CanvasEventSender,
    )
}

#[cfg(feature = "hydrate")]
impl ViewerBridge {
    pub(crate) fn send(&self, command: ViewerCommand) {
        let _ = self.0.send(command);
    }
}

#[cfg(not(feature = "hydrate"))]
impl ViewerBridge {
    pub(crate) fn send(&self, command: ViewerCommand) {
        match command {
            ViewerCommand::ResetCamera => {}
            ViewerCommand::SetAutoOrbit(enabled) => {
                let _ = enabled;
            }
            ViewerCommand::FocusPrim(path) => drop(path),
        }
    }
}

#[cfg(feature = "hydrate")]
pub(crate) fn viewport_canvas(
    receiver: CanvasReceiver,
    event_sender: CanvasEventSender,
) -> impl IntoView {
    view! { <BevyCanvas init=move || init_bevy_app(receiver, event_sender) canvas_id=CANVAS_ID /> }
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn viewport_canvas(
    _receiver: CanvasReceiver,
    _event_sender: CanvasEventSender,
) -> impl IntoView {
    view! { <canvas id=CANVAS_ID></canvas> }
}

#[cfg(feature = "hydrate")]
pub(crate) fn install_stage_feedback(
    receiver: StageEventReceiver,
    set_stage_name: WriteSignal<String>,
    set_stage_prims: WriteSignal<Vec<StagePrimInfo>>,
    set_selected: WriteSignal<StagePrimInfo>,
    set_mesh_count: WriteSignal<usize>,
    set_stage_status: WriteSignal<String>,
    set_missing_dependencies: WriteSignal<Vec<String>>,
) {
    Effect::new(move |_| {
        let Some(event) = receiver.get() else {
            return;
        };
        match event {
            ViewerEvent::StageLoaded {
                name,
                prims,
                mesh_count,
                warning,
                missing_dependencies,
            } => {
                if let Some(first) = prims
                    .iter()
                    .find(|prim| prim.kind == "Mesh")
                    .or_else(|| prims.first())
                    .cloned()
                {
                    set_selected.set(first);
                }
                set_stage_name.set(name);
                set_mesh_count.set(mesh_count);
                set_stage_status.set(warning.unwrap_or_else(|| "STAGE READY".to_string()));
                set_stage_prims.set(prims);
                set_missing_dependencies.set(missing_dependencies);
            }
            ViewerEvent::StageLoadFailed { name, error } => {
                set_stage_name.set(name);
                set_stage_status.set(format!("LOAD FAILED: {error}"));
                set_missing_dependencies.set(Vec::new());
            }
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn install_stage_feedback(
    _receiver: StageEventReceiver,
    _set_stage_name: WriteSignal<String>,
    _set_stage_prims: WriteSignal<Vec<StagePrimInfo>>,
    _set_selected: WriteSignal<StagePrimInfo>,
    _set_mesh_count: WriteSignal<usize>,
    _set_stage_status: WriteSignal<String>,
    _set_missing_dependencies: WriteSignal<Vec<String>>,
) {
}

#[cfg(feature = "hydrate")]
fn init_bevy_app(
    command_rx: BevyMessageReceiver<ViewerCommand>,
    event_sender: BevyMessageSender<ViewerEvent>,
) -> bevy::prelude::App {
    let mut app = bevy::prelude::App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "OpenUSD Web Viewport".into(),
                    canvas: Some(format!("#{CANVAS_ID}")),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins((UsdPlugin, LiveStagePlugin))
    .import_message_from_leptos(command_rx)
    .export_message_to_leptos(event_sender)
    .insert_resource(ClearColor(Color::srgb_u8(18, 20, 23)))
    .insert_resource(DisplayPurposes {
        render: true,
        proxy: true,
        guide: false,
    })
    .init_resource::<PendingStage>()
    .init_resource::<FrameStage>()
    .add_systems(Startup, setup_viewport)
    .add_systems(
        Update,
        (
            apply_scene_style,
            handle_viewer_commands,
            orbit_camera,
            draw_editor_grid,
        ),
    )
    .add_systems(PostUpdate, crate::usd_loader::apply_pending_stage)
    .add_systems(Last, frame_loaded_stage);

    let stage = build_showroom_stage();
    app.world_mut().insert_non_send(LiveStage::new(stage));
    app
}

#[cfg(feature = "hydrate")]
fn build_showroom_stage() -> Stage {
    let stage = Stage::builder()
        .in_memory("showroom.usda")
        .expect("in-memory OpenUSD stage");
    stage
        .define_prim("/World")
        .expect("world prim")
        .set_type_name("Xform")
        .expect("world type");

    let specs = [
        ("Anchor", "Cube", [-2.4, 0.8, 0.0], Some(("size", 1.6))),
        ("Core", "Sphere", [0.0, 1.0, 0.0], Some(("radius", 1.0))),
        ("Tower", "Cylinder", [2.4, 1.0, 0.0], Some(("radius", 0.72))),
        ("Link", "Capsule", [-1.3, 1.1, -2.2], Some(("radius", 0.52))),
        ("Marker", "Cone", [1.3, 1.0, -2.2], Some(("radius", 0.84))),
        ("Deck", "Cube", [0.0, 0.2, 2.2], Some(("size", 0.4))),
    ];

    for (name, kind, position, shaped_attr) in specs {
        let path = format!("/World/{name}");
        stage
            .define_prim(path.as_str())
            .expect("shape prim")
            .set_type_name(kind)
            .expect("shape type");
        stage
            .create_attribute(format!("{path}.xformOp:translate"), "double3")
            .expect("translate attribute")
            .set(Value::Vec3d(openusd::gf::Vec3d::from(position)))
            .expect("translate value");
        stage
            .create_attribute(format!("{path}.xformOpOrder"), "token[]")
            .expect("xform order")
            .set(Value::TokenVec(vec!["xformOp:translate".into()]))
            .expect("xform order value");
        if let Some((name, value)) = shaped_attr {
            stage
                .create_attribute(format!("{path}.{name}"), "double")
                .expect("shape attribute")
                .set(Value::Double(value))
                .expect("shape value");
        }
        if matches!(kind, "Cylinder" | "Capsule" | "Cone") {
            stage
                .create_attribute(format!("{path}.height"), "double")
                .expect("height attribute")
                .set(Value::Double(if kind == "Capsule" { 1.25 } else { 2.0 }))
                .expect("height value");
            stage
                .create_attribute(format!("{path}.axis"), "token")
                .expect("axis attribute")
                .set(Value::Token("Y".into()))
                .expect("axis value");
        }
    }
    stage
}

#[cfg(feature = "hydrate")]
fn setup_viewport(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            near: MIN_CAMERA_RADIUS,
            far: CAMERA_FAR,
            ..default()
        }),
        Transform::from_xyz(8.5, 6.2, 10.5).looking_at(Vec3::new(0.0, 0.7, 0.0), Vec3::Y),
        OrbitCamera {
            focus: Vec3::new(0.0, 0.7, 0.0),
            radius: 14.8,
            yaw: 0.68,
            pitch: -0.36,
            auto_orbit: false,
        },
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 13_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 9.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 850_000.0,
            color: Color::srgb_u8(255, 190, 142),
            range: 18.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-4.0, 4.8, 3.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(32.0, 32.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.18, 0.19, 0.20, 0.10),
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.02, 0.0),
    ));
}

#[cfg(feature = "hydrate")]
fn apply_scene_style(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<
        (Entity, &UsdPrimRef),
        (
            bevy::ecs::query::With<Mesh3d>,
            bevy::ecs::query::Without<SceneStyled>,
        ),
    >,
) {
    for (entity, prim) in &query {
        let color = match prim.path.as_str() {
            "/World/Anchor" => Color::srgb_u8(206, 88, 53),
            "/World/Core" => Color::srgb_u8(237, 159, 72),
            "/World/Tower" => Color::srgb_u8(193, 200, 202),
            "/World/Link" => Color::srgb_u8(93, 111, 121),
            "/World/Marker" => Color::srgb_u8(43, 48, 53),
            _ => Color::srgb_u8(112, 124, 130),
        };
        let material = materials.add(StandardMaterial {
            base_color: color,
            metallic: if prim.path.ends_with("Link") {
                0.72
            } else {
                0.22
            },
            perceptual_roughness: 0.5,
            ..default()
        });
        commands
            .entity(entity)
            .insert((MeshMaterial3d(material), SceneStyled));
    }
}

#[cfg(feature = "hydrate")]
fn handle_viewer_commands(
    mut messages: MessageReader<ViewerCommand>,
    prims: Res<PrimEntities>,
    transforms: Query<&GlobalTransform>,
    mut cameras: Query<&mut OrbitCamera>,
    mut pending_stage: ResMut<PendingStage>,
    mut frame_stage: ResMut<FrameStage>,
) {
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };
    for message in messages.read() {
        match message {
            ViewerCommand::ResetCamera => {
                frame_stage.0 = true;
            }
            ViewerCommand::SetAutoOrbit(enabled) => camera.auto_orbit = *enabled,
            ViewerCommand::FocusPrim(path) => {
                if let Some(entity) = prims.entity(path)
                    && let Ok(transform) = transforms.get(entity)
                {
                    camera.focus = transform.translation();
                }
            }
            ViewerCommand::LoadUsd { name, files } => {
                pending_stage.0 = Some((name.clone(), files.clone()));
            }
        }
    }
}

#[cfg(feature = "hydrate")]
fn frame_loaded_stage(
    mut request: ResMut<FrameStage>,
    meshes: Res<Assets<Mesh>>,
    geometry: Query<(&Mesh3d, &GlobalTransform), bevy::ecs::query::With<UsdPrimRef>>,
    mut cameras: Query<(&mut OrbitCamera, &Projection)>,
) {
    if !request.0 {
        return;
    }

    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let mut points_found = 0usize;
    for (mesh_handle, transform) in &geometry {
        let Some(mesh) = meshes.get(&mesh_handle.0) else {
            continue;
        };
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };
        for position in positions {
            let world_position = transform.transform_point(Vec3::from_array(*position));
            min = min.min(world_position);
            max = max.max(world_position);
            points_found += 1;
        }
    }
    if points_found == 0 {
        return;
    }

    let Ok((mut camera, projection)) = cameras.single_mut() else {
        return;
    };
    let size = max - min;
    camera.focus = (min + max) * 0.5;
    let half_fov = match projection {
        Projection::Perspective(projection) => {
            let vertical = projection.fov * 0.5;
            let horizontal = (vertical.tan() * projection.aspect_ratio).atan();
            vertical.min(horizontal)
        }
        _ => std::f32::consts::FRAC_PI_8,
    };
    let bounds_radius = size.length() * 0.5;
    camera.radius =
        (bounds_radius / half_fov.sin() * 1.12).clamp(MIN_CAMERA_RADIUS, MAX_CAMERA_RADIUS);
    request.0 = false;
}

#[cfg(feature = "hydrate")]
fn orbit_camera(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut frame_stage: ResMut<FrameStage>,
    mut cameras: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    let Ok((mut transform, mut orbit)) = cameras.single_mut() else {
        return;
    };
    let delta = motion
        .read()
        .fold(Vec2::ZERO, |sum, event| sum + event.delta);
    let view_modifier = keyboard.pressed(KeyCode::Space)
        || keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    let precision = if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        0.2
    } else {
        1.0
    };
    let mut navigating = false;

    if view_modifier && mouse.pressed(MouseButton::Left) {
        orbit.yaw -= delta.x * 0.005 * precision;
        orbit.pitch = (orbit.pitch - delta.y * 0.005 * precision).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.001,
            std::f32::consts::FRAC_PI_2 - 0.001,
        );
        navigating = true;
    } else if view_modifier && mouse.pressed(MouseButton::Middle) {
        let rotation = Quat::from_euler(EulerRot::YXZ, orbit.yaw, orbit.pitch, 0.0);
        let right = rotation * Vec3::X;
        let up = rotation * Vec3::Y;
        let scale = orbit.radius * 0.0015 * precision;
        orbit.focus += (-right * delta.x + up * delta.y) * scale;
        navigating = true;
    } else if view_modifier && mouse.pressed(MouseButton::Right) {
        orbit.radius = (orbit.radius * (delta.y * 0.01 * precision).exp())
            .clamp(MIN_CAMERA_RADIUS, MAX_CAMERA_RADIUS);
        navigating = true;
    }

    if view_modifier && keyboard.just_pressed(KeyCode::KeyH) {
        frame_stage.0 = true;
        navigating = true;
    }

    let scroll: f32 = wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 40.0,
        })
        .sum();
    if scroll != 0.0 {
        orbit.radius = (orbit.radius * (-scroll * 0.12 * precision).exp())
            .clamp(MIN_CAMERA_RADIUS, MAX_CAMERA_RADIUS);
        navigating = true;
    }

    if navigating {
        orbit.auto_orbit = false;
    } else if orbit.auto_orbit {
        orbit.yaw += time.delta_secs() * 0.12;
    }

    let rotation = Quat::from_euler(EulerRot::YXZ, orbit.yaw, orbit.pitch, 0.0);
    transform.translation = orbit.focus + rotation * Vec3::new(0.0, 0.0, orbit.radius);
    transform.look_at(orbit.focus, Vec3::Y);
}

#[cfg(feature = "hydrate")]
fn draw_editor_grid(mut gizmos: Gizmos) {
    let major = Color::srgba(0.42, 0.46, 0.48, 0.22);
    let minor = Color::srgba(0.33, 0.36, 0.38, 0.11);
    for i in -16..=16 {
        let p = i as f32;
        let color = if i % 4 == 0 { major } else { minor };
        gizmos.line(Vec3::new(p, 0.012, -16.0), Vec3::new(p, 0.012, 16.0), color);
        gizmos.line(Vec3::new(-16.0, 0.012, p), Vec3::new(16.0, 0.012, p), color);
    }
}
