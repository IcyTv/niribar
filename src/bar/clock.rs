use astal_io::Time;
use astal_io::prelude::TimeExt;
use gtk4::glib::object::Cast;
use gtk4::glib::{DateTime, SignalHandlerId, clone};
use gtk4::prelude::BoxExt;

use crate::icons;

pub struct Clock {
	container:      gtk4::Box,
	timer:          Time,
	signal_handler: Option<SignalHandlerId>,
}

impl Clock {
	pub fn new() -> Self {
		let container = gtk4::Box::builder()
			.name("clock")
			.spacing(6)
			.orientation(gtk4::Orientation::Horizontal)
			.build();
		let calendar_icon = gtk4::Image::from_icon_name(icons::Icon::Calendar.name());
		let date_label = gtk4::Label::builder().build();
		let clock_icon = gtk4::Image::from_icon_name(icons::Icon::Clock.name());
		let time_label = gtk4::Label::builder().build();

		container.append(&calendar_icon);
		container.append(&date_label);
		container.append(&clock_icon);
		container.append(&time_label);
		let timer = Time::interval(1000, None);

		let handler = timer.connect_now(clone!(
			#[weak]
			date_label,
			move |_| {
				let time = match DateTime::now_local() {
					Ok(time) => time,
					Err(e) => {
						eprintln!("Error {e}");
						return;
					}
				};

				let date = match time.format("%a %b %d") {
					Ok(time) => time,
					Err(e) => {
						eprintln!("Error {e}");
						return;
					}
				};

				date_label.set_markup(&date);

				let time = match time.format("%X") {
					Ok(time) => time,
					Err(e) => {
						eprintln!("Error {e}");
						return;
					}
				};

				time_label.set_markup(&time);
			}
		));

		Self {
			container,
			timer,
			signal_handler: Some(handler),
		}
	}

	pub fn widget(&self) -> &gtk4::Widget {
		self.container.upcast_ref()
	}
}
