use std::cell::RefCell;

use astal4::prelude::*;
use glib::Properties;
use gtk4::CompositeTemplate;
use gtk4::gio::{self};
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use crate::icons;

pub struct PopupLauncher {
	pub window: astal4::Window,
	content:    PopupLauncherContent,
}

impl PopupLauncher {
	pub fn new() -> Self {
		let window = astal4::Window::builder()
			.anchor(astal4::WindowAnchor::LEFT | astal4::WindowAnchor::TOP)
			.layer(astal4::Layer::Overlay)
			.keymode(astal4::Keymode::None)
			.build();

		let launcher_content = PopupLauncherContent::new();

		window.set_child(Some(&launcher_content));

		Self {
			window,
			content: launcher_content,
		}
	}
}

glib::wrapper! {
	struct PopupLauncherContent(ObjectSubclass<imp::PopupLauncherContent>)
		@extends gtk4::Box, gtk4::Widget,
		@implements gtk4::Accessible, gtk4::Buildable, gtk4::Constraint, gtk4::ConstraintTarget, gtk4::ShortcutManager, gtk4::Root, gtk4::Native;
}

impl PopupLauncherContent {
	pub fn new() -> Self {
		glib::Object::builder()
			.property("power-off-icon", icons::Icon::Power.name())
			.property("restart-icon", icons::Icon::RotateCcw.name())
			.property("logout-icon", icons::Icon::LogOut.name())
			.build()
	}
}

mod imp {

	use super::*;

	#[derive(Default, Properties, CompositeTemplate)]
	#[template(file = "./src/popups/launcher.blp")]
	#[properties(wrapper_type = super::PopupLauncherContent)]
	pub(super) struct PopupLauncherContent {
		#[property(get, set)]
		pub power_off_icon: RefCell<String>,
		#[property(get, set)]
		pub restart_icon:   RefCell<String>,
		#[property(get, set)]
		pub logout_icon:    RefCell<String>,
	}

	#[glib::object_subclass]
	impl ObjectSubclass for PopupLauncherContent {
		type ParentType = gtk4::Box;
		type Type = super::PopupLauncherContent;

		const NAME: &'static str = "PopupLauncher";

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
			// Self::Type::bind_template_callback(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	#[glib::derived_properties]
	impl ObjectImpl for PopupLauncherContent {
		fn constructed(&self) {
			self.parent_constructed();

			let obj = self.obj();
		}
	}

	impl WidgetImpl for PopupLauncherContent {}

	impl BoxImpl for PopupLauncherContent {}
}
