use iced::Color;

pub const BG_PRIMARY: Color = rgb(1, 1, 1);
pub const BG_SECONDARY: Color = rgb(1, 1, 2);
pub const BORDER_SUBTLE: Color = rgb(40, 40, 50);
pub const DROP_OVERLAY: Color = rgba(60, 8, 100, 0.25);
pub const HANDLE_ARROW: Color = rgb(150, 4, 250);
pub const HANDLE_ARROW_HOVER: Color = rgb(180, 100, 250);
pub const HANDLE_BG: Color = rgb(8, 2, 10);
pub const HANDLE_BORDER: Color = rgba(60, 8, 100, 0.4);
pub const HANDLE_DOT_BOTTOM: Color = rgb(0, 0, 0);
pub const HANDLE_DOT_TOP: Color = rgb(150, 4, 250);
pub const TEXT_PRIMARY: Color = rgb(230, 230, 230);

const fn rgb(r: u8, g: u8, b: u8) -> Color {
	Color::from_rgb8(r, g, b)
}

const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
	Color::from_rgba8(r, g, b, a)
}
