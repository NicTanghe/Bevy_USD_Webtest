use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title};

use crate::bevy_canvas::{CanvasEventSender, CanvasReceiver, ViewerBridge, viewport_canvas};
use crate::model::{BrowserSceneFiles, StagePrimInfo, ViewerCommand};
use crate::usd_loader::{load_dependency_folder, load_selected_file, request_dependency_folder};

#[derive(Clone, Copy)]
pub(crate) struct UiState {
    pub(crate) stage_prims: ReadSignal<Vec<StagePrimInfo>>,
    pub(crate) set_stage_prims: WriteSignal<Vec<StagePrimInfo>>,
    pub(crate) selected: ReadSignal<StagePrimInfo>,
    pub(crate) set_selected: WriteSignal<StagePrimInfo>,
    pub(crate) auto_orbit: ReadSignal<bool>,
    pub(crate) set_auto_orbit: WriteSignal<bool>,
    pub(crate) left_open: ReadSignal<bool>,
    pub(crate) set_left_open: WriteSignal<bool>,
    pub(crate) right_open: ReadSignal<bool>,
    pub(crate) set_right_open: WriteSignal<bool>,
    pub(crate) stage_name: ReadSignal<String>,
    pub(crate) set_stage_name: WriteSignal<String>,
    pub(crate) mesh_count: ReadSignal<usize>,
    pub(crate) set_mesh_count: WriteSignal<usize>,
    pub(crate) stage_status: ReadSignal<String>,
    pub(crate) set_stage_status: WriteSignal<String>,
    pub(crate) scene_files: ReadSignal<Option<BrowserSceneFiles>>,
    pub(crate) set_scene_files: WriteSignal<Option<BrowserSceneFiles>>,
    pub(crate) missing_dependencies: ReadSignal<Vec<String>>,
    pub(crate) set_missing_dependencies: WriteSignal<Vec<String>>,
}

#[component]
pub(crate) fn ViewerPage(
    state: UiState,
    bridge: ViewerBridge,
    command_rx: CanvasReceiver,
    event_sender: CanvasEventSender,
) -> impl IntoView {
    view! {
        <Stylesheet id="leptos" href="/pkg/webtest.css"/>
        <Title text="OpenUSD Web Viewer"/>
        <Meta
            name="description"
            content="A browser-native OpenUSD stage viewer powered by Leptos, Bevy, and bevy_openusd."
        />
        <main class="app-shell">
            <TopBar state bridge=bridge.clone()/>
            <DependencyFolderInput state bridge=bridge.clone()/>
            <DependencyDialog state/>
            <section class="workspace">
                <ScenePanel state bridge=bridge.clone()/>
                <ViewportPane command_rx event_sender/>
                <PropertiesPanel state/>
            </section>
            <StatusBar state/>
        </main>
    }
}

#[component]
fn TopBar(state: UiState, bridge: ViewerBridge) -> impl IntoView {
    let file_bridge = bridge.clone();
    let orbit_bridge = bridge.clone();
    let toggle_orbit = move |_| {
        let next = !state.auto_orbit.get_untracked();
        state.set_auto_orbit.set(next);
        orbit_bridge.send(ViewerCommand::SetAutoOrbit(next));
    };

    view! {
        <header class="topbar">
            <div class="brand" aria-label="OpenUSD Web">
                <span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i></span>
                <span class="brand-name">"OPENUSD"</span>
                <span class="brand-suffix">"// WEB"</span>
            </div>
            <div class="stage-title">
                <span class="status-dot"></span>
                <span>{move || state.stage_name.get()}</span>
                <small>"LIVE STAGE"</small>
            </div>
            <div class="top-actions">
                <button
                    class="icon-button mobile-only"
                    aria-label="Toggle scene outliner"
                    on:click=move |_| {
                        state.set_left_open.set(!state.left_open.get_untracked())
                    }
                >
                    "SCENE"
                </button>
                <label class="tool-button file-button">
                    <span aria-hidden="true">"+"</span> "OPEN USD"
                    <input
                        type="file"
                        accept=".usd,.usda,.usdc,.usdz"
                        on:change=move |event| {
                            load_selected_file(
                                event,
                                file_bridge.clone(),
                                state.set_stage_name,
                                state.set_stage_status,
                                state.set_scene_files,
                                state.set_missing_dependencies,
                            )
                        }
                    />
                </label>
                <button
                    class="tool-button"
                    on:click=move |_| bridge.send(ViewerCommand::ResetCamera)
                >
                    <span class="reset-icon" aria-hidden="true">"↺"</span> "FRAME ALL"
                </button>
                <button
                    class:active=move || state.auto_orbit.get()
                    class="tool-button"
                    aria-pressed=move || state.auto_orbit.get().to_string()
                    on:click=toggle_orbit
                >
                    <span class="orbit-icon" aria-hidden="true">"◉"</span>
                    {move || if state.auto_orbit.get() { "ORBIT ON" } else { "ORBIT OFF" }}
                </button>
                <button
                    class="icon-button mobile-only"
                    aria-label="Toggle properties"
                    on:click=move |_| {
                        state.set_right_open.set(!state.right_open.get_untracked())
                    }
                >
                    "INFO"
                </button>
            </div>
        </header>
    }
}

