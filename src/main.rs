use leptos::prelude::*;
use leptos_meta::{Meta, MetaTags, Stylesheet, Title, provide_meta_context};

#[cfg(feature = "hydrate")]
use bevy::input::mouse::{MouseMotion, MouseWheel};
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

#[derive(Clone)]
#[cfg_attr(feature = "hydrate", derive(Message))]
enum ViewerCommand {
    ResetCamera,
    SetAutoOrbit(bool),
    FocusPrim(String),
    LoadUsd {
        name: String,
        bytes: std::sync::Arc<Vec<u8>>,
    },
}

#[derive(Clone, PartialEq)]
struct StagePrimInfo {
    path: String,
    name: String,
    kind: String,
    position: String,
    color: String,
}

#[derive(Clone)]
#[cfg_attr(feature = "hydrate", derive(Message))]
enum ViewerEvent {
    StageLoaded {
        name: String,
        prims: Vec<StagePrimInfo>,
        mesh_count: usize,
        warning: Option<String>,
    },
    StageLoadFailed {
        name: String,
        error: String,
    },
}

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
#[derive(Resource, Default)]
struct PendingStage(Option<(String, std::sync::Arc<Vec<u8>>)>);

#[cfg(feature = "hydrate")]
#[derive(Resource, Default)]
struct FrameStage(bool);

#[derive(Clone, Copy)]
struct PrimInfo {
    path: &'static str,
    name: &'static str,
    kind: &'static str,
    position: &'static str,
    color: &'static str,
}

impl From<PrimInfo> for StagePrimInfo {
    fn from(prim: PrimInfo) -> Self {
        Self {
            path: prim.path.to_string(),
            name: prim.name.to_string(),
            kind: prim.kind.to_string(),
            position: prim.position.to_string(),
            color: prim.color.to_string(),
        }
    }
}

const PRIMS: [PrimInfo; 6] = [
    PrimInfo {
        path: "/World/Anchor",
        name: "Anchor",
        kind: "Cube",
        position: "−2.4, 0.8, 0.0",
        color: "Oxide",
    },
    PrimInfo {
        path: "/World/Core",
        name: "Core",
        kind: "Sphere",
        position: "0.0, 1.0, 0.0",
        color: "Signal",
    },
    PrimInfo {
        path: "/World/Tower",
        name: "Tower",
        kind: "Cylinder",
        position: "2.4, 1.0, 0.0",
        color: "Ceramic",
    },
    PrimInfo {
        path: "/World/Link",
        name: "Link",
        kind: "Capsule",
        position: "−1.3, 1.1, −2.2",
        color: "Alloy",
    },
    PrimInfo {
        path: "/World/Marker",
        name: "Marker",
        kind: "Cone",
        position: "1.3, 1.0, −2.2",
        color: "Carbon",
    },
    PrimInfo {
        path: "/World/Deck",
        name: "Deck",
        kind: "Cube",
        position: "0.0, 0.2, 2.2",
        color: "Slate",
    },
];

#[cfg(feature = "hydrate")]
#[derive(Clone)]
struct ViewerBridge(LeptosMessageSender<ViewerCommand>);

#[cfg(not(feature = "hydrate"))]
#[derive(Clone)]
struct ViewerBridge;

#[cfg(feature = "hydrate")]
type CanvasReceiver = BevyMessageReceiver<ViewerCommand>;

#[cfg(feature = "hydrate")]
type StageEventReceiver = LeptosMessageReceiver<ViewerEvent>;

#[cfg(feature = "hydrate")]
type CanvasEventSender = BevyMessageSender<ViewerEvent>;

#[cfg(not(feature = "hydrate"))]
struct CanvasReceiver;

#[cfg(not(feature = "hydrate"))]
struct StageEventReceiver;

#[cfg(not(feature = "hydrate"))]
struct CanvasEventSender;

