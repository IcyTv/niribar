use std::sync::{Arc, Mutex};

use astal4::prelude::*;
use gtk4::gdk;
use gtk4::prelude::{BoxExt as _, *};
use sysinfo::System;

mod bluetooth;
mod clock;
mod mediaplayer;
mod network;
mod overview;
mod taskbar;
mod volume;

pub struct Bar {
	pub window: astal4::Window,
	center_box: gtk4::CenterBox,
	system:     Arc<Mutex<System>>,
}

impl Bar {
	pub fn new(monitor_index: i32, monitor_width: i32, args: &super::Args) -> Self {
		let overview = overview::Overview::new(args);
		let taskbar = taskbar::Taskbar::new(monitor_index);
		let start_child = gtk4::Box::builder()
			.hexpand(true)
			.orientation(gtk4::Orientation::Horizontal)
			.spacing(8)
			.build();

		start_child.append(overview.widget());
		start_child.append(taskbar.widget());

		// let clock = clock::Clock::new();

		let mediaplayer = mediaplayer::MediaPlayerWidget::new();

		let volume = volume::Volume::new();
		let network = network::Network::new();
		let bluetooth = bluetooth::Bluetooth::new();
		let clock = clock::Clock::new();

		let end_box = gtk4::Box::builder()
			.hexpand(true)
			.orientation(gtk4::Orientation::Horizontal)
			.halign(gtk4::Align::End)
			.spacing(8)
			.build();

		end_box.append(volume.widget());
		end_box.append(network.widget());
		end_box.append(bluetooth.widget());
		end_box.append(clock.widget());

		let center_box = gtk4::CenterBox::builder()
			.start_widget(&start_child)
			.center_widget(&mediaplayer)
			.end_widget(&end_box)
			.css_classes(["bar"])
			.build();

		let window = astal4::Window::builder()
			.layer(astal4::Layer::Top)
			.anchor(astal4::WindowAnchor::TOP)
			.exclusivity(astal4::Exclusivity::Exclusive)
			.child(&center_box)
			.keymode(astal4::Keymode::None)
			.name("bar")
			.css_classes(["bar"])
			.monitor(monitor_index)
			.width_request(monitor_width)
			.build();

		Self {
			window,
			center_box,
			system: Arc::new(Mutex::new(System::new_all())),
		}
	}

	pub fn for_all_monitors(display: &gtk4::gdk::Display, args: &super::Args) -> Vec<Self> {
		display
			.monitors()
			.iter::<gdk::Monitor>()
			.map(|li| li.unwrap())
			.enumerate()
			.map(|(idx, monitor)| {
				let width = monitor.geometry().width();
				Bar::new(idx as i32, width, args)
			})
			.collect()
	}
}
