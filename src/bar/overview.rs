use glib::Bytes;
use gtk4::Image;
use gtk4::gdk::Texture;
use gtk4::prelude::*;

const NIXOS_ICON: &[u8] = include_bytes!("./NixOS.png");

pub struct Overview {
	widget: gtk4::Button,
}

impl Overview {
	pub fn new() -> Self {
		let bytes = Bytes::from(NIXOS_ICON);
		let texture = Texture::from_bytes(&bytes).unwrap();
		let image = Image::from_paintable(Some(&texture));
		image.set_pixel_size(32);

		let button = gtk4::Button::builder()
			.width_request(32)
			.height_request(32)
			.child(&image)
			.build();

		Self { widget: button }
	}

	pub fn widget(&self) -> &gtk4::Widget {
		self.widget.upcast_ref()
	}
}
