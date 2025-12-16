use std::cell::RefCell;
use std::rc::Rc;

use astal_mpris::prelude::{MprisExt, PlayerExt};
use astal_mpris::{Mpris, PlaybackStatus, Player};
use glib::value::FromValue;
use glib::{Properties, clone};
use gtk4::CompositeTemplate;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{EventControllerMotion, glib};
use lazy_regex::regex;

use crate::icons;

glib::wrapper! {
	pub struct MediaPlayerWidget(ObjectSubclass<media_player_imp::MediaPlayerWidget>)
		@extends gtk4::Box, gtk4::Widget,
		@implements gtk4::Accessible, gtk4::Orientable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl MediaPlayerWidget {
	pub fn new() -> Self {
		glib::Object::builder()
			.property("switcher-icon-name", icons::Icon::ArrowUpDown.name())
			.build()
	}
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
	#[default]
	PlayerIcon,
	SwitcherIcon,
}

impl glib::enums::EnumerationValue<ButtonState> for ButtonState {
	type GlibType = glib::GString;

	const ZERO: Self = Self::PlayerIcon;
}

impl glib::property::Property for ButtonState {
	type Value = glib::GString;
}

impl ToValue for ButtonState {
	fn to_value(&self) -> glib::Value {
		let s = match self {
			ButtonState::PlayerIcon => "player-icon",
			ButtonState::SwitcherIcon => "switcher-icon",
		};
		s.to_value()
	}

	fn value_type(&self) -> glib::Type {
		glib::GString::static_type()
	}
}

impl StaticType for ButtonState {
	fn static_type() -> glib::Type {
		glib::GString::static_type()
	}
}

unsafe impl<'a> FromValue<'a> for ButtonState {
	type Checker = glib::value::GenericValueTypeChecker<Self>;

	unsafe fn from_value(value: &'a glib::Value) -> Self {
		let s = value
			.get::<glib::GString>()
			.expect("ButtonState should be convertible from GString");

		match s.as_str() {
			"player-icon" => ButtonState::PlayerIcon,
			"switcher-icon" => ButtonState::SwitcherIcon,
			_ => ButtonState::PlayerIcon,
		}
	}
}

impl std::borrow::Borrow<str> for ButtonState {
	fn borrow(&self) -> &str {
		match self {
			ButtonState::PlayerIcon => "player-icon",
			ButtonState::SwitcherIcon => "switcher-icon",
		}
	}
}

mod media_player_imp {
	use super::*;

	#[derive(Properties, Default, CompositeTemplate)]
	#[template(file = "./src/bar/mediaplayer.blp")]
	#[properties(wrapper_type = super::MediaPlayerWidget)]
	pub struct MediaPlayerWidget {
		#[property(get, set)]
		current_player: RefCell<u32>,
		players: RefCell<Vec<Player>>,

		#[property(get, set)]
		playing_title: RefCell<String>,

		#[property(get, set)]
		button_state: RefCell<ButtonState>,

		#[property(get, set)]
		switcher_icon_name: RefCell<String>,
		#[property(get, set)]
		player_icon: RefCell<String>,

		#[template_child]
		switcher_button_stack: TemplateChild<gtk4::Stack>,
		#[template_child]
		title_label: TemplateChild<gtk4::Label>,
	}

	#[glib::object_subclass]
	impl ObjectSubclass for MediaPlayerWidget {
		type ParentType = gtk4::Box;
		type Type = super::MediaPlayerWidget;

		const NAME: &'static str = "MediaPlayerWidget";

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
			Self::bind_template_callbacks(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	#[glib::derived_properties]
	impl ObjectImpl for MediaPlayerWidget {
		fn constructed(&self) {
			self.parent_constructed();

			let mpris = Mpris::default();
			let sorted_players = Rc::new(RefCell::new(Vec::<Player>::new()));

			let obj = self.obj();

			let hover_controller = EventControllerMotion::new();
			hover_controller.connect_enter(clone!(
				#[weak]
				obj,
				move |_, _, _| {
					obj.set_button_state(ButtonState::SwitcherIcon);
				}
			));
			hover_controller.connect_leave(clone!(
				#[weak]
				obj,
				move |_| {
					obj.set_button_state(ButtonState::PlayerIcon);
				}
			));
			self.switcher_button_stack.add_controller(hover_controller);

			// --- Bindings & Update Logic ---

			// let update_ui = clone!(
			// 	#[weak]
			// 	title_label,
			// 	#[weak]
			// 	obj,
			// 	#[strong]
			// 	current_index,
			// 	#[strong]
			// 	sorted_players,
			// 	#[strong]
			// 	current_playback_status_binding,
			// 	move |mpris: &Mpris| {
			// 		let mut new_players = valid_mpris_players(mpris);
			// 		new_players.sort_by_key(player_to_key);
			// 		*sorted_players.borrow_mut() = new_players;
			//
			// 		let mut index_mut = current_index.borrow_mut();
			// 		if *index_mut >= sorted_players.borrow().len() as u32 {
			// 			*index_mut = 0;
			// 		}
			// 		let index = *index_mut;
			// 		drop(index_mut);
			//
			// 		let players = sorted_players.borrow();
			// 		if let Some(player) = players.get(index as usize) {
			// 			if let Some(b) = current_title_binding.borrow_mut().take() {
			// 				b.unbind();
			// 			}
			// 			if let Some(b) = current_playback_status_binding.borrow_mut().take() {
			// 				b.unbind();
			// 			}
			//
			// 			let binding = player
			// 				.bind_property("title", &obj, "playing-title")
			// 				.sync_create()
			// 				.build();
			// 			*current_title_binding.borrow_mut() = Some(binding);
			//
			// 			let icon_name = player_icon_name(player);
			// 			obj.set_player_icon(icon_name);
			//
			// 			let playback_binding = player
			// 				.bind_property("playback-status", &title_label, "css-classes")
			// 				.transform_to(|binding, status: PlaybackStatus| -> Option<Vec<glib::GString>> {
			// 					let button = binding.target().and_downcast::<gtk4::Label>().unwrap();
			// 					let mut classes = button.css_classes();
			// 					classes.retain(|c| c.as_str() != "playing");
			// 					if status == PlaybackStatus::Playing {
			// 						classes.push("playing".into());
			// 					}
			// 					Some(classes)
			// 				})
			// 				.sync_create()
			// 				.build();
			// 			*current_playback_status_binding.borrow_mut() = Some(playback_binding);
			// 		} else {
			// 			title_label.set_label("No media playing");
			// 			obj.set_player_icon(icons::Icon::VolumeOff.name());
			// 			if let Some(b) = current_title_binding.borrow_mut().take() {
			// 				b.unbind();
			// 			}
			// 			if let Some(b) = current_playback_status_binding.borrow_mut().take() {
			// 				b.unbind();
			// 			}
			// 		}
			// 	}
			// );

			let players = &self.players;
			let update_players = clone!(
				#[strong]
				players,
				move |mpris: &Mpris| {
					let mut new_players = valid_mpris_players(mpris);
					new_players.sort_by_key(player_to_key);
					*players.borrow_mut() = new_players;
				}
			);

			update_players(&mpris);
			mpris.connect_players_notify(update_players);

			// self.connect_title_button_clicked(clone!(
			// 	#[strong]
			// 	sorted_players,
			// 	#[strong]
			// 	current_index,
			// 	move |_| {
			// 		let index = *current_index.borrow();
			// 		let players = sorted_players.borrow();
			// 		if let Some(player) = players.get(index) {
			// 			if player.can_pause() && player.playback_status() == PlaybackStatus::Playing {
			// 				player.pause();
			// 			} else if player.can_play() && player.playback_status() != PlaybackStatus::Playing {
			// 				player.play();
			// 			}
			// 		}
			// 	}
			// ));
		}
	}

	impl WidgetImpl for MediaPlayerWidget {}
	impl BoxImpl for MediaPlayerWidget {}

	impl MediaPlayerWidget {
		pub fn next_player(&self) {
			let mut current_player = *self.current_player.borrow();
			current_player += 1;
			// TODO store this information somewhere more convenient
			current_player %= valid_mpris_players(&Mpris::default()).len() as u32;

			*self.current_player.borrow_mut() = current_player;
		}
	}

	#[gtk4::template_callbacks]
	impl MediaPlayerWidget {
		#[template_callback]
		fn on_player_switch_clicked(&self) {
			self.next_player();
		}

		#[template_callback]
		fn on_title_button_clicked() {}
	}
}

fn valid_mpris_players(mpris: &Mpris) -> Vec<Player> {
	mpris
		.players()
		.into_iter()
		.filter(|p| player_to_key(p) < usize::MAX)
		.collect()
}

fn player_to_key(player: &Player) -> usize {
	match (
		player.bus_name().as_str(),
		player.title().as_str(),
		player.playback_status(),
	) {
		(_, _, PlaybackStatus::Playing) => 90,
		(_, "", _) => usize::MAX,
		(bn, ..) if bn.ends_with("spotify") => 100,
		(bn, ..) if regex!(r#"^org.mpris.MediaPlayer2.firefox.instance_.*$"#).is_match(&bn) => 200,
		_ => usize::MAX - 1000,
	}
}

fn player_icon_name(player: &Player) -> &'static str {
	match player.bus_name().as_str() {
		bn if bn.ends_with("spotify") => crate::icons::Icon::Spotify.name(),
		bn if regex!(r#"^org.mpris.MediaPlayer2.firefox.instance_.*$"#).is_match(&bn) => {
			crate::icons::Icon::Firefox.name()
		}
		_ => "audio-x-generic",
	}
}
