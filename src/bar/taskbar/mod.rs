mod niri;
mod widgets;

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use async_channel::Sender;
use futures::{Stream, StreamExt};
use glib::clone;
use glib::object::Cast;
use gtk4::prelude::*;
use gtk4::{gdk, gio};

use crate::bar::taskbar::widgets::{NiriWindowWidget, NiriWorkspaceWidget};

pub struct Taskbar {
	widget: gtk4::ListView,
}

impl Taskbar {
	pub fn new(monitor_index: i32) -> Self {
		let window_factory = create_window_factory();
		let header_factory = create_workspace_index_factory();

		let (full_sorter, workspace_index_sorter) = create_sorters();

		let store = gio::ListStore::new::<NiriWindowWidget>();

		let sort_model = gtk4::SortListModel::new(Some(store.clone()), Some(full_sorter));
		sort_model.set_section_sorter(Some(&workspace_index_sorter));

		let selection_model = gtk4::NoSelection::new(Some(sort_model));

		let widget = gtk4::ListView::builder()
			.name("taskbar")
			.orientation(gtk4::Orientation::Horizontal)
			.model(&selection_model)
			.factory(&window_factory)
			.header_factory(&header_factory)
			.css_classes(vec!["taskbar"])
			.build();

		let niri = niri::Niri::new();

		glib::spawn_future_local(clone!(
			#[weak]
			store,
			async move {
				let mut window_widgets = BTreeMap::<u64, NiriWindowWidget>::new();

				let monitor = gdk::Display::default()
					.expect("to have a default display")
					.monitors()
					.into_iter()
					.enumerate()
					.find_map(|(i, m)| if i as i32 == monitor_index { Some(m) } else { None })
					.unwrap()
					.unwrap()
					.downcast::<gdk::Monitor>()
					.unwrap();

				let output_filter = Self::build_output_filter(niri, &monitor).await;
				let mut event_stream = Box::pin(event_stream(niri));

				while let Some(event) = event_stream.next().await {
					match event {
						Event::Snapshot(windows) => {
							let mut omitted = window_widgets.keys().copied().collect::<BTreeSet<_>>();

							// TODO: Filter
							let window_iter = windows.iter().filter(|w| output_filter(w));

							for window in window_iter {
								let widget = match window_widgets.entry(window.id) {
									Entry::Occupied(entry) => entry.into_mut(),
									Entry::Vacant(entry) => {
										// TODO: Just pass window instead
										let widget = NiriWindowWidget::from_window(
											window.workspace_idx(),
											window.workspace_id(),
											window,
										);
										store.append(&widget);
										entry.insert(widget)
									}
								};

								if window.is_focused && !widget.has_css_class("focused") {
									widget.add_css_class("focused");
								} else if !window.is_focused && widget.has_css_class("focused") {
									widget.remove_css_class("focused");
								}

								omitted.remove(&window.id);
							}

							for id in omitted {
								if let Some(button) = window_widgets.remove(&id)
									&& let Some(index) = store.find(&button)
								{
									store.remove(index);
								}
							}
						}
					}
				}
			}
		));

		Self { widget }
	}

	pub fn widget(&self) -> &gtk4::Widget {
		self.widget.upcast_ref()
	}

	async fn build_output_filter(niri: niri::Niri, monitor: &gtk4::gdk::Monitor) -> Box<dyn Fn(&niri::Window) -> bool> {
		let outputs = match gio::spawn_blocking(move || niri.outputs()).await {
			Ok(outputs) => outputs,
			Err(_e) => {
				eprintln!("Failed to get outputs from Niri");
				return Box::new(|_| true);
			}
		};

		if outputs.is_empty() {
			return Box::new(|_| true);
		}

		for (name, output) in outputs {
			if monitor
				.connector()
				.is_some_and(|connector| connector.as_str() == name.as_str())
			{
				return Box::new(move |window: &niri::Window| {
					window
						.output()
						.as_ref()
						.is_some_and(|win_output| win_output == &output.name)
				});
			}
		}

		Box::new(|_| true)
	}
}

enum Event {
	Snapshot(Vec<niri::Window>),
	// Workspace(Vec<Workspace>),
}

async fn window_stream(tx: Sender<Event>, window_stream: niri::WindowStream) {
	while let Some(snap) = window_stream.next().await {
		if let Err(e) = tx.send(Event::Snapshot(snap)).await {
			eprintln!("Failed to send window snapshot: {e}");
		}
	}
}

