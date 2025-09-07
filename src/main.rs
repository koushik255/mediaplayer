use iced::widget::{Button, Column, Container, Row, Slider, Text, button};
use iced::{Element, Task};
use iced_video_player::{Video, VideoPlayer};
use std::thread::{self, sleep};
use std::time::Duration;
use tokio::time::timeout;

use ass_parser::{AssFile, Dialogue, Dialogues};
use srtlib::{Subtitles, Timestamp};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};

fn main() -> iced::Result {
    iced::application("Iced Video Player", App::update, App::view).run_with(App::new)
}

#[derive(Clone, Debug)]
enum Message {
    TogglePause,
    ToggleLoop,
    Seek(f64),
    SeekRelease,
    EndOfStream,
    NewFrame,
    Open,
    Opened(Result<url::Url, String>),
    VolumeChanged(f64),
    ToggleMute,
    OpenSubtitle,
    OpenedSubtitles(Result<std::path::PathBuf, String>),
}

struct App {
    video: Video,
    position: f64,
    dragging: bool,
    volume: f64,
    muted: bool,

    subtitles: Vec<SubtitleEntry>,
    active_subtitle: Option<String>,
    rx: Receiver<String>,
    tx: Sender<String>,
}

#[derive(Debug, Clone)]
struct SubtitleEntry {
    start: Duration,
    end: Duration,
    text: String,
}

impl Default for App {
    fn default() -> Self {
        let mut video = Video::new(
            &url::Url::from_file_path(std::path::PathBuf::from(
                "/home/koushikk/Downloads/Download(1).mp4",
            ))
            .unwrap(),
        )
        .unwrap();

        video.set_volume(1.0);
        println!("Video initialized with volume: 1.0");
        let def_sub =
            "/home/koushikk/Documents/Rust2/parseingsrt/src/Darling in the FranXX - Ep 001.ass";

        //let def_url = Url::from_file_path(
        //     "/home/koushikk/Documents/Rust2/parseingsrt/src/Darling in the FranXX - Ep 001.ass",
        //  );

        let subtitles = parse_example_subs(def_sub).unwrap();

        let (tx, rx) = mpsc::channel();

        Self {
            video,
            position: 0.0,
            dragging: false,
            volume: 1.0,
            muted: false,
            subtitles,
            active_subtitle: None,
            tx,
            rx,
        }
    }
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TogglePause => {
                self.video.set_paused(!self.video.paused());
                Task::none()
            }
            Message::ToggleLoop => {
                self.video.set_looping(!self.video.looping());
                Task::none()
            }
            Message::Seek(secs) => {
                self.dragging = true;
                self.video.set_paused(true);
                self.position = secs;
                Task::none()
            }
            Message::SeekRelease => {
                self.dragging = false;
                self.video
                    .seek(Duration::from_secs_f64(self.position), false)
                    .expect("seek");
                self.video.set_paused(false);
                self.update_active_subtitle();
                Task::none()
            }
            Message::EndOfStream => {
                println!("end of stream");
                Task::none()
            }
            Message::NewFrame => {
                if !self.dragging {
                    self.position = self.video.position().as_secs_f64();
                    self.update_active_subtitle();
                }
                Task::none()
            }
            Message::VolumeChanged(vol) => {
                self.volume = vol;
                let actual_volume = if self.muted { 0.0 } else { vol };
                self.video.set_volume(actual_volume);
                println!("Volume changed to: {} (actual: {})", vol, actual_volume);
                Task::none()
            }
            Message::ToggleMute => {
                self.muted = !self.muted;
                let actual_volume = if self.muted { 0.0 } else { self.volume };
                self.video.set_volume(actual_volume);
                println!("Mute toggled: {} (volume: {})", self.muted, actual_volume);
                Task::none()
            }

            // next task is to make it so i can push the subtitles forwards or backwards,
            // meaning like you can sync them up yourself
            //im pretty sure the ass_parser library has something for this
            //but i probably want to do this in memory right?
            //or i could just like make a new file each time and since the .ass files arent
            //large this wouldnt really be a problem and it would deal with like the history
            //of the file itsself, could i not just put a sleep on the playing subtitles func and
            //have it mut? honeslty it would be beneficial if i did that but i dont feel like it
            //right now because all it is, if i want it to be as fast ass possible i should just
            //change the file tbh
            //reading and writing can be put onto threads since they can be blocking
            Message::Open => Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("Choose a video file")
                        .add_filter("Video files", &["mp4", "avi", "mkv", "mov", "wmv"])
                        .pick_file()
                        .await
                        .ok_or_else(|| "No file chosen".to_string())?;