#[cfg(feature = "hydrate")]
fn viewer_bridge() -> (
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
fn viewer_bridge() -> (
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
    fn send(&self, command: ViewerCommand) {
        let _ = self.0.send(command);
    }
}

#[cfg(not(feature = "hydrate"))]
impl ViewerBridge {
    fn send(&self, _command: ViewerCommand) {}
}

#[cfg(feature = "hydrate")]
fn viewport_canvas(receiver: CanvasReceiver, event_sender: CanvasEventSender) -> impl IntoView {
    view! { <BevyCanvas init=move || init_bevy_app(receiver, event_sender) canvas_id=CANVAS_ID /> }
}

#[cfg(not(feature = "hydrate"))]
fn viewport_canvas(_receiver: CanvasReceiver, _event_sender: CanvasEventSender) -> impl IntoView {
    view! { <canvas id=CANVAS_ID></canvas> }
}

#[cfg(feature = "hydrate")]
struct BrowserFileResolver {
    name: String,
    bytes: std::sync::Arc<Vec<u8>>,
}

#[cfg(feature = "hydrate")]
impl BrowserFileResolver {
    fn matches(&self, path: &str) -> bool {
        std::path::Path::new(path).file_name() == std::path::Path::new(&self.name).file_name()
    }
}

#[cfg(feature = "hydrate")]
impl openusd::ar::Resolver for BrowserFileResolver {
    fn create_identifier(
        &self,
        asset_path: &str,
        _anchor: Option<&openusd::ar::ResolvedPath>,
    ) -> String {
        if self.matches(asset_path) {
            self.name.clone()
        } else {
            asset_path.replace('\\', "/")
        }
    }

    fn resolve(&self, asset_path: &str) -> Option<openusd::ar::ResolvedPath> {
        self.matches(asset_path)
            .then(|| openusd::ar::ResolvedPath::new(&self.name))
    }

    fn resolve_for_new_asset(&self, asset_path: &str) -> Option<openusd::ar::ResolvedPath> {
        self.resolve(asset_path)
    }

    fn open_asset(
        &self,
        resolved_path: &openusd::ar::ResolvedPath,
    ) -> std::io::Result<Box<dyn openusd::ar::Asset>> {
        if !self.matches(&resolved_path.to_string()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("asset is not available in the browser: {resolved_path}"),
            ));
        }
        Ok(Box::new(std::io::Cursor::new((*self.bytes).clone())))
    }

    fn identity(&self) -> String {
        format!("browser:{}:{}", self.name, self.bytes.len())
    }
}