// async fn workspace_stream(tx: Sender<Event>, stream: impl Stream<Item = Vec<Workspace>>) {
// 	let mut workspace_stream = Box::pin(stream);
// 	while let Some(workspaces) = workspace_stream.next().await {
// 		if let Err(e) = tx.send(Event::Workspace(workspaces)).await {
// 			eprintln!("Failed to send workspace update: {e}");
// 		}
// 	}
// }

fn event_stream(niri: niri::Niri) -> impl Stream<Item = Event> + use<> {
	let (tx, rx) = async_channel::unbounded();

	glib::spawn_future_local(window_stream(tx.clone(), niri.window_stream()));

	// let mut delay = Some((tx, niri.workspace_stream()));

	async_stream::stream! {
		while let Ok(event) = rx.recv().await {
			// if let Some((tx, stream)) = delay.take() {
			// 	if let &Event::Workspace(_) = &event {
			// 		glib::spawn_future_local(workspace_stream(tx, stream));
			// 	}
			// }

			yield event;
		}
	}
}

fn create_workspace_index_factory() -> gtk4::SignalListItemFactory {
	let factory = gtk4::SignalListItemFactory::new();

	factory.connect_setup(|_, item| {
		let header = item.downcast_ref::<gtk4::ListHeader>().unwrap();

		let workspace_widget = NiriWorkspaceWidget::new_null();

		header.set_child(Some(&workspace_widget));
	});

	factory.connect_bind(|_, item| {
		let header = item.downcast_ref::<gtk4::ListHeader>().unwrap();
		let workspace_widget = header.child().unwrap().downcast::<NiriWorkspaceWidget>().unwrap();

		if let Some(obj) = header.item().and_then(|o| o.downcast::<NiriWindowWidget>().ok()) {
			let workspace_id = obj.workspace_id();
			let workspace_idx = obj.workspace_index();
			workspace_widget.set_workspace_id(workspace_id);
			workspace_widget.set_workspace_index(workspace_idx);
		}
	});

	factory.connect_unbind(|_, item| {
		if let Some(header) = item.downcast_ref::<gtk4::ListHeader>()
			&& let Some(workspace_widget) = header.child().and_then(|c| c.downcast::<NiriWorkspaceWidget>().ok())
		{
			workspace_widget.set_workspace_id(0);
			workspace_widget.set_workspace_index(0);
		}
	});

	factory.connect_teardown(|_, item| {
		if let Some(header) = item.downcast_ref::<gtk4::ListHeader>() {
			header.set_child(None::<&gtk4::Widget>);
		}
	});

	factory
}

fn create_window_factory() -> gtk4::SignalListItemFactory {
	let factory = gtk4::SignalListItemFactory::new();

	factory.connect_setup(|_, li| {
		let li = li.downcast_ref::<gtk4::ListItem>().expect("to be a ListItem");

		li.set_child(Some(&gtk4::Box::new(gtk4::Orientation::Horizontal, 0)));
	});
	factory.connect_bind(|_, li| {
		let list_item = li.downcast_ref::<gtk4::ListItem>().expect("Needs to be a ListItem");

		let item = list_item.item().and_downcast::<NiriWindowWidget>().unwrap();
		list_item.set_child(Some(&item));
	});
	factory.connect_unbind(|_, li| {
		let list_item = li.downcast_ref::<gtk4::ListItem>().expect("Needs to be a ListItem");
		list_item.set_child(None::<&gtk4::Widget>);
	});
	factory
}

fn create_sorters() -> (gtk4::MultiSorter, gtk4::NumericSorter) {
	let workspace_index_sorter = gtk4::NumericSorter::builder()
		.expression(gtk4::PropertyExpression::new(
			NiriWindowWidget::static_type(),
			gtk4::Expression::NONE,
			"workspace-index",
		))
		.build();

	let positional_sorter = gtk4::NumericSorter::builder()
		.expression(gtk4::PropertyExpression::new(
			NiriWindowWidget::static_type(),
			gtk4::Expression::NONE,
			"sort-key",
		))
		.build();

	let full_sorter = gtk4::MultiSorter::new();
	full_sorter.append(workspace_index_sorter.clone());
	full_sorter.append(positional_sorter);

	(full_sorter, workspace_index_sorter)
}