#[component]
fn DependencyFolderInput(state: UiState, bridge: ViewerBridge) -> impl IntoView {
    let folder_picker_attribute =
        leptos::tachys::html::attribute::custom::custom_attribute("webkitdirectory", "");

    view! {
        <input
            {..folder_picker_attribute}
            id="usd_dependency_folder"
            class="dependency-folder-input"
            type="file"
            multiple
            on:change=move |event| {
                load_dependency_folder(
                    event,
                    bridge.clone(),
                    state.scene_files,
                    state.set_scene_files,
                    state.set_stage_status,
                )
            }
        />
    }
}

#[component]
fn DependencyDialog(state: UiState) -> impl IntoView {
    view! {
        <Show when=move || !state.missing_dependencies.get().is_empty()>
            <div class="dependency-modal-backdrop">
                <section
                    class="dependency-modal"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="dependency-title"
                >
                    <div class="dependency-modal-heading">
                        <span class="eyebrow">"USD COMPOSITION"</span>
                        <h2 id="dependency-title">"Referenced files need access"</h2>
                    </div>
                    <p>
                        "The selected file remains the root stage. Grant access to the folder containing these referenced USD files:"
                    </p>
                    <ul class="dependency-list">
                        {move || {
                            state
                                .missing_dependencies
                                .get()
                                .into_iter()
                                .map(|path| view! { <li>{path}</li> })
                                .collect_view()
                        }}
                    </ul>
                    <p class="dependency-note">
                        "The folder is read only for this session. If another reference is outside it, you will be asked again."
                    </p>
                    <div class="dependency-actions">
                        <button
                            type="button"
                            class="dependency-secondary"
                            on:click=move |_| state.set_missing_dependencies.set(Vec::new())
                        >
                            "NOT NOW"
                        </button>
                        <button
                            type="button"
                            class="dependency-primary"
                            on:click=move |_| request_dependency_folder()
                        >
                            "GRANT FOLDER ACCESS"
                        </button>
                    </div>
                </section>
            </div>
        </Show>
    }
}

#[component]
fn ScenePanel(state: UiState, bridge: ViewerBridge) -> impl IntoView {
    view! {
        <aside class:open=move || state.left_open.get() class="panel outliner">
            <div class="panel-heading">
                <div><span class="eyebrow">"STAGE"</span><h2>"Outliner"</h2></div>
                <span class="count">
                    {move || format!("{} PRIMS", state.stage_prims.get().len())}
                </span>
            </div>
            <label class="search">
                <span aria-hidden="true">"⌕"</span>
                <input type="search" placeholder="Filter stage…" aria-label="Filter stage" />
                <kbd>"/"</kbd>
            </label>
            <nav class="tree" aria-label="Stage prims">
                {move || {
                    let select_bridge = bridge.clone();
                    state
                        .stage_prims
                        .get()
                        .into_iter()
                        .map(|prim| {
                            let prim_bridge = select_bridge.clone();
                            let selected_path = prim.path.clone();
                            let focus_path = prim.path.clone();
                            let click_prim = prim.clone();
                            let icon_class = if prim.kind == "Mesh" {
                                "prim-icon mesh"
                            } else {
                                "prim-icon xform"
                            };
                            view! {
                                <button
                                    type="button"
                                    class="tree-row child-row"
                                    class:selected=move || state.selected.get().path == selected_path
                                    on:click=move |_| {
                                        state.set_selected.set(click_prim.clone());
                                        prim_bridge.send(ViewerCommand::FocusPrim(focus_path.clone()));
                                    }
                                >
                                    <span class="branch-line"></span>
                                    <span class=icon_class></span>
                                    <span>{prim.name}</span>
                                    <small>{prim.kind}</small>
                                </button>
                            }
                        })
                        .collect_view()
                }}
            </nav>
            <div class="panel-foot">
                <span><i class="legend mesh-dot"></i>"GEOMETRY"</span>
                <span><i class="legend xform-dot"></i>"XFORM"</span>
            </div>
        </aside>
    }
}

