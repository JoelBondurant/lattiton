use iced::advanced::layout::{self, Layout, Node};
use iced::advanced::renderer;
use iced::advanced::text;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Point, Rectangle, Size};

use crate::lattiton::handle::{
	self, HandleAction, HandleZone,
	collapsed_strip_thickness, handle_thickness,
};
use crate::lattiton::state::{
	Axis, CollapseState, MaximizeState, NodeId, PaneId, SplitId, State,
};
use crate::lattiton::style::Style;

#[derive(Debug, Clone)]
pub enum InternalMessage {
	DragStarted(SplitId),
	DragMoved(Point),
	DragEnded,
	CollapseFirst(SplitId),
	CollapseSecond(SplitId),
	Expand(SplitId),
	Maximize(PaneId),
}

pub struct Lattiton<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
	state: &'a State,
	content: Vec<(PaneId, Element<'a, Message, Theme, Renderer>)>,
	style: Style,
	on_message: Box<dyn Fn(InternalMessage) -> Message + 'a>,
}

impl<'a, Message, Theme, Renderer> Lattiton<'a, Message, Theme, Renderer>
where
	Renderer: renderer::Renderer + text::Renderer<Font = iced::Font>,
{
	pub fn new(
		state: &'a State,
		content: Vec<(PaneId, Element<'a, Message, Theme, Renderer>)>,
		on_message: impl Fn(InternalMessage) -> Message + 'a,
	) -> Self {
		Self {
			state,
			content,
			style: Style::default(),
			on_message: Box::new(on_message),
		}
	}

	pub fn style(mut self, style: Style) -> Self {
		self.style = style;
		self
	}

	fn find_content(&self, pane: PaneId) -> Option<usize> {
		self.content.iter().position(|(id, _)| *id == pane)
	}
}

#[derive(Default)]
struct WidgetState {
	handle_zones: Vec<HandleZone>,
	pane_bounds: Vec<(PaneId, Rectangle)>,
	hovered_arrow: Option<(SplitId, bool)>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
	for Lattiton<'_, Message, Theme, Renderer>
where
	Renderer: renderer::Renderer + text::Renderer<Font = iced::Font>,
{
	fn size(&self) -> Size<Length> {
		Size::new(Length::Fill, Length::Fill)
	}

	fn tag(&self) -> widget::tree::Tag {
		widget::tree::Tag::of::<WidgetState>()
	}

	fn state(&self) -> widget::tree::State {
		widget::tree::State::new(WidgetState::default())
	}

	fn children(&self) -> Vec<widget::Tree> {
		self.content.iter().map(|(_, elem)| Tree::new(elem)).collect()
	}

	fn diff(&self, tree: &mut Tree) {
		tree.diff_children(
			&self.content.iter().map(|(_, e)| e).collect::<Vec<_>>(),
		);
	}

	fn layout(
		&mut self,
		tree: &mut Tree,
		renderer: &Renderer,
		limits: &layout::Limits,
	) -> Node {
		let size = limits.max();

		// Build layout into temporary vecs, then write to widget state after
		let mut handle_zones = Vec::new();
		let mut pane_bounds = Vec::new();

		let children = match self.state.maximize() {
			MaximizeState::Maximized(pane) => {
				let mut children = Vec::new();
				if let Some(idx) = self.find_content(pane) {
					let child_node = self.content[idx].1.as_widget_mut().layout(
						&mut tree.children[idx],
						renderer,
						&layout::Limits::new(Size::ZERO, size),
					);
					pane_bounds.push((pane, Rectangle::new(Point::ORIGIN, size)));
					children.push(child_node);
				}
				children
			}
			MaximizeState::None => {
				let mut children = Vec::new();
				if let Some(root) = self.state.root() {
					layout_node(
						&mut self.content,
						self.state,
						&self.style,
						&mut tree.children,
						renderer,
						root,
						Rectangle::new(Point::ORIGIN, size),
						&mut handle_zones,
						&mut pane_bounds,
						&mut children,
					);
				}
				children
			}
		};

		let widget_state = tree.state.downcast_mut::<WidgetState>();
		widget_state.handle_zones = handle_zones;
		widget_state.pane_bounds = pane_bounds;

		Node::with_children(size, children)
	}

	fn draw(
		&self,
		tree: &Tree,
		renderer: &mut Renderer,
		theme: &Theme,
		style: &renderer::Style,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		viewport: &Rectangle,
	) {
		let widget_state = tree.state.downcast_ref::<WidgetState>();

		// Draw pane backgrounds
		for &(_, bounds) in &widget_state.pane_bounds {
			renderer.fill_quad(
				renderer::Quad {
					bounds,
					border: iced::Border {
						color: self.style.pane.border_color,
						width: self.style.pane.border_width,
						radius: 0.0.into(),
					},
					..renderer::Quad::default()
				},
				self.style.pane.background,
			);
		}

		// Draw child content
		for (child_idx, child_layout) in layout.children().enumerate() {
			if child_idx < self.content.len() {
				self.content[child_idx].1.as_widget().draw(
					&tree.children[child_idx],
					renderer,
					theme,
					style,
					child_layout,
					cursor,
					viewport,
				);
			}
		}

		// Draw handles on top
		for zone in &widget_state.handle_zones {
			let hovered = widget_state.hovered_arrow
				.filter(|(sid, _)| *sid == zone.split_id)
				.map(|(_, is_first)| is_first);
			handle::draw_handle(renderer, zone, &self.style.handle, hovered);
		}
	}

	fn update(
		&mut self,
		tree: &mut Tree,
		event: &Event,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		renderer: &Renderer,
		clipboard: &mut dyn Clipboard,
		shell: &mut Shell<'_, Message>,
		viewport: &Rectangle,
	) {
		let widget_state = tree.state.downcast_mut::<WidgetState>();

		// Forward events to children first
		for (child_idx, child_layout) in layout.children().enumerate() {
			if child_idx < self.content.len() {
				self.content[child_idx].1.as_widget_mut().update(
					&mut tree.children[child_idx],
					event,
					child_layout,
					cursor,
					renderer,
					clipboard,
					shell,
					viewport,
				);
			}
		}

		match event {
			Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
				if let Some(pos) = cursor.position() {
					// Check arrow clicks first
					for zone in &widget_state.handle_zones {
						if let Some(action) = zone.hit_test(pos) {
							let msg = match action {
								HandleAction::CollapseFirst(id) => {
									InternalMessage::CollapseFirst(id)
								}
								HandleAction::CollapseSecond(id) => {
									InternalMessage::CollapseSecond(id)
								}
								HandleAction::Expand(id) => {
									InternalMessage::Expand(id)
								}
							};
							shell.publish((self.on_message)(msg));
							return;
						}
					}
					// Check handle drag
					for zone in &widget_state.handle_zones {
						if zone.contains(pos) {
							shell.publish((self.on_message)(
								InternalMessage::DragStarted(zone.split_id),
							));
							return;
						}
					}
				}
			}
			Event::Mouse(mouse::Event::CursorMoved { .. }) => {
				if let Some(pos) = cursor.position() {
					if self.state.dragging().is_some() {
						shell.publish((self.on_message)(
							InternalMessage::DragMoved(pos),
						));
						return;
					}
					// Update hover state for arrows
					let mut new_hover = None;
					for zone in &widget_state.handle_zones {
						if zone.first_arrow.contains(pos) {
							new_hover = Some((zone.split_id, true));
							break;
						}
						if zone.second_arrow.contains(pos) {
							new_hover = Some((zone.split_id, false));
							break;
						}
					}
					widget_state.hovered_arrow = new_hover;
				}
			}
			Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
				if self.state.dragging().is_some() {
					shell.publish((self.on_message)(InternalMessage::DragEnded));
				}
			}
			_ => {}
		}
	}

	fn mouse_interaction(
		&self,
		tree: &Tree,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		viewport: &Rectangle,
		renderer: &Renderer,
	) -> mouse::Interaction {
		let widget_state = tree.state.downcast_ref::<WidgetState>();

		if self.state.dragging().is_some() {
			return mouse::Interaction::Grabbing;
		}

		if let Some(pos) = cursor.position() {
			for zone in &widget_state.handle_zones {
				if zone.first_arrow.contains(pos) || zone.second_arrow.contains(pos) {
					return mouse::Interaction::Pointer;
				}
				if zone.contains(pos) {
					return match zone.axis {
						Axis::Horizontal => mouse::Interaction::ResizingHorizontally,
						Axis::Vertical => mouse::Interaction::ResizingVertically,
					};
				}
			}
		}

		// Check children
		for (child_idx, child_layout) in layout.children().enumerate() {
			if child_idx < self.content.len() {
				let interaction = self.content[child_idx].1.as_widget().mouse_interaction(
					&tree.children[child_idx],
					child_layout,
					cursor,
					viewport,
					renderer,
				);
				if interaction != mouse::Interaction::None {
					return interaction;
				}
			}
		}

		mouse::Interaction::None
	}
}

