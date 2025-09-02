use std::{sync::Arc};
use eframe::egui::{self, IconData};

use super::app::AppModel;

/// Trait every tab/page implements.
/// Keeping per-tab state inside each struct makes it easy to add tabs.
pub trait TabPage {
    /// Unique, stable ID for the tab (good for saving/restoring state).
    fn id(&self) -> &'static str;

    /// Human-readable label shown in the sidebar.
    fn label(&self) -> &'static str;

    /// Draw the main content for this tab (right side).
    fn ui(&mut self, ui: &mut egui::Ui, model: &mut AppModel);
}

pub fn load_icon() -> Option<Arc<IconData>> {
	let (icon_rgba, icon_width, icon_height) = {
		let icon = include_bytes!("../assets/icon.png");
		let image = image::load_from_memory(icon)
			.ok()?
			.into_rgba8();
		let (width, height) = image.dimensions();
		let rgba = image.into_raw();
		(rgba, width, height)
	};
	
	Some(IconData {
		rgba: icon_rgba,
		width: icon_width,
		height: icon_height,
	}.into())
}