                    url::Url::from_file_path(handle.path())
                        .map_err(|_| "Invalid file path".to_string())
                },
                Message::Opened,
            ),
            Message::Opened(result) => {
                match result {
                    Ok(url) => match Video::new(&url) {
                        Ok(new_video) => {
                            self.video = new_video;
                            self.position = 0.0;
                            self.dragging = false;
                            // load new subtitle here
                            self.update_active_subtitle();
                        }
                        Err(e) => {
                            eprintln!("Failed to load video: {:?}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to open file: {}", e);
                    }
                }
                Task::none()
            }

            Message::OpenSubtitle => Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("Choose a subtitle file")
                        .add_filter("Video files", &["srt", "ass"])
                        .pick_file()
                        .await;

                    match handle {
                        Some(file_handle) => Ok(file_handle.path().to_path_buf()),
                        None => Err("no file chosen".to_string()),
                    }

                    // println!("url beofre to str {:?}", handle);

                    //url::Url::from_file_path(handle.path())
                    //    .map_err(|_| "Invalid file path".to_string())
                },
                Message::OpenedSubtitles,
            ),
            Message::OpenedSubtitles(file) => {
                println!("url before  opedn sub() {:?}", file);
                match file {
                    Ok(path) => {
                        let path_str = path.to_string_lossy();
                        match parse_example_subs(&path_str) {
                            Ok(new_sub) => {
                                println!("loaded :{}", new_sub.len());
                                self.subtitles = new_sub;
                                self.update_active_subtitle();
                            }
                            Err(e) => {
                                eprintln!("failed to load subttiesl {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("faield to open sub file{}", e);
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let subtitle_text = self.active_subtitle.as_deref().unwrap_or("");

        Column::new()
            .push(
                Container::new(
                    VideoPlayer::new(&self.video)
                        .on_end_of_stream(Message::EndOfStream)
                        .on_new_frame(Message::NewFrame),
                )
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
            )
            .push(
                Container::new(Text::new(subtitle_text).size(50))
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center)
                    .padding(iced::Padding::new(10.0).left(20.0).right(100.0)),
            )
            .push(
                Container::new(
                    Slider::new(
                        0.0..=self.video.duration().as_secs_f64(),
                        self.position,
                        Message::Seek,
                    )
                    .step(0.1)
                    .on_release(Message::SeekRelease),
                )
                .padding(iced::Padding::new(5.0).left(10.0).right(10.0)),
            )
            .push(
                Container::new(
                    Row::new()
                        .spacing(10)
                        .push(
                            Button::new(Text::new(if self.muted { "🔇" } else { "🔊" }))
                                .width(40.0)
                                .on_press(Message::ToggleMute),
                        )
                        .push(Text::new("Volume:"))
                        .push(
                            Slider::new(0.0..=1.0, self.volume, Message::VolumeChanged)
                                .step(0.01)
                                .width(150.0),
                        )
                        .push(Text::new(format!(
                            "{:.0}%",
                            if self.muted { 0.0 } else { self.volume * 100.0 }
                        ))),
                )
                .padding(iced::Padding::new(5.0).left(10.0).right(10.0)),
            )
            .push(
                Row::new()
                    .spacing(5)
                    .align_y(iced::alignment::Vertical::Center)
                    .padding(iced::Padding::new(10.0).top(0.0))
                    .push(
                        Button::new(Text::new(if self.video.paused() {
                            "Play"
                        } else {
                            "Pause"
                        }))
                        .width(80.0)
                        .on_press(Message::TogglePause),
                    )
                    .push(
                        Button::new(Text::new(if self.video.looping() {
                            "Disable Loop"
                        } else {
                            "Enable Loop"
                        }))
                        .width(120.0)
                        .on_press(Message::ToggleLoop),
                    )
                    .push(button("Open").on_press(Message::Open))
                    .push(button("Open Subtitles").on_press(Message::OpenSubtitle))
                    .push(
                        Text::new(format!(
                            "{}:{:02} / {}:{:02}",
                            self.position as u64 / 60,
                            self.position as u64 % 60,
                            self.video.duration().as_secs() / 60,
                            self.video.duration().as_secs() % 60,
                        ))
                        .width(iced::Length::Fill)
                        .align_x(iced::alignment::Horizontal::Right),
                    ),
            )
            .into()
    }
    fn update_active_subtitle(&mut self) {
        let t = Duration::from_secs_f64(self.position);

        // if i make this run on a different thread, then it wouldnt conflict with the pauseing of
        // the video right?

        if let Some(entry) = self.subtitles.iter().find(|s| {
            s.start + Duration::from_millis(3000) <= t && t <= s.end + Duration::from_millis(3000)
        }) {
            self.active_subtitle = Some(entry.text.clone());
            //{
            //  thread::sleep(Duration::from_secs(1));
            //};
            println!("{:?}", Some(entry.text.clone()));
            let herebro = entry.text.clone();

            self.tx.send(herebro).expect("error sending herebro");
        } else {
            self.active_subtitle = None;
        }
    }
}
fn ts_to_duration(t: &Timestamp) -> Duration {
    let (h, m, s, ms) = t.get();
    Duration::from_millis(
        (h as u64) * 3_600_000 + (m as u64) * 60_000 + (s as u64) * 1_000 + (ms as u64),
    )
}

fn ass_time_to_duration(t: &str) -> Option<Duration> {
    let mut parts = t.split(':');
    let h = parts.next()?.parse::<u64>().ok()?;
    let m = parts.next()?.parse::<u64>().ok()?;
    let sec_cs = parts.next()?; // "SS.cs"

    let mut sc = sec_cs.split('.');
    let s = sc.next()?.parse::<u64>().ok()?;
    let cs = sc.next()?.parse::<u64>().ok()?; // centiseconds (00–99)

    let millis = h * 3_600_000 + m * 60_000 + s * 1_000 + cs * 10;
    Some(Duration::from_millis(millis))
}

fn strip_ass_tags(s: &str) -> String {
    // Simple tag stripper: removes {...} blocks and converts \N to newline
    let mut out = String::with_capacity(s.len());
    let mut in_brace = false;
    for c in s.chars() {
        match c {
            '{' => in_brace = true,
            '}' => in_brace = false,
            _ if !in_brace => out.push(c),
            _ => {}
        }
    }
    //let new = out.replace("\\N", "\n").trim().to_string();
    out.replace("\\N", "\n").trim().to_string()
    // commas (, ) current if something is after a comma its being cut off
}

fn parse_example_subs(file: &str) -> Result<Vec<SubtitleEntry>, String> {
    let mut entries: Vec<SubtitleEntry> = Vec::new();
    println!("Before file parse from file");
    //let file = file.as_str();

    if let Ok(subs) = Subtitles::parse_from_file(file, None) {
        let mut v = subs.to_vec();
        v.sort();
        for s in v {
            entries.push(SubtitleEntry {
                start: ts_to_duration(&s.start_time),
                end: ts_to_duration(&s.end_time),
                text: s.text.trim().to_string(),
            });
        }
    }

    if let Ok(ass_file) = AssFile::from_file(
        //"/home/koushikk/Documents/Rust2/parseingsrt/src/Darling in the FranXX [Glue]/[Glue] Darling in the FranXX - 02 [40426618]_Track02_eng.ass",
        file,
    ) {
        let dialogues: Vec<Dialogue> = ass_file.events.get_dialogues();
        for d in dialogues {
            if let (Some(b), Some(e), Some(txt)) = (d.get_start(), d.get_end(), d.get_text()) {
                println!("before duration ass parse");
                println!("{}", file);
                let (start, end) = (
                    ass_time_to_duration(&b).unwrap(),
                    ass_time_to_duration(&e).unwrap(),
                );
                {
                    let clean = strip_ass_tags(&txt);
                    //let herebrodow = txt.replace(",", " ").trim().to_string();
                    println!("subtitles before push {:?}", &txt.clone());
                    entries.push(SubtitleEntry {
                        start,
                        end,
                        text: clean,
                    });
                }
            }
        }
    }

    entries.sort_by_key(|e| e.start);

    println!("LOADED SUBTITLE FILE");
    Ok(entries)
}

#[allow(dead_code)]
fn read_file(path: &Path) -> String {
    let mut f = File::open(path).expect("failed to open file");
    let mut s = String::new();
    f.read_to_string(&mut s)
        .expect("failed to read file as utf-8 text");
    s
}
