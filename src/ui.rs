use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title};

use crate::bevy_canvas::{
    CameraEventSender, CanvasEventSender, CanvasReceiver, ViewerBridge, viewport_canvas,
};
use crate::model::{AxisGizmoState, BrowserSceneFiles, StagePrimInfo, ViewerCommand};
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
    pub(crate) axis_gizmo: ReadSignal<AxisGizmoState>,
    pub(crate) outliner_filter: ReadSignal<String>,
    pub(crate) set_outliner_filter: WriteSignal<String>,
    pub(crate) collapsed_prims: ReadSignal<std::collections::HashSet<String>>,
    pub(crate) set_collapsed_prims: WriteSignal<std::collections::HashSet<String>>,
    pub(crate) hidden_prims: ReadSignal<std::collections::HashSet<String>>,
    pub(crate) set_hidden_prims: WriteSignal<std::collections::HashSet<String>>,
}

fn prim_path_depth(path: &str) -> usize {
    path.split('/').filter(|part| !part.is_empty()).count()
}

fn is_descendant_path(path: &str, ancestor: &str) -> bool {
    path.strip_prefix(ancestor)
        .is_some_and(|suffix| suffix.starts_with('/'))
}

fn matches_prim_filter(prim: &StagePrimInfo, filter: &str) -> bool {
    prim.name.to_lowercase().contains(filter)
        || prim.path.to_lowercase().contains(filter)
        || prim.kind.to_lowercase().contains(filter)
}

