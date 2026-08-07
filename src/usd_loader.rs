use leptos::prelude::*;

use crate::bevy_canvas::ViewerBridge;
use crate::model::BrowserSceneFiles;

#[cfg(feature = "hydrate")]
use crate::model::{
    BrowserFile, FrameStage, PendingStage, StagePrimInfo, ViewerCommand, ViewerEvent,
};
#[cfg(feature = "hydrate")]
use bevy::prelude::*;
#[cfg(feature = "hydrate")]
use openusd::usd::Stage;
#[cfg(feature = "hydrate")]
use usd_bevy::live::{LiveStage, PrimEntities};

#[cfg(feature = "hydrate")]
struct BrowserFileResolver {
    files: std::sync::Arc<Vec<BrowserFile>>,
}

#[cfg(feature = "hydrate")]
impl BrowserFileResolver {
    fn file(&self, path: &str) -> Option<&BrowserFile> {
        let normalized = normalize_browser_path(path);
        if let Some(file) = self.files.iter().find(|file| file.path == normalized) {
            return Some(file);
        }

        // Directory selection includes the selected folder's own name. Match
        // the authored path against the end of that browser-relative path so
        // either a project root or a narrower dependency folder can be granted.
        let suffix = format!("/{normalized}");
        let mut matches = self
            .files
            .iter()
            .filter(|file| file.path.ends_with(&suffix));
        let file = matches.next();
        if let Some(file) = file
            && matches.next().is_none()
        {
            return Some(file);
        }

        // Some browsers do not expose a directory-relative path. Fall back to
        // the basename only when it identifies exactly one granted file.
        let name = std::path::Path::new(&normalized).file_name()?;
        let mut matches = self
            .files
            .iter()
            .filter(|file| std::path::Path::new(&file.path).file_name() == Some(name));
        let file = matches.next()?;
        matches.next().is_none().then_some(file)
    }
}

#[cfg(feature = "hydrate")]
fn normalize_browser_path(path: &str) -> String {
    let path = path.replace('\\', "/");
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

#[cfg(feature = "hydrate")]
fn join_browser_path(directory: &str, path: &str) -> String {
    if directory.is_empty() {
        normalize_browser_path(path)
    } else {
        normalize_browser_path(&format!("{directory}/{path}"))
    }
}

#[cfg(feature = "hydrate")]
impl openusd::ar::Resolver for BrowserFileResolver {
    fn create_identifier(
        &self,
        asset_path: &str,
        anchor: Option<&openusd::ar::ResolvedPath>,
    ) -> String {
        if asset_path.is_empty() {
            return String::new();
        }
        if openusd::ar::is_package_relative_path(asset_path)
            && let Some((package, inner)) =
                openusd::ar::split_package_relative_path_outer(asset_path)
        {
            let package = self.create_identifier(&package, anchor);
            return openusd::ar::join_package_relative_path(&package, &inner);
        }
        if let Some(anchor) = anchor {
            let anchor = anchor.to_string();
            if openusd::ar::is_package_relative_path(&anchor)
                && let Some((package, inner)) =
                    openusd::ar::split_package_relative_path_inner(&anchor)
            {
                let directory = inner
                    .rsplit_once('/')
                    .map_or("", |(directory, _)| directory);
                let inner = join_browser_path(directory, asset_path);
                return openusd::ar::join_package_relative_path(&package, &inner);
            }
            let directory = anchor
                .rsplit_once('/')
                .map_or("", |(directory, _)| directory);
            return join_browser_path(directory, asset_path);
        }
        normalize_browser_path(asset_path)
    }

    fn resolve(&self, asset_path: &str) -> Option<openusd::ar::ResolvedPath> {
        if openusd::ar::is_package_relative_path(asset_path) {
            let (package, _) = openusd::ar::split_package_relative_path_outer(asset_path)?;
            self.file(&package)
                .map(|_| openusd::ar::ResolvedPath::new(asset_path))
        } else {
            // Preserve the authored identifier as the resolved path. This keeps
            // later relative references anchored correctly even when the file
            // was found through the unique-basename fallback above.
            self.file(asset_path)
                .map(|_| openusd::ar::ResolvedPath::new(normalize_browser_path(asset_path)))
        }
    }

    fn resolve_for_new_asset(&self, asset_path: &str) -> Option<openusd::ar::ResolvedPath> {
        self.resolve(asset_path)
    }

    fn open_asset(
        &self,
        resolved_path: &openusd::ar::ResolvedPath,
    ) -> std::io::Result<Box<dyn openusd::ar::Asset>> {
        let resolved = resolved_path.to_string();
        if let Some((package, inner)) = openusd::ar::split_package_relative_path_outer(&resolved) {
            use std::io::Read;

            let file = self.file(&package).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("package is not available in the browser: {package}"),
                )
            })?;
            let cursor = std::io::Cursor::new((*file.bytes).clone());
            let mut archive = zip::ZipArchive::new(cursor).map_err(std::io::Error::other)?;
            let mut entry = archive.by_name(&inner).map_err(std::io::Error::other)?;
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            return Ok(Box::new(std::io::Cursor::new(bytes)));
        }

        let file = self.file(&resolved).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("asset is not available in the browser: {resolved_path}"),
            )
        })?;
        Ok(Box::new(std::io::Cursor::new((*file.bytes).clone())))
    }

    fn identity(&self) -> String {
        let files = self
            .files
            .iter()
            .map(|file| format!("{}:{}", file.path, file.bytes.len()))
            .collect::<Vec<_>>()
            .join("|");
        format!("browser:{files}")
    }
}

