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
    iced::application("Iced Video Player", App::update, App::view)
        .window_size(iced::Size::new(1700.0, 1300.0))
        .subscription(App::subscription)
        .run_with(App::new)
}
