mod app;
mod ui;

use app::App;

fn main() -> iced::Result {
    println!("runnig on new");
    iced::application("Iced Video Player", App::update, App::view).run_with(App::new)
}