#[cfg(feature = "hydrate")]
fn load_selected_file(
    event: leptos::ev::Event,
    bridge: ViewerBridge,
    set_stage_name: WriteSignal<String>,
) {
    use wasm_bindgen::JsCast;

    let Some(input) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return;
    };
    let Some(file) = input.files().and_then(|files| files.get(0)) else {
        return;
    };

    let name = file.name();
    wasm_bindgen_futures::spawn_local(async move {
        match gloo_file::futures::read_as_bytes(&gloo_file::File::from(file)).await {
            Ok(bytes) => {
                set_stage_name.set(name.clone());
                bridge.send(ViewerCommand::LoadUsd {
                    name,
                    bytes: std::sync::Arc::new(bytes),
                });
            }
            Err(error) => leptos::logging::error!("Could not read the selected USD file: {error}"),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_selected_file(
    _event: leptos::ev::Event,
    _bridge: ViewerBridge,
    _set_stage_name: WriteSignal<String>,
) {
}

#[cfg(feature = "hydrate")]
fn install_stage_feedback(
    receiver: StageEventReceiver,
    set_stage_name: WriteSignal<String>,
    set_stage_prims: WriteSignal<Vec<StagePrimInfo>>,
    set_selected: WriteSignal<StagePrimInfo>,
    set_mesh_count: WriteSignal<usize>,
    set_stage_status: WriteSignal<String>,
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
            }
            ViewerEvent::StageLoadFailed { name, error } => {
                set_stage_name.set(name);
                set_stage_status.set(format!("LOAD FAILED: {error}"));
            }
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn install_stage_feedback(
    _receiver: StageEventReceiver,
    _set_stage_name: WriteSignal<String>,
    _set_stage_prims: WriteSignal<Vec<StagePrimInfo>>,
    _set_selected: WriteSignal<StagePrimInfo>,
    _set_mesh_count: WriteSignal<usize>,
    _set_stage_status: WriteSignal<String>,
) {
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let (bridge, command_rx, stage_events, event_sender) = viewer_bridge();
    let initial_prims = PRIMS
        .into_iter()
        .map(StagePrimInfo::from)
        .collect::<Vec<_>>();
    let (stage_prims, set_stage_prims) = signal(initial_prims);
    let (selected, set_selected) = signal(StagePrimInfo::from(PRIMS[1]));
    let (auto_orbit, set_auto_orbit) = signal(true);
    let (left_open, set_left_open) = signal(false);
    let (right_open, set_right_open) = signal(false);
    let (stage_name, set_stage_name) = signal("showroom.usda".to_string());
    let (mesh_count, set_mesh_count) = signal(6usize);
    let (stage_status, set_stage_status) = signal("STAGE READY".to_string());

    install_stage_feedback(
        stage_events,
        set_stage_name,
        set_stage_prims,
        set_selected,
        set_mesh_count,
        set_stage_status,
    );

    let select_bridge = bridge.clone();
    let orbit_bridge = bridge.clone();
    let file_bridge = bridge.clone();
    let toggle_orbit = move |_| {
        let next = !auto_orbit.get_untracked();
        set_auto_orbit.set(next);
        orbit_bridge.send(ViewerCommand::SetAutoOrbit(next));
    };

    view! {
        <Stylesheet id="leptos" href="/pkg/webtest.css"/>
        <Title text="OpenUSD Web Viewer"/>
        <Meta name="description" content="A browser-native OpenUSD stage viewer powered by Leptos, Bevy, and bevy_openusd."/>
        <main class="app-shell">
            <header class="topbar">
                <div class="brand" aria-label="OpenUSD Web">
                    <span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i></span>
                    <span class="brand-name">"OPENUSD"</span>
                    <span class="brand-suffix">"// WEB"</span>
                </div>
                <div class="stage-title">
                    <span class="status-dot"></span>
                    <span>{move || stage_name.get()}</span>
                    <small>"LIVE STAGE"</small>
                </div>
                <div class="top-actions">
                    <button
                        class="icon-button mobile-only"
                        aria-label="Toggle scene outliner"
                        on:click=move |_| set_left_open.set(!left_open.get_untracked())
                    >"SCENE"</button>
                    <label class="tool-button file-button">
                        <span aria-hidden="true">"+"</span> "OPEN USD"
                        <input
                            type="file"
                            accept=".usd,.usda,.usdc,.usdz"
                            on:change=move |event| {
                                load_selected_file(event, file_bridge.clone(), set_stage_name)
                            }
                        />
                    </label>
                    <button class="tool-button" on:click=move |_| bridge.send(ViewerCommand::ResetCamera)>
                        <span class="reset-icon" aria-hidden="true">"↺"</span> "FRAME ALL"
                    </button>
                    <button
                        class:active=move || auto_orbit.get()
                        class="tool-button"
                        aria-pressed=move || auto_orbit.get().to_string()
                        on:click=toggle_orbit
                    >
                        <span class="orbit-icon" aria-hidden="true">"◉"</span>
                        {move || if auto_orbit.get() { "ORBIT ON" } else { "ORBIT OFF" }}
                    </button>
                    <button
                        class="icon-button mobile-only"
                        aria-label="Toggle properties"
                        on:click=move |_| set_right_open.set(!right_open.get_untracked())
                    >"INFO"</button>
                </div>
            </header>

            <section class="workspace">
                <aside class:open=move || left_open.get() class="panel outliner">
                    <div class="panel-heading">
                        <div><span class="eyebrow">"STAGE"</span><h2>"Outliner"</h2></div>
                        <span class="count">{move || format!("{} PRIMS", stage_prims.get().len())}</span>
                    </div>
                    <label class="search">
                        <span aria-hidden="true">"⌕"</span>
                        <input type="search" placeholder="Filter stage…" aria-label="Filter stage" />
                        <kbd>"/"</kbd>
                    </label>
                    <nav class="tree" aria-label="Stage prims">
                        {move || stage_prims.get().into_iter().map(|prim| {
                            let prim_bridge = select_bridge.clone();
                            let selected_path = prim.path.clone();
                            let focus_path = prim.path.clone();
                            let click_prim = prim.clone();
                            let icon_class = if prim.kind == "Mesh" { "prim-icon mesh" } else { "prim-icon xform" };
                            view! {
                                <button
                                    type="button"
                                    class="tree-row child-row"
                                    class:selected=move || selected.get().path == selected_path
                                    on:click=move |_| {
                                        set_selected.set(click_prim.clone());
                                        prim_bridge.send(ViewerCommand::FocusPrim(focus_path.clone()));
                                    }
                                >
                                    <span class="branch-line"></span>
                                    <span class=icon_class></span>
                                    <span>{prim.name}</span>
                                    <small>{prim.kind}</small>
                                </button>
                            }
                        }).collect_view()}
                    </nav>
                    <div class="panel-foot">
                        <span><i class="legend mesh-dot"></i>"GEOMETRY"</span>
                        <span><i class="legend xform-dot"></i>"XFORM"</span>
                    </div>
                </aside>

                <section class="viewport-wrap">
                    <div class="viewport-toolbar">
                        <div class="view-mode"><span>"PERSPECTIVE"</span><i></i><span>"MATERIAL"</span></div>
                        <div class="viewport-hint">"DRAG TO ORBIT"<span>"·"</span>"SCROLL TO ZOOM"</div>
                    </div>
                    {viewport_canvas(command_rx, event_sender)}
                    <div class="axis-gizmo" aria-hidden="true">
                        <span class="axis-y">"Y"</span><span class="axis-x">"X"</span><span class="axis-z">"Z"</span>
                        <i class="line-y"></i><i class="line-x"></i><i class="line-z"></i>
                    </div>
                    <div class="viewport-badge"><span></span>"BEVY 0.19 / WEBGL2"</div>
                </section>

                <aside class:open=move || right_open.get() class="panel inspector">
                    <div class="panel-heading inspector-heading">
                        <div><span class="eyebrow">"SELECTION"</span><h2>{move || selected.get().name}</h2></div>
                        <span class="type-pill">{move || selected.get().kind.to_uppercase()}</span>
                    </div>
                    <section class="property-section">
                        <h3><span>"01"</span>"IDENTITY"</h3>
                        <dl>
                            <div><dt>"Path"</dt><dd class="path-value">{move || selected.get().path}</dd></div>
                            <div><dt>"Type"</dt><dd>{move || selected.get().kind}</dd></div>
                            <div><dt>"Purpose"</dt><dd>"default"</dd></div>
                        </dl>
                    </section>
                    <section class="property-section">
                        <h3><span>"02"</span>"TRANSFORM"</h3>
                        <dl>
                            <div><dt>"Translate"</dt><dd class="mono">{move || selected.get().position}</dd></div>
                            <div><dt>"Rotate"</dt><dd class="mono">"0.0, 0.0, 0.0"</dd></div>
                            <div><dt>"Scale"</dt><dd class="mono">"1.0, 1.0, 1.0"</dd></div>
                        </dl>
                    </section>
                    <section class="property-section material-card">
                        <h3><span>"03"</span>"MATERIAL"</h3>
                        <div class="swatch-row"><span class="material-swatch"></span><div><small>"SURFACE"</small><strong>{move || selected.get().color}</strong></div></div>
                        <div class="meter"><span>"ROUGHNESS"</span><i><b style="width: 68%"></b></i><em>"0.68"</em></div>
                        <div class="meter"><span>"METALLIC"</span><i><b style="width: 24%"></b></i><em>"0.24"</em></div>
                    </section>
                    <div class="source-note">
                        <span>"SOURCE"</span>
                        <a href="https://github.com/bresilla/bevy_openusd" target="_blank" rel="noreferrer">"bevy_openusd ↗"</a>
                    </div>
                </aside>
            </section>

            <footer class="statusbar">
                <div><span class="status-dot"></span>{move || stage_status.get()}</div>
                <div>{move || format!("{} PRIMS", stage_prims.get().len())}<span>"·"</span>{move || format!("{} MESHES", mesh_count.get())}</div>
                <div class="runtime-label">"LEPTOS + BEVY + OPENUSD"</div>
            </footer>
        </main>
    }
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
                    prevent_default_event_handling: false,
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
    .add_systems(PostUpdate, apply_pending_stage)
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
        Transform::from_xyz(8.5, 6.2, 10.5).looking_at(Vec3::new(0.0, 0.7, 0.0), Vec3::Y),
        OrbitCamera {
            focus: Vec3::new(0.0, 0.7, 0.0),
            radius: 14.8,
            yaw: 0.68,
            pitch: -0.36,
            auto_orbit: true,
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
            base_color: Color::srgb(0.055, 0.061, 0.068),
            perceptual_roughness: 0.92,
            metallic: 0.12,
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
                    camera.radius = 6.5;
                }
            }
            ViewerCommand::LoadUsd { name, bytes } => {
                pending_stage.0 = Some((name.clone(), bytes.clone()));
            }
        }
    }
}

