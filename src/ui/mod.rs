pub mod main_menu;

use bevy::prelude::*;
use main_menu::MenuPlugin;

pub fn setup_ui(app: &mut App) {
    app.add_plugins(MenuPlugin);
}
