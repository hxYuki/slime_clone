mod title;

use bevy::{
    ecs::template::template,
    scene::{Scene, bsn},
    state::{state::States, state_scoped::DespawnOnExit},
    ui::{Node, percent},
};
pub use title::*;

use crate::AppState;

pub fn ui_root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
        }
    }
}

pub fn ui_root_despawn_on_exit<S: States + Default + Clone + Unpin>(state: S) -> impl Scene {
    bsn! {
        ui_root()
        DespawnOnExit<S>({state.clone()})
    }
}
