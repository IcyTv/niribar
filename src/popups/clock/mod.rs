use std::cell::RefCell;

use astal_mpris::prelude::PlayerExt;
use astal_mpris::{Loop, Player, Shuffle};
use glib::{Properties, clone};
use gtk4::CompositeTemplate;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use crate::icons::Icon;

glib::wrapper! {
	pub struct ClockPopup(ObjectSubclass<imp::ClockPopup>)
		@extends gtk4::Popover, gtk4::Widget,
		@implements gtk4::Accessible, gtk4::Buildable, gtk4::Constraint, gtk4::ConstraintTarget, gtk4::ShortcutManager, gtk4::Native;
}

impl ClockPopup {
	pub fn new() -> Self {
		glib::Object::builder().build()
	}
}

mod imp {

	use super::*;

	#[derive(Default, Properties, CompositeTemplate)]
	#[template(file = "./src/popups/clock/clock.blp")]
	#[properties(wrapper_type = super::ClockPopup)]
	pub struct ClockPopup {}

	#[glib::object_subclass]
	impl ObjectSubclass for ClockPopup {
		type ParentType = gtk4::Popover;
		type Type = super::ClockPopup;
		const NAME: &'static str = "ClockPopup";

		fn class_init(klass: &mut Self::Class) {
			klass.bind_template();
		}
		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	#[glib::derived_properties]
	impl ObjectImpl for ClockPopup {
		fn constructed(&self) {
			self.parent_constructed();
		}
	}

	impl WidgetImpl for ClockPopup {}
	impl PopoverImpl for ClockPopup {}
}
