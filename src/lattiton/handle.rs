use iced::advanced::renderer;
use iced::{Point, Rectangle, Color};

use crate::lattiton::state::{Axis, CollapseState, SplitId};
use crate::lattiton::style::HandleStyle;

const DOT_RADIUS: f32 = 1.5;
const DOT_SPACING_ALONG: f32 = 5.0;
const DOT_SPACING_ACROSS: f32 = 4.0;
const ARROW_SIZE: f32 = 5.0;
const ARROW_MARGIN: f32 = 6.0;
const COLLAPSED_STRIP_THICKNESS: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleAction {
	CollapseFirst(SplitId),
	CollapseSecond(SplitId),
	Expand(SplitId),
}

#[derive(Debug, Clone, Copy)]
pub struct HandleZone {
	pub split_id: SplitId,
	pub bounds: Rectangle,
	pub axis: Axis,
	pub collapse: CollapseState,
	pub first_arrow: Rectangle,
	pub second_arrow: Rectangle,
}

impl HandleZone {
	pub fn new(
		split_id: SplitId,
		bounds: Rectangle,
		axis: Axis,
		collapse: CollapseState,
	) -> Self {
		let (first_arrow, second_arrow) = arrow_zones(bounds, axis);
		Self {
			split_id,
			bounds,
			axis,
			collapse,
			first_arrow,
			second_arrow,
		}
	}

	pub fn hit_test(&self, point: Point) -> Option<HandleAction> {
		if self.first_arrow.contains(point) {
			return Some(match self.collapse {
				CollapseState::FirstCollapsed => HandleAction::Expand(self.split_id),
				_ => HandleAction::CollapseFirst(self.split_id),
			});
		}
		if self.second_arrow.contains(point) {
			return Some(match self.collapse {
				CollapseState::SecondCollapsed => HandleAction::Expand(self.split_id),
				_ => HandleAction::CollapseSecond(self.split_id),
			});
		}
		None
	}

	pub fn contains(&self, point: Point) -> bool {
		self.bounds.contains(point)
	}
}

fn arrow_zones(bounds: Rectangle, axis: Axis) -> (Rectangle, Rectangle) {
	let arrow_size = ARROW_SIZE * 3.0;
	match axis {
		Axis::Horizontal => {
			let first = Rectangle {
				x: bounds.x + ARROW_MARGIN,
				y: bounds.y + (bounds.height - arrow_size) / 2.0,
				width: arrow_size,
				height: arrow_size,
			};
			let second = Rectangle {
				x: bounds.x + bounds.width - ARROW_MARGIN - arrow_size,
				y: bounds.y + (bounds.height - arrow_size) / 2.0,
				width: arrow_size,
				height: arrow_size,
			};
			(first, second)
		}
		Axis::Vertical => {
			let first = Rectangle {
				x: bounds.x + (bounds.width - arrow_size) / 2.0,
				y: bounds.y + ARROW_MARGIN,
				width: arrow_size,
				height: arrow_size,
			};
			let second = Rectangle {
				x: bounds.x + (bounds.width - arrow_size) / 2.0,
				y: bounds.y + bounds.height - ARROW_MARGIN - arrow_size,
				width: arrow_size,
				height: arrow_size,
			};
			(first, second)
		}
	}
}

pub fn draw_handle<Renderer>(
	renderer: &mut Renderer,
	zone: &HandleZone,
	style: &HandleStyle,
	hovered_arrow: Option<bool>,
) where
	Renderer: renderer::Renderer,
{
	let bounds = zone.bounds;

	// Handle background
	renderer.fill_quad(
		renderer::Quad {
			bounds,
			border: iced::Border {
				color: style.border_color,
				width: 0.5,
				radius: 0.0.into(),
			},
			..renderer::Quad::default()
		},
		style.background,
	);

	// 3x2 dots in the center
	draw_dots(renderer, bounds, zone.axis, style.dot_color);

	// Collapse/expand arrows
	let first_color = if hovered_arrow == Some(true) {
		style.arrow_hover_color
	} else {
		style.arrow_color
	};
	let second_color = if hovered_arrow == Some(false) {
		style.arrow_hover_color
	} else {
		style.arrow_color
	};

	draw_arrow(
		renderer,
		zone.first_arrow,
		zone.axis,
		zone.collapse,
		true,
		first_color,
	);
	draw_arrow(
		renderer,
		zone.second_arrow,
		zone.axis,
		zone.collapse,
		false,
		second_color,
	);
}

fn draw_dots<Renderer>(
	renderer: &mut Renderer,
	bounds: Rectangle,
	axis: Axis,
	color: Color,
) where
	Renderer: renderer::Renderer,
{
	let cx = bounds.x + bounds.width / 2.0;
	let cy = bounds.y + bounds.height / 2.0;

	// 3 dots along the axis, 2 dots across
	let along_offsets: [f32; 3] = [-DOT_SPACING_ALONG, 0.0, DOT_SPACING_ALONG];
	let across_offsets: [f32; 2] = [-DOT_SPACING_ACROSS / 2.0, DOT_SPACING_ACROSS / 2.0];

	for &along in &along_offsets {
		for &across in &across_offsets {
			let (x, y) = match axis {
				Axis::Horizontal => (cx + across, cy + along),
				Axis::Vertical => (cx + along, cy + across),
			};
			renderer.fill_quad(
				renderer::Quad {
					bounds: Rectangle {
						x: x - DOT_RADIUS,
						y: y - DOT_RADIUS,
						width: DOT_RADIUS * 2.0,
						height: DOT_RADIUS * 2.0,
					},
					border: iced::Border {
						color: Color::TRANSPARENT,
						width: 0.0,
						radius: DOT_RADIUS.into(),
					},
					..renderer::Quad::default()
				},
				color,
			);
		}
	}
}

