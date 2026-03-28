mod app;
mod colors;
mod lattiton;

fn main() -> iced::Result {
	iced::application(app::boot, app::update, app::view)
		.window_size((1200.0, 800.0))
		.antialiasing(true)
		.run()
}