#[cfg(feature = "hydrate")]
fn apply_pending_stage(world: &mut World) {
    let pending = world.resource_mut::<PendingStage>().0.take();
    let Some((name, bytes)) = pending else {
        return;
    };

    let resolver = BrowserFileResolver {
        name: name.clone(),
        bytes,
    };
    let stage = match Stage::builder().resolver(resolver).open(&name) {
        Ok(stage) => stage,
        Err(error) => {
            bevy::log::error!("Could not open {name}: {error}");
            world.write_message(ViewerEvent::StageLoadFailed {
                name,
                error: error.to_string(),
            });
            return;
        }
    };

    let composition_errors = stage.composition_errors();
    if !composition_errors.is_empty() {
        bevy::log::warn!(
            "Stage {name} loaded with unresolved composition dependencies: {composition_errors:#?}"
        );
    }
    let mut prim_count = 0usize;
    let mut mesh_count = 0usize;
    let mut prims = Vec::new();
    if let Err(error) = stage.traverse(openusd::usd::PrimPredicate::default(), |path| {
        prim_count += 1;
        let kind = stage
            .prim(path.clone())
            .type_name()
            .ok()
            .flatten()
            .unwrap_or_else(|| "Prim".to_string());
        if kind == "Mesh" {
            mesh_count += 1;
        }
        let path = path.as_str().to_string();
        let prim_name = path.rsplit('/').next().unwrap_or(&path).to_string();
        prims.push(StagePrimInfo {
            path,
            name: prim_name,
            kind,
            position: "from USD".to_string(),
            color: "USD material".to_string(),
        });
    }) {
        bevy::log::warn!("Could not inspect {name}: {error}");
    }
    let warning = if mesh_count == 0 {
        Some(format!(
            "NO MESHES: {prim_count} prims; select referenced assets or use USDZ"
        ))
    } else if composition_errors.is_empty() {
        None
    } else {
        Some(format!(
            "LOADED WITH {} UNRESOLVED DEPENDENCIES",
            composition_errors.len()
        ))
    };
    if mesh_count == 0 {
        bevy::log::warn!(
            "Stage {name} contains {prim_count} composed prims but no Mesh prims. Its geometry may be in external files that were not selected."
        );
    } else {
        bevy::log::info!("Stage {name}: {prim_count} prims, {mesh_count} meshes");
    }

    let entities = world
        .resource::<PrimEntities>()
        .iter()
        .map(|(_, entity)| entity)
        .collect::<Vec<_>>();
    for entity in entities {
        world.despawn(entity);
    }
    *world.resource_mut::<PrimEntities>() = PrimEntities::default();
    world.remove_non_send::<LiveStage>();
    world.insert_non_send(LiveStage::new(stage));
    world.resource_mut::<FrameStage>().0 = true;
    world.write_message(ViewerEvent::StageLoaded {
        name: name.clone(),
        prims,
        mesh_count,
        warning,
    });
    bevy::log::info!("Loaded USD stage {name}");
}

