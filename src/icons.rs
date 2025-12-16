use const_format::{Case, concatcp, map_ascii_case};
use gtk4::gio;

pub fn register_bundled_icons() {
	let res_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/lucide.gresource"));
	let resource = gio::Resource::from_data(&res_bytes.into()).unwrap();
	gio::resources_register(&resource);

	let display = gtk4::gdk::Display::default().unwrap();
	let theme = gtk4::IconTheme::for_display(&display);

	theme.add_resource_path("/de/icytv/niribar/icons");
}

include!(concat!(env!("OUT_DIR"), "/icons.rs"));
