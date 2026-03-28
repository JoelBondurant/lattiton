use iced::widget::{container, text};
use iced::{Element, Length, Rectangle, Task};

use crate::colors;
use crate::lattiton::{
	self, Axis, InternalMessage, Lattiton, PaneId, State,
};

#[derive(Debug, Clone)]
pub enum Message {
	Lattiton(InternalMessage),
}

pub struct App {
	state: State,
	panes: Vec<(PaneId, String)>,
}

pub fn boot() -> (App, Task<Message>) {
	let (mut state, first) = State::with_initial_pane();

	let (_, second) = state.split(Axis::Horizontal, first).unwrap();
	let (_, third) = state.split(Axis::Vertical, first).unwrap();
	let (_, fourth) = state.split(Axis::Vertical, second).unwrap();

	let panes = vec![
		(first, "Code Editor".to_string()),
		(second, "Data Table".to_string()),
		(third, "Plot View".to_string()),
		(fourth, "Settings".to_string()),
	];

	(App { state, panes }, Task::none())
}

pub fn update(app: &mut App, message: Message) -> Task<Message> {
	match message {
		Message::Lattiton(msg) => {
			let bounds = Rectangle {
				x: 0.0,
				y: 0.0,
				width: 1200.0,
				height: 800.0,
			};
			lattiton::update(&mut app.state, msg, bounds);
		}
	}
	Task::none()
}

pub fn view(app: &App) -> Element<'_, Message> {
	let content: Vec<(PaneId, Element<Message>)> = app
		.panes
		.iter()
		.map(|(id, label)| {
			let elem = container(
				text(label.as_str())
					.size(16)
					.color(colors::TEXT_PRIMARY),
			)
			.width(Length::Fill)
			.height(Length::Fill)
			.center_x(Length::Fill)
			.center_y(Length::Fill)
			.style(|_theme| container::Style {
				background: Some(colors::BG_SECONDARY.into()),
				..Default::default()
			})
			.into();
			(*id, elem)
		})
		.collect();

	Lattiton::new(&app.state, content, Message::Lattiton).into()
}
