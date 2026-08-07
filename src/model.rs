#[cfg(feature = "hydrate")]
use bevy::prelude::{Message, Resource};

#[derive(Clone)]
#[cfg_attr(feature = "hydrate", derive(Message))]
pub(crate) enum ViewerCommand {
    ResetCamera,
    SetAutoOrbit(bool),
    FocusPrim(String),
    #[cfg(feature = "hydrate")]
    LoadUsd {
        name: String,
        files: std::sync::Arc<Vec<BrowserFile>>,
    },
}

#[cfg(feature = "hydrate")]
#[derive(Clone)]
pub(crate) struct BrowserFile {
    pub(crate) path: String,
    pub(crate) bytes: std::sync::Arc<Vec<u8>>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone)]
pub(crate) struct BrowserSceneFiles {
    pub(crate) root: String,
    pub(crate) files: Vec<BrowserFile>,
}

#[cfg(not(feature = "hydrate"))]
#[derive(Clone)]
pub(crate) struct BrowserSceneFiles;

#[derive(Clone, PartialEq)]
pub(crate) struct StagePrimInfo {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) position: String,
    pub(crate) color: String,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Message)]
pub(crate) enum ViewerEvent {
    StageLoaded {
        name: String,
        prims: Vec<StagePrimInfo>,
        mesh_count: usize,
        warning: Option<String>,
        missing_dependencies: Vec<String>,
    },
    StageLoadFailed {
        name: String,
        error: String,
    },
}

#[cfg(feature = "hydrate")]
#[derive(Resource, Default)]
pub(crate) struct PendingStage(pub(crate) Option<(String, std::sync::Arc<Vec<BrowserFile>>)>);

#[cfg(feature = "hydrate")]
#[derive(Resource, Default)]
pub(crate) struct FrameStage(pub(crate) bool);

#[derive(Clone, Copy)]
struct ShowroomPrim {
    path: &'static str,
    name: &'static str,
    kind: &'static str,
    position: &'static str,
    color: &'static str,
}

impl From<ShowroomPrim> for StagePrimInfo {
    fn from(prim: ShowroomPrim) -> Self {
        Self {
            path: prim.path.to_string(),
            name: prim.name.to_string(),
            kind: prim.kind.to_string(),
            position: prim.position.to_string(),
            color: prim.color.to_string(),
        }
    }
}

const SHOWROOM_PRIMS: [ShowroomPrim; 6] = [
    ShowroomPrim {
        path: "/World/Anchor",
        name: "Anchor",
        kind: "Cube",
        position: "−2.4, 0.8, 0.0",
        color: "Oxide",
    },
    ShowroomPrim {
        path: "/World/Core",
        name: "Core",
        kind: "Sphere",
        position: "0.0, 1.0, 0.0",
        color: "Signal",
    },
    ShowroomPrim {
        path: "/World/Tower",
        name: "Tower",
        kind: "Cylinder",
        position: "2.4, 1.0, 0.0",
        color: "Ceramic",
    },
    ShowroomPrim {
        path: "/World/Link",
        name: "Link",
        kind: "Capsule",
        position: "−1.3, 1.1, −2.2",
        color: "Alloy",
    },
    ShowroomPrim {
        path: "/World/Marker",
        name: "Marker",
        kind: "Cone",
        position: "1.3, 1.0, −2.2",
        color: "Carbon",
    },
    ShowroomPrim {
        path: "/World/Deck",
        name: "Deck",
        kind: "Cube",
        position: "0.0, 0.2, 2.2",
        color: "Slate",
    },
];

pub(crate) fn showroom_prims() -> Vec<StagePrimInfo> {
    SHOWROOM_PRIMS
        .into_iter()
        .map(StagePrimInfo::from)
        .collect()
}

pub(crate) fn initial_selection() -> StagePrimInfo {
    StagePrimInfo::from(SHOWROOM_PRIMS[1])
}
