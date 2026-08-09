use leptos::prelude::*;
use leptos_meta::{MetaTags, provide_meta_context};

use crate::bevy_canvas::{install_camera_feedback, install_stage_feedback, viewer_bridge};
use crate::model::{AxisGizmoState, BrowserSceneFiles, initial_selection, showroom_prims};
use crate::ui::{UiState, ViewerPage};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let (bridge, command_rx, stage_events, event_sender, camera_events, camera_sender) =
        viewer_bridge();

    let (stage_prims, set_stage_prims) = signal(showroom_prims());
    let (selected, set_selected) = signal(initial_selection());
    let (auto_orbit, set_auto_orbit) = signal(false);
    let (left_open, set_left_open) = signal(false);
    let (right_open, set_right_open) = signal(false);
    let (stage_name, set_stage_name) = signal("showroom.usda".to_string());
    let (mesh_count, set_mesh_count) = signal(6usize);
    let (stage_status, set_stage_status) = signal("STAGE READY".to_string());
    let (scene_files, set_scene_files) = signal(None::<BrowserSceneFiles>);
    let (missing_dependencies, set_missing_dependencies) = signal(Vec::<String>::new());
    let (axis_gizmo, set_axis_gizmo) = signal(AxisGizmoState::default());
    let (outliner_filter, set_outliner_filter) = signal(String::new());
    let (collapsed_prims, set_collapsed_prims) = signal(std::collections::HashSet::<String>::new());
    let (hidden_prims, set_hidden_prims) = signal(std::collections::HashSet::<String>::new());

    let state = UiState {
        stage_prims,
        set_stage_prims,
        selected,
        set_selected,
        auto_orbit,
        set_auto_orbit,
        left_open,
        set_left_open,
        right_open,
        set_right_open,
        stage_name,
        set_stage_name,
        mesh_count,
        set_mesh_count,
        stage_status,
        set_stage_status,
        scene_files,
        set_scene_files,
        missing_dependencies,
        set_missing_dependencies,
        axis_gizmo,
        outliner_filter,
        set_outliner_filter,
        collapsed_prims,
        set_collapsed_prims,
        hidden_prims,
        set_hidden_prims,
    };

    install_stage_feedback(
        stage_events,
        state.set_stage_name,
        state.set_stage_prims,
        state.set_selected,
        state.set_mesh_count,
        state.set_stage_status,
        state.set_missing_dependencies,
    );
    install_camera_feedback(camera_events, set_axis_gizmo);
    Effect::new(move |_| {
        stage_name.track();
        set_outliner_filter.set(String::new());
        set_collapsed_prims.set(std::collections::HashSet::new());
        set_hidden_prims.set(std::collections::HashSet::new());
    });

    view! { <ViewerPage state bridge command_rx event_sender camera_sender/> }
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
