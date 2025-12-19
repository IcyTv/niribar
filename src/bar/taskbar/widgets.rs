use std::cell::RefCell;

use glib::Properties;
use gtk4::gio::prelude::AppInfoExt;
use gtk4::gio::{self};
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use niri_ipc::Window as NiriWindow;

use crate::icons::{self, Icon};

glib::wrapper! {
	pub struct NiriWindowWidget(ObjectSubclass<niri_window_imp::NiriWindowWidget>)
		@extends gtk4::Button, gtk4::Widget,
		@implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl NiriWindowWidget {
	pub fn from_window(workspace_index: u8, workspace_id: u64, window: &NiriWindow) -> Self {
		let icon = window
			.app_id
			.as_ref()
			.and_then(Self::get_icon_for_app_id)
			.unwrap_or_else(|| gio::Icon::for_string(icons::Icon::FileTerminal.name()).unwrap());

		let tile_pos = window.layout.pos_in_scrolling_layout.unwrap_or_default();
		let tile_pos = (tile_pos.0 as u32, tile_pos.1 as u32);

		let sort_key = ((tile_pos.0 as u64) << 32) | (tile_pos.1 as u64);

		let widget: Self = glib::Object::builder()
			.property("icon", Some(icon))
			.property("sort-key", sort_key)
			.property("title", window.title.clone())
			.property("workspace-index", workspace_index)
			.property("workspace-id", workspace_id)
			.property("window-id", window.id)
			.build();

		widget
	}

	fn get_icon_for_app_id(app_id: impl AsRef<str>) -> Option<gio::Icon> {
		let app_info = gio::DesktopAppInfo::new(&format!("{}.desktop", app_id.as_ref()))?;

		app_info.icon()
	}
}

mod niri_window_imp {
	use super::*;

	#[derive(Properties, Default)]
	#[properties(wrapper_type = super::NiriWindowWidget)]
	pub struct NiriWindowWidget {
		#[property(get, construct_only)]
		window_id:       RefCell<u64>,
		#[property(get, set)]
		pub icon:        RefCell<Option<gio::Icon>>,
		#[property(get, construct_only)]
		sort_key:        RefCell<u64>,
		#[property(get, set)]
		title:           RefCell<String>,
		#[property(get, set)]
		workspace_index: RefCell<u8>,
		#[property(get, set)]
		workspace_id:    RefCell<u64>,
	}

	#[glib::object_subclass]
	impl ObjectSubclass for NiriWindowWidget {
		type ParentType = gtk4::Button;
		type Type = super::NiriWindowWidget;

		const NAME: &'static str = "NiriWindowWidget";
	}

	#[glib::derived_properties]
	impl ObjectImpl for NiriWindowWidget {
		fn constructed(&self) {
			self.parent_constructed();

			let obj = self.obj();
			obj.set_css_classes(&["niri-window"]);

			let image = gtk4::Image::new();
			image.set_pixel_size(24);
			obj.set_child(Some(&image));

			obj.bind_property("icon", &image, "gicon").sync_create().build();
		}
	}

	// Trait shared by all widgets
	impl WidgetImpl for NiriWindowWidget {}

	// Trait shared by all buttons
	impl ButtonImpl for NiriWindowWidget {
		fn clicked(&self) {
			// TODO: get() instead of new. Or pass it somewhere...
			let niri = super::super::niri::Niri::new();

			niri.activate_window(*self.window_id.borrow());
		}
	}
}

glib::wrapper! {
	pub struct NiriWorkspaceWidget(ObjectSubclass<niri_workspace_imp::NiriWorkspaceWidget>)
		@extends gtk4::Button, gtk4::Widget,
		@implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl NiriWorkspaceWidget {
	pub fn new_null() -> Self {
		glib::Object::builder()
			.property("icon", None::<String>)
			.property("workspace-id", 0u64)
			.build()
	}
}

mod niri_workspace_imp {
	use super::*;

	#[derive(Properties, Default)]
	#[properties(wrapper_type = super::NiriWorkspaceWidget)]
	pub struct NiriWorkspaceWidget {
		#[property(get, set)]
		pub icon:        RefCell<Option<String>>,
		#[property(get, set)]
		workspace_id:    RefCell<u64>,
		#[property(get, set)]
		workspace_index: RefCell<u8>,
	}

	#[glib::object_subclass]
	impl ObjectSubclass for NiriWorkspaceWidget {
		type ParentType = gtk4::Button;
		type Type = super::NiriWorkspaceWidget;

		const NAME: &'static str = "NiriWorkspaceWidget";
	}

	#[glib::derived_properties]
	impl ObjectImpl for NiriWorkspaceWidget {
		fn constructed(&self) {
			self.parent_constructed();

			let obj = self.obj();
			obj.set_css_classes(&["niri-workspace"]);

			let stack = gtk4::Stack::builder().hhomogeneous(true).vhomogeneous(true).build();

			let image = gtk4::Image::new();
			image.set_pixel_size(24);

			let label = gtk4::Label::new(None);

			stack.add_named(&image, Some("icon"));
			stack.add_named(&label, Some("label"));

			obj.set_child(Some(&stack));

			obj.bind_property("icon", &image, "icon-name").sync_create().build();
			obj.bind_property("workspace-index", &label, "label")
				.sync_create()
				.transform_to(|_, id: u8| Some(format!("{}", id + 1)))
				.build();

			obj.bind_property("icon", &stack, "visible-child-name")
				.sync_create()
				.transform_to(|_, v: Option<String>| {
					let name = if v.is_some() { "icon" } else { "label" };
					Some(name)
				})
				.build();
		}
	}

	impl WidgetImpl for NiriWorkspaceWidget {}

	impl ButtonImpl for NiriWorkspaceWidget {
		fn clicked(&self) {
			let niri = super::super::niri::Niri::new();
			niri.activate_workspace(*self.workspace_id.borrow());
		}
	}
}