#[cfg(feature = "hydrate")]
fn frame_loaded_stage(
    mut request: ResMut<FrameStage>,
    meshes: Res<Assets<Mesh>>,
    geometry: Query<(&Mesh3d, &GlobalTransform), bevy::ecs::query::With<UsdPrimRef>>,
    mut cameras: Query<&mut OrbitCamera>,
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

    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };
    let size = max - min;
    camera.focus = (min + max) * 0.5;
    camera.radius = (size.length() * 0.85).max(0.5);
    request.0 = false;
}

#[cfg(feature = "hydrate")]
fn orbit_camera(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut cameras: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    let Ok((mut transform, mut orbit)) = cameras.single_mut() else {
        return;
    };
    let delta = motion
        .read()
        .fold(Vec2::ZERO, |sum, event| sum + event.delta);
    if mouse.pressed(MouseButton::Left) {
        orbit.yaw -= delta.x * 0.007;
        orbit.pitch = (orbit.pitch - delta.y * 0.007).clamp(-1.25, 0.15);
        orbit.auto_orbit = false;
    } else if orbit.auto_orbit {
        orbit.yaw += time.delta_secs() * 0.12;
    }
    let scroll: f32 = wheel.read().map(|event| event.y).sum();
    orbit.radius = (orbit.radius * (1.0 - scroll * 0.08)).clamp(3.2, 28.0);

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

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="theme-color" content="#121417"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body><App/></body>
        </html>
    }
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    let conf = get_configuration(None).expect("Leptos configuration");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    println!("OpenUSD Web listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind server address");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("serve Axum application");
}

#[cfg(not(feature = "ssr"))]
fn main() {}
