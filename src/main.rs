mod app;
mod colors;
mod fonts;
mod pane_grid;

fn main() -> iced::Result {
	iced::application(app::boot, app::update, app::view)
		.antialiasing(true)
		.font(fonts::DEJAVU_SANS_MONO)
		.title("Lattiton")
		.window_size((1200.0, 800.0))
		.run()
}