fn draw_arrow<Renderer>(
	renderer: &mut Renderer,
	zone: Rectangle,
	axis: Axis,
	collapse: CollapseState,
	is_first: bool,
	color: Color,
) where
	Renderer: renderer::Renderer,
{
	let pointing_inward = match (is_first, collapse) {
		(true, CollapseState::FirstCollapsed) => false,
		(true, _) => true,
		(false, CollapseState::SecondCollapsed) => false,
		(false, _) => true,
	};

	let cx = zone.x + zone.width / 2.0;
	let cy = zone.y + zone.height / 2.0;
	let half = ARROW_SIZE / 2.0;

	match (axis, is_first) {
		(Axis::Horizontal, true) => {
			let dir = if pointing_inward { -1.0 } else { 1.0 };
			let tip_x = cx + dir * half;
			let base_x = cx - dir * half;
			draw_chevron_h(renderer, tip_x, base_x, cy, half, color);
		}
		(Axis::Horizontal, false) => {
			let dir = if pointing_inward { 1.0 } else { -1.0 };
			let tip_x = cx + dir * half;
			let base_x = cx - dir * half;
			draw_chevron_h(renderer, tip_x, base_x, cy, half, color);
		}
		(Axis::Vertical, true) => {
			let dir = if pointing_inward { -1.0 } else { 1.0 };
			let tip_y = cy + dir * half;
			let base_y = cy - dir * half;
			draw_chevron_v(renderer, cx, tip_y, base_y, half, color);
		}
		(Axis::Vertical, false) => {
			let dir = if pointing_inward { 1.0 } else { -1.0 };
			let tip_y = cy + dir * half;
			let base_y = cy - dir * half;
			draw_chevron_v(renderer, cx, tip_y, base_y, half, color);
		}
	}
}

fn draw_chevron_h<Renderer>(
	renderer: &mut Renderer,
	tip_x: f32,
	base_x: f32,
	cy: f32,
	half_h: f32,
	color: Color,
) where
	Renderer: renderer::Renderer,
{
	let thickness = 1.5;
	let mid_x = (tip_x + base_x) / 2.0;
	// Upper arm
	renderer.fill_quad(
		renderer::Quad {
			bounds: Rectangle {
				x: mid_x.min(tip_x),
				y: cy - half_h,
				width: (tip_x - mid_x).abs().max(thickness),
				height: thickness,
			},
			..renderer::Quad::default()
		},
		color,
	);
	// Lower arm
	renderer.fill_quad(
		renderer::Quad {
			bounds: Rectangle {
				x: mid_x.min(tip_x),
				y: cy + half_h - thickness,
				width: (tip_x - mid_x).abs().max(thickness),
				height: thickness,
			},
			..renderer::Quad::default()
		},
		color,
	);
	// Center bar
	renderer.fill_quad(
		renderer::Quad {
			bounds: Rectangle {
				x: tip_x - thickness / 2.0,
				y: cy - half_h,
				width: thickness,
				height: half_h * 2.0,
			},
			..renderer::Quad::default()
		},
		color,
	);
}

fn draw_chevron_v<Renderer>(
	renderer: &mut Renderer,
	cx: f32,
	tip_y: f32,
	base_y: f32,
	half_w: f32,
	color: Color,
) where
	Renderer: renderer::Renderer,
{
	let thickness = 1.5;
	let mid_y = (tip_y + base_y) / 2.0;
	// Left arm
	renderer.fill_quad(
		renderer::Quad {
			bounds: Rectangle {
				x: cx - half_w,
				y: mid_y.min(tip_y),
				width: thickness,
				height: (tip_y - mid_y).abs().max(thickness),
			},
			..renderer::Quad::default()
		},
		color,
	);
	// Right arm
	renderer.fill_quad(
		renderer::Quad {
			bounds: Rectangle {
				x: cx + half_w - thickness,
				y: mid_y.min(tip_y),
				width: thickness,
				height: (tip_y - mid_y).abs().max(thickness),
			},
			..renderer::Quad::default()
		},
		color,
	);
	// Center bar
	renderer.fill_quad(
		renderer::Quad {
			bounds: Rectangle {
				x: cx - half_w,
				y: tip_y - thickness / 2.0,
				width: half_w * 2.0,
				height: thickness,
			},
			..renderer::Quad::default()
		},
		color,
	);
}

pub fn collapsed_strip_thickness() -> f32 {
	COLLAPSED_STRIP_THICKNESS
}

pub fn handle_thickness(style: &HandleStyle) -> f32 {
	style.thickness
}
