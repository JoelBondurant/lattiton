use iced::Color;

pub const BG_PRIMARY: Color = rgb(1, 1, 1);
pub const BG_SECONDARY: Color = rgb(18, 18, 22);
pub const BORDER_SUBTLE: Color = rgb(40, 40, 50);
pub const TEXT_PRIMARY: Color = rgb(230, 230, 230);
pub const HANDLE_DOT: Color = rgb(120, 120, 140);
pub const HANDLE_BG: Color = rgb(12, 12, 16);
pub const HANDLE_BORDER: Color = rgba(60, 8, 100, 0.4);
pub const HANDLE_ARROW: Color = rgb(100, 100, 120);
pub const HANDLE_ARROW_HOVER: Color = rgb(150, 4, 250);

const fn rgb(r: u8, g: u8, b: u8) -> Color {
	Color::from_rgb8(r, g, b)
}

const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
	Color::from_rgba8(r, g, b, a)
}