#[component]
fn ViewportPane(command_rx: CanvasReceiver, event_sender: CanvasEventSender) -> impl IntoView {
    view! {
        <section class="viewport-wrap">
            <div class="viewport-toolbar">
                <div class="view-mode"><span>"PERSPECTIVE"</span><i></i><span>"MATERIAL"</span></div>
                <div class="viewport-hint">
                    "SPACE/ALT + LMB TUMBLE"
                    <span>"·"</span>
                    "MMB TRACK"
                    <span>"·"</span>
                    "RMB/WHEEL DOLLY"
                </div>
            </div>
            {viewport_canvas(command_rx, event_sender)}
            <div class="axis-gizmo" aria-hidden="true">
                <span class="axis-y">"Y"</span>
                <span class="axis-x">"X"</span>
                <span class="axis-z">"Z"</span>
                <i class="line-y"></i><i class="line-x"></i><i class="line-z"></i>
            </div>
            <div class="viewport-badge"><span></span>"BEVY 0.19 / WEBGL2"</div>
        </section>
    }
}

#[component]
fn PropertiesPanel(state: UiState) -> impl IntoView {
    view! {
        <aside class:open=move || state.right_open.get() class="panel inspector">
            <div class="panel-heading inspector-heading">
                <div>
                    <span class="eyebrow">"SELECTION"</span>
                    <h2>{move || state.selected.get().name}</h2>
                </div>
                <span class="type-pill">
                    {move || state.selected.get().kind.to_uppercase()}
                </span>
            </div>
            <section class="property-section">
                <h3><span>"01"</span>"IDENTITY"</h3>
                <dl>
                    <div>
                        <dt>"Path"</dt>
                        <dd class="path-value">{move || state.selected.get().path}</dd>
                    </div>
                    <div><dt>"Type"</dt><dd>{move || state.selected.get().kind}</dd></div>
                    <div><dt>"Purpose"</dt><dd>"default"</dd></div>
                </dl>
            </section>
            <section class="property-section">
                <h3><span>"02"</span>"TRANSFORM"</h3>
                <dl>
                    <div>
                        <dt>"Translate"</dt>
                        <dd class="mono">{move || state.selected.get().position}</dd>
                    </div>
                    <div><dt>"Rotate"</dt><dd class="mono">"0.0, 0.0, 0.0"</dd></div>
                    <div><dt>"Scale"</dt><dd class="mono">"1.0, 1.0, 1.0"</dd></div>
                </dl>
            </section>
            <section class="property-section material-card">
                <h3><span>"03"</span>"MATERIAL"</h3>
                <div class="swatch-row">
                    <span class="material-swatch"></span>
                    <div>
                        <small>"SURFACE"</small>
                        <strong>{move || state.selected.get().color}</strong>
                    </div>
                </div>
                <div class="meter">
                    <span>"ROUGHNESS"</span><i><b style="width: 68%"></b></i><em>"0.68"</em>
                </div>
                <div class="meter">
                    <span>"METALLIC"</span><i><b style="width: 24%"></b></i><em>"0.24"</em>
                </div>
            </section>
            <div class="source-note">
                <span>"SOURCE"</span>
                <a
                    href="https://github.com/bresilla/bevy_openusd"
                    target="_blank"
                    rel="noreferrer"
                >
                    "bevy_openusd ↗"
                </a>
            </div>
        </aside>
    }
}

#[component]
fn StatusBar(state: UiState) -> impl IntoView {
    view! {
        <footer class="statusbar">
            <div><span class="status-dot"></span>{move || state.stage_status.get()}</div>
            <div>
                {move || format!("{} PRIMS", state.stage_prims.get().len())}
                <span>"·"</span>
                {move || format!("{} MESHES", state.mesh_count.get())}
            </div>
            <div class="runtime-label">"LEPTOS + BEVY + OPENUSD"</div>
        </footer>
    }
}
