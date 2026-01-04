mod app;
mod ui;
mod app_types;
mod database;
mod subtitles;
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
        .run_with(App::new)
}