#[cfg(feature = "hydrate")]
fn webkit_relative_path(file: &web_sys::File) -> String {
    js_sys::Reflect::get(
        file.as_ref(),
        &wasm_bindgen::JsValue::from_str("webkitRelativePath"),
    )
    .ok()
    .and_then(|path| path.as_string())
    .unwrap_or_default()
}

#[cfg(feature = "hydrate")]
fn send_browser_scene(bridge: &ViewerBridge, scene: &BrowserSceneFiles) {
    bridge.send(ViewerCommand::LoadUsd {
        name: scene.root.clone(),
        files: std::sync::Arc::new(scene.files.clone()),
    });
}

#[cfg(feature = "hydrate")]
pub(crate) fn request_dependency_folder() {
    use wasm_bindgen::JsCast;

    let Some(input) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("usd_dependency_folder"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return;
    };
    input.click();
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn request_dependency_folder() {}

#[cfg(feature = "hydrate")]
pub(crate) fn load_selected_file(
    event: leptos::ev::Event,
    bridge: ViewerBridge,
    set_stage_name: WriteSignal<String>,
    set_stage_status: WriteSignal<String>,
    set_scene_files: WriteSignal<Option<BrowserSceneFiles>>,
    set_missing_dependencies: WriteSignal<Vec<String>>,
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
    input.set_value("");

    let name = file.name();
    set_stage_name.set(name.clone());
    set_stage_status.set("LOADING STAGE…".to_string());
    set_missing_dependencies.set(Vec::new());
    wasm_bindgen_futures::spawn_local(async move {
        match gloo_file::futures::read_as_bytes(&gloo_file::File::from(file)).await {
            Ok(bytes) => {
                let scene = BrowserSceneFiles {
                    root: name.clone(),
                    files: vec![BrowserFile {
                        path: name,
                        bytes: std::sync::Arc::new(bytes),
                    }],
                };
                set_scene_files.set(Some(scene.clone()));
                send_browser_scene(&bridge, &scene);
            }
            Err(error) => {
                let message = format!("Could not read the selected USD file: {error}");
                leptos::logging::error!("{message}");
                set_stage_status.set(format!("LOAD FAILED: {message}"));
            }
        }
    });
}

#[cfg(feature = "hydrate")]
pub(crate) fn load_dependency_folder(
    event: leptos::ev::Event,
    bridge: ViewerBridge,
    scene_files: ReadSignal<Option<BrowserSceneFiles>>,
    set_scene_files: WriteSignal<Option<BrowserSceneFiles>>,
    set_stage_status: WriteSignal<String>,
) {
    use wasm_bindgen::JsCast;

    let Some(input) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return;
    };
    let Some(file_list) = input.files() else {
        return;
    };
    let selected = (0..file_list.length())
        .filter_map(|index| file_list.get(index))
        .map(|file| {
            let path = webkit_relative_path(&file);
            let path = if path.is_empty() { file.name() } else { path };
            (file, normalize_browser_path(&path))
        })
        .collect::<Vec<_>>();
    input.set_value("");
    let Some(mut scene) = scene_files.get_untracked() else {
        return;
    };
    if selected.is_empty() {
        set_stage_status.set("NO FILES FOUND IN GRANTED FOLDER".to_string());
        return;
    }

    set_stage_status.set("READING DEPENDENCIES…".to_string());
    wasm_bindgen_futures::spawn_local(async move {
        for (file, path) in selected {
            let bytes = match gloo_file::futures::read_as_bytes(&gloo_file::File::from(file)).await
            {
                Ok(bytes) => std::sync::Arc::new(bytes),
                Err(error) => {
                    let message = format!("Could not read dependency {path}: {error}");
                    leptos::logging::error!("{message}");
                    set_stage_status.set(format!("LOAD FAILED: {message}"));
                    return;
                }
            };

            // The explicitly selected root always wins over a same-named file
            // encountered while granting a directory.
            if path == scene.root {
                continue;
            }
            if let Some(existing) = scene.files.iter_mut().find(|file| file.path == path) {
                existing.bytes = bytes;
            } else {
                scene.files.push(BrowserFile { path, bytes });
            }
        }
        set_scene_files.set(Some(scene.clone()));
        set_stage_status.set("RECOMPOSING STAGE…".to_string());
        send_browser_scene(&bridge, &scene);
    });
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn load_selected_file(
    _event: leptos::ev::Event,
    _bridge: ViewerBridge,
    _set_stage_name: WriteSignal<String>,
    _set_stage_status: WriteSignal<String>,
    _set_scene_files: WriteSignal<Option<BrowserSceneFiles>>,
    _set_missing_dependencies: WriteSignal<Vec<String>>,
) {
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn load_dependency_folder(
    _event: leptos::ev::Event,
    _bridge: ViewerBridge,
    _scene_files: ReadSignal<Option<BrowserSceneFiles>>,
    _set_scene_files: WriteSignal<Option<BrowserSceneFiles>>,
    _set_stage_status: WriteSignal<String>,
) {
}

#[cfg(feature = "hydrate")]
pub(crate) fn apply_pending_stage(world: &mut World) {
    let pending = world.resource_mut::<PendingStage>().0.take();
    let Some((name, files)) = pending else {
        return;
    };

    let selected_file_count = files.len();
    let resolver = BrowserFileResolver { files };
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
    let mut missing_dependencies = composition_errors
        .iter()
        .filter_map(|error| match error {
            openusd::pcp::Error::UnresolvedLayer { asset_path, .. }
            | openusd::pcp::Error::UnresolvedSublayer { asset_path, .. } => {
                Some(asset_path.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    missing_dependencies.sort();
    missing_dependencies.dedup();

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
            .map(|kind| kind.to_string())
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

    // Some valid stages compose an instancer/prototype hierarchy without
    // reporting unresolved arcs, even though the referenced geometry was not
    // available to the browser resolver. Offer folder access on the initial
    // single-file load so the user can provide those unreported dependencies.
    if mesh_count == 0 && missing_dependencies.is_empty() && selected_file_count == 1 {
        missing_dependencies.push(
            "Referenced geometry files (select the folder containing this stage)".to_string(),
        );
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
        // Despawning a parent recursively removes its children. The bimap also
        // contains those child IDs, so check each one before touching it again.
        if world.get_entity(entity).is_ok() {
            world.despawn(entity);
        }
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
        missing_dependencies,
    });
    bevy::log::info!("Loaded USD stage {name}");
}
