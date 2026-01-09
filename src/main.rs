mod app;
mod app_types;
mod config;
mod database;
mod subtitles;
mod ui;
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use app::App;

fn main() -> iced::Result {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    println!("runnig on new");
    iced::application(App::new, App::update, App::view)
        .resizable(true)
        .window_size(iced::Size::new(1920.0, 1200.0))
        .resizable(true)
        .subscription(App::subscription)
        .run()
}
