use std::cell::RefCell;

use astal_io::Time;
use astal_io::prelude::*;
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
	pub fn new(time: &Time) -> Self {
		glib::Object::builder().property("timer", time).build()
	}
}

mod imp {

	use glib::DateTime;

	use super::*;

	#[derive(Default, Properties, CompositeTemplate)]
	#[template(file = "./src/popups/clock/clock.blp")]
	#[properties(wrapper_type = super::ClockPopup)]
	pub struct ClockPopup {
		#[property(get, set)]
		current_time: RefCell<String>,
		#[property(get, construct_only)]
		timer: RefCell<Time>,
	}

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

			let obj = self.obj();

			let timer = &*self.timer.borrow();
			timer.connect_now(clone!(
				#[weak]
				obj,
				move |_| {
					let Ok(time) = DateTime::now_local() else { return };
					let Ok(formatted) = time.format("%T") else { return };

					obj.set_current_time(formatted);
				}
			));
		}
	}

	impl WidgetImpl for ClockPopup {}
	impl PopoverImpl for ClockPopup {}
}