/// Recursive layout helper — free function to avoid borrow conflicts on `self`.
fn layout_node<Message, Theme, Renderer>(
	content: &mut [(PaneId, Element<'_, Message, Theme, Renderer>)],
	state: &State,
	style: &Style,
	tree_children: &mut [Tree],
	renderer: &Renderer,
	node: NodeId,
	region: Rectangle,
	handle_zones: &mut Vec<HandleZone>,
	pane_bounds: &mut Vec<(PaneId, Rectangle)>,
	children: &mut Vec<Node>,
) where
	Renderer: renderer::Renderer + text::Renderer<Font = iced::Font>,
{
	match node {
		NodeId::Pane(pane) => {
			let idx = content.iter().position(|(id, _)| *id == pane);
			if let Some(idx) = idx {
				let limits = layout::Limits::new(
					Size::ZERO,
					Size::new(region.width, region.height),
				);
				let child_node = content[idx].1.as_widget_mut().layout(
					&mut tree_children[idx],
					renderer,
					&limits,
				);
				let child_node = child_node.move_to(Point::new(region.x, region.y));
				children.push(child_node);
				pane_bounds.push((pane, region));
			}
		}
		NodeId::Split(split_id) => {
			if let Some(split) = state.get_split(split_id) {
				let axis = split.axis;
				let ratio = split.ratio;
				let collapse = split.collapse;
				let ht = handle_thickness(&style.handle);

				let (first_region, handle_region, second_region) =
					split_regions(region, axis, ratio, collapse, ht);

				let first = split.first;
				let second = split.second;

				if collapse != CollapseState::FirstCollapsed {
					layout_node(
						content, state, style, tree_children, renderer,
						first, first_region, handle_zones, pane_bounds, children,
					);
				}

				if collapse != CollapseState::SecondCollapsed {
					layout_node(
						content, state, style, tree_children, renderer,
						second, second_region, handle_zones, pane_bounds, children,
					);
				}

				handle_zones.push(HandleZone::new(
					split_id,
					handle_region,
					axis,
					collapse,
				));
			}
		}
	}
}

fn split_regions(
	region: Rectangle,
	axis: Axis,
	ratio: f32,
	collapse: CollapseState,
	handle_thickness: f32,
) -> (Rectangle, Rectangle, Rectangle) {
	let strip = collapsed_strip_thickness();

	match collapse {
		CollapseState::FirstCollapsed => {
			let handle = match axis {
				Axis::Horizontal => Rectangle {
					x: region.x,
					y: region.y,
					width: strip,
					height: region.height,
				},
				Axis::Vertical => Rectangle {
					x: region.x,
					y: region.y,
					width: region.width,
					height: strip,
				},
			};
			let first = Rectangle {
				x: region.x,
				y: region.y,
				width: 0.0,
				height: 0.0,
			};
			let second = match axis {
				Axis::Horizontal => Rectangle {
					x: region.x + strip,
					y: region.y,
					width: region.width - strip,
					height: region.height,
				},
				Axis::Vertical => Rectangle {
					x: region.x,
					y: region.y + strip,
					width: region.width,
					height: region.height - strip,
				},
			};
			(first, handle, second)
		}
		CollapseState::SecondCollapsed => {
			let second = Rectangle {
				x: region.x,
				y: region.y,
				width: 0.0,
				height: 0.0,
			};
			let (handle, first) = match axis {
				Axis::Horizontal => (
					Rectangle {
						x: region.x + region.width - strip,
						y: region.y,
						width: strip,
						height: region.height,
					},
					Rectangle {
						x: region.x,
						y: region.y,
						width: region.width - strip,
						height: region.height,
					},
				),
				Axis::Vertical => (
					Rectangle {
						x: region.x,
						y: region.y + region.height - strip,
						width: region.width,
						height: strip,
					},
					Rectangle {
						x: region.x,
						y: region.y,
						width: region.width,
						height: region.height - strip,
					},
				),
			};
			(first, handle, second)
		}
		CollapseState::Expanded => {
			match axis {
				Axis::Horizontal => {
					let first_w = (region.width - handle_thickness) * ratio;
					let first = Rectangle {
						x: region.x,
						y: region.y,
						width: first_w,
						height: region.height,
					};
					let handle = Rectangle {
						x: region.x + first_w,
						y: region.y,
						width: handle_thickness,
						height: region.height,
					};
					let second = Rectangle {
						x: region.x + first_w + handle_thickness,
						y: region.y,
						width: region.width - first_w - handle_thickness,
						height: region.height,
					};
					(first, handle, second)
				}
				Axis::Vertical => {
					let first_h = (region.height - handle_thickness) * ratio;
					let first = Rectangle {
						x: region.x,
						y: region.y,
						width: region.width,
						height: first_h,
					};
					let handle = Rectangle {
						x: region.x,
						y: region.y + first_h,
						width: region.width,
						height: handle_thickness,
					};
					let second = Rectangle {
						x: region.x,
						y: region.y + first_h + handle_thickness,
						width: region.width,
						height: region.height - first_h - handle_thickness,
					};
					(first, handle, second)
				}
			}
		}
	}
}

impl<'a, Message, Theme, Renderer> From<Lattiton<'a, Message, Theme, Renderer>>
	for Element<'a, Message, Theme, Renderer>
where
	Message: 'a,
	Theme: 'a,
	Renderer: renderer::Renderer + text::Renderer<Font = iced::Font> + 'a,
{
	fn from(lattiton: Lattiton<'a, Message, Theme, Renderer>) -> Self {
		Element::new(lattiton)
	}
}

/// Process an InternalMessage by updating the State.
/// Call this from your app's update function.
pub fn update(state: &mut State, message: InternalMessage, bounds: Rectangle) {
	match message {
		InternalMessage::DragStarted(split_id) => {
			state.set_dragging(Some(split_id));
		}
		InternalMessage::DragMoved(pos) => {
			if let Some(split_id) = state.dragging() {
				if let Some(split) = state.get_split(split_id) {
					let axis = split.axis;
					let ratio = match axis {
						Axis::Horizontal => {
							(pos.x - bounds.x) / bounds.width
						}
						Axis::Vertical => {
							(pos.y - bounds.y) / bounds.height
						}
					};
					state.resize(split_id, ratio);
				}
			}
		}
		InternalMessage::DragEnded => {
			state.set_dragging(None);
		}
		InternalMessage::CollapseFirst(split_id) => {
			state.collapse_first(split_id);
		}
		InternalMessage::CollapseSecond(split_id) => {
			state.collapse_second(split_id);
		}
		InternalMessage::Expand(split_id) => {
			state.expand(split_id);
		}
		InternalMessage::Maximize(pane) => {
			state.toggle_maximize(pane);
		}
	}
}
