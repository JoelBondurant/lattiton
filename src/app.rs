use iced::widget::{container, text};
use iced::{Element, Length, Task};

use crate::colors;
use crate::lattiton::{
	self, Axis, ChromeVisibility, InternalMessage, Lattiton, PaneId, State,
	Style,
};

#[derive(Debug, Clone)]
pub enum Message {
	Outer(InternalMessage),
	PlotGrid(InternalMessage),
}

pub struct App {
	state: State,
	panes: Vec<(PaneId, String)>,
	plot_state: State,
	plot_panes: Vec<(PaneId, String)>,
	/// Which outer pane holds the plot dashboard
	plot_pane_id: PaneId,
}

pub fn boot() -> (App, Task<Message>) {
	let (mut state, first) = State::with_initial_pane();

	let (_, second) = state.split(Axis::Horizontal, first).unwrap();
	let (_, third) = state.split(Axis::Vertical, first).unwrap();
	let (_, fourth) = state.split(Axis::Vertical, second).unwrap();

	let panes = vec![
		(first, "Code Editor".to_string()),
		(second, "Data Table".to_string()),
		(third, "Plot Dashboard".to_string()),
		(fourth, "Settings".to_string()),
	];

	// Inner plot dashboard: 3 plots in an L-shape
	let (mut plot_state, p1) = State::with_initial_pane();
	let (_, p2) = plot_state.split(Axis::Horizontal, p1).unwrap();
	let (_, p3) = plot_state.split(Axis::Vertical, p1).unwrap();

	let plot_panes = vec![
		(p1, "Scatter Plot".to_string()),
		(p2, "Histogram".to_string()),
		(p3, "Time Series".to_string()),
	];

	(
		App {
			state,
			panes,
			plot_state,
			plot_panes,
			plot_pane_id: third,
		},
		Task::none(),
	)
}

pub fn update(app: &mut App, message: Message) -> Task<Message> {
	match message {
		Message::Outer(msg) => {
			lattiton::update(&mut app.state, msg);
		}
		Message::PlotGrid(msg) => {
			lattiton::update(&mut app.plot_state, msg);
		}
	}
	Task::none()
}

pub fn view(app: &App) -> Element<'_, Message> {
	let plot_style = Style {
		chrome: ChromeVisibility::OnHover,
		..Style::default()
	};

	let content: Vec<(PaneId, Element<Message>)> = app
		.panes
		.iter()
		.map(|(id, label)| {
			let elem: Element<Message> = if *id == app.plot_pane_id {
				// Nested plot dashboard
				let plot_content: Vec<(PaneId, Element<Message>)> = app
					.plot_panes
					.iter()
					.map(|(pid, plabel)| {
						let e = container(
							text(plabel.as_str())
								.size(14)
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
						(*pid, e)
					})
					.collect();

				Lattiton::new(&app.plot_state, plot_content, Message::PlotGrid)
					.style(plot_style)
					.into()
			} else {
				container(
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
				.into()
			};
			(*id, elem)
		})
		.collect();

	Lattiton::new(&app.state, content, Message::Outer).into()
}