#[component]
pub(crate) fn ViewerPage(
    state: UiState,
    bridge: ViewerBridge,
    command_rx: CanvasReceiver,
    event_sender: CanvasEventSender,
    camera_sender: CameraEventSender,
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
                <ViewportPane state command_rx event_sender camera_sender/>
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
            <div class="outliner-titlebar">
                <span class="scene-caret" aria-hidden="true">"⌄"</span>
                <span class="scene-folder" aria-hidden="true"></span>
                <strong>"SCENE"</strong>
            </div>
            <label class="outliner-search">
                <span aria-hidden="true">"⌕"</span>
                <input
                    type="search"
                    placeholder="filter by name / path…"
                    aria-label="Filter scene"
                    prop:value=move || state.outliner_filter.get()
                    on:input=move |event| {
                        state.set_outliner_filter.set(event_target_value(&event))
                    }
                />
            </label>
            <nav class="tree" role="tree" aria-label="Stage prims">
                {move || {
                    let prims = state.stage_prims.get();
                    let collapsed = state.collapsed_prims.get();
                    let hidden = state.hidden_prims.get();
                    let filter = state.outliner_filter.get().trim().to_lowercase();
                    let base_depth = prims
                        .iter()
                        .map(|prim| prim_path_depth(&prim.path))
                        .min()
                        .unwrap_or(1);
                    let select_bridge = bridge.clone();
                    prims
                        .iter()
                        .filter(|prim| {
                            let hidden_by_parent = filter.is_empty()
                                && collapsed
                                    .iter()
                                    .any(|parent| is_descendant_path(&prim.path, parent));
                            if hidden_by_parent {
                                return false;
                            }
                            filter.is_empty()
                                || matches_prim_filter(prim, &filter)
                                || prims.iter().any(|candidate| {
                                    is_descendant_path(&candidate.path, &prim.path)
                                        && matches_prim_filter(candidate, &filter)
                                })
                        })
                        .cloned()
                        .map(|prim| {
                            let has_children = prims
                                .iter()
                                .any(|candidate| is_descendant_path(&candidate.path, &prim.path));
                            let depth = prim_path_depth(&prim.path).saturating_sub(base_depth);
                            let is_collapsed = collapsed.contains(&prim.path);
                            let is_hidden = hidden.contains(&prim.path);
                            let prim_bridge = select_bridge.clone();
                            let visibility_bridge = select_bridge.clone();
                            let selected_path = prim.path.clone();
                            let focus_path = prim.path.clone();
                            let collapse_path = prim.path.clone();
                            let visibility_path = prim.path.clone();
                            let click_prim = prim.clone();
                            let icon_class = if prim.kind == "Mesh" {
                                "prim-cube mesh"
                            } else {
                                "prim-cube"
                            };
                            view! {
                                <div
                                    class="tree-row"
                                    class:selected=move || state.selected.get().path == selected_path
                                    class:muted=is_hidden
                                    role="treeitem"
                                    attr:aria-level=(depth + 1).to_string()
                                    attr:aria-expanded=has_children.then(|| (!is_collapsed).to_string())
                                    style=format!("--tree-indent: {}px", depth * 13)
                                >
                                    <button
                                        type="button"
                                        class="tree-disclosure"
                                        class:empty=!has_children
                                        aria-label=if is_collapsed { "Expand prim" } else { "Collapse prim" }
                                        on:click=move |_| {
                                            if has_children {
                                                let mut paths = state.collapsed_prims.get_untracked();
                                                if !paths.insert(collapse_path.clone()) {
                                                    paths.remove(&collapse_path);
                                                }
                                                state.set_collapsed_prims.set(paths);
                                            }
                                        }
                                    >
                                        {if is_collapsed { "›" } else { "⌄" }}
                                    </button>
                                    <span class=icon_class></span>
                                    <button
                                        type="button"
                                        class="tree-select"
                                        on:click=move |_| {
                                            state.set_selected.set(click_prim.clone());
                                            prim_bridge.send(ViewerCommand::FocusPrim(focus_path.clone()));
                                        }
                                    >
                                        <span>{prim.name}</span>
                                        <small>{prim.kind}</small>
                                    </button>
                                    <button
                                        type="button"
                                        class="tree-visibility"
                                        class:hidden=is_hidden
                                        aria-label=if is_hidden { "Show prim" } else { "Hide prim" }
                                        aria-pressed=(!is_hidden).to_string()
                                        on:click=move |_| {
                                            let mut paths = state.hidden_prims.get_untracked();
                                            if !paths.insert(visibility_path.clone()) {
                                                paths.remove(&visibility_path);
                                            }
                                            state.set_hidden_prims.set(paths);
                                            visibility_bridge.send(ViewerCommand::SetPrimVisibility {
                                                path: visibility_path.clone(),
                                                visible: is_hidden,
                                            });
                                        }
                                    >
                                        <svg viewBox="0 0 16 10" aria-hidden="true">
                                            <path d="M1 5c1.7-2.5 4-3.75 7-3.75S13.3 2.5 15 5c-1.7 2.5-4 3.75-7 3.75S2.7 7.5 1 5Z"/>
                                            <circle cx="8" cy="5" r="1.7"/>
                                        </svg>
                                    </button>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </nav>
        </aside>
    }
}

fn axis_gizmo_view(axis_gizmo: ReadSignal<AxisGizmoState>) -> impl IntoView {
    let coordinate = move |axis: usize, component: usize, radius: f32| {
        32.0 + axis_gizmo.get().axes[axis][component] * radius
    };
    let opacity = move |axis: usize| {
        let depth = axis_gizmo.get().axes[axis][2];
        0.48 + (depth + 1.0) * 0.24
    };

    view! {
        <svg class="axis-gizmo" viewBox="0 0 64 64" aria-hidden="true">
            <line
                class="axis-line axis-x"
                x1="32" y1="32"
                x2=move || coordinate(0, 0, 22.0)
                y2=move || coordinate(0, 1, 22.0)
                opacity=move || opacity(0)
            />
            <line
                class="axis-line axis-y"
                x1="32" y1="32"
                x2=move || coordinate(1, 0, 22.0)
                y2=move || coordinate(1, 1, 22.0)
                opacity=move || opacity(1)
            />
            <line
                class="axis-line axis-z"
                x1="32" y1="32"
                x2=move || coordinate(2, 0, 22.0)
                y2=move || coordinate(2, 1, 22.0)
                opacity=move || opacity(2)
            />
            <circle class="axis-origin" cx="32" cy="32" r="2"/>
            <text
                class="axis-label axis-x"
                x=move || coordinate(0, 0, 27.0)
                y=move || coordinate(0, 1, 27.0)
                opacity=move || opacity(0)
            >"X"</text>
            <text
                class="axis-label axis-y"
                x=move || coordinate(1, 0, 27.0)
                y=move || coordinate(1, 1, 27.0)
                opacity=move || opacity(1)
            >"Y"</text>
            <text
                class="axis-label axis-z"
                x=move || coordinate(2, 0, 27.0)
                y=move || coordinate(2, 1, 27.0)
                opacity=move || opacity(2)
            >"Z"</text>
        </svg>
    }
}

#[component]
fn ViewportPane(
    state: UiState,
    command_rx: CanvasReceiver,
    event_sender: CanvasEventSender,
    camera_sender: CameraEventSender,
) -> impl IntoView {
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
            {viewport_canvas(command_rx, event_sender, camera_sender)}
            {axis_gizmo_view(state.axis_gizmo)}
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
