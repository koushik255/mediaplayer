use iced::widget::{Button, Column, Container, Row, Slider, Text, button};
use iced::{Element, Task};
use iced_video_player::{Video, VideoPlayer};
use std::time::Duration;

use ass_parser::{AssFile, Dialogue, Dialogues};
use srtlib::{Subtitles, Timestamp};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::Instant;

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
}

struct App {
    video: Video,
    position: f64,
    dragging: bool,
    volume: f64,
    muted: bool,

    subtitles: Vec<SubtitleEntry>,
    active_subtitle: Option<String>,
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
        let subtitles = parse_example_subs().unwrap();

        Self {
            video,
            position: 0.0,
            dragging: false,
            volume: 1.0,
            muted: false,
            subtitles,
            active_subtitle: None,
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

            // for the subtitles could i not just like put it under the video like wait i can 100%
            // just display the text and sync it up with the video i mean the video and the
            // subtitles should have a direction relationship
            // what if i took the srt read it then at each time at the video if the time of the
            // video == the time of the subtitle it just displayed under it that lowk so free no?
            // Goal: figure out how to parse the subtitles one by one
            // i mean as said above their i would just need to correlate the subtitles with the
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
                Container::new(Text::new(subtitle_text).size(24))
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

        if let Some(entry) = self.subtitles.iter().find(|s| s.start <= t && t <= s.end) {
            self.active_subtitle = Some(entry.text.clone());
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
    out.replace("\\N", "\n").trim().to_string()
}

fn parse_example_subs() -> Result<Vec<SubtitleEntry>, String> {
    let mut entries: Vec<SubtitleEntry> = Vec::new();

    if let Ok(subs) = Subtitles::parse_from_file("example.srt", None) {
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
        "/home/koushikk/Documents/Rust2/parseingsrt/src/Darling in the FranXX - Ep 001.ass",
    ) {
        let dialogues: Vec<Dialogue> = ass_file.events.get_dialogues();
        for d in dialogues {
            let (b, e, txt) = (
                (d.get_start().unwrap()),
                d.get_end().unwrap(),
                d.get_text().unwrap(),
            );
            {
                let (start, end) = (
                    ass_time_to_duration(&b).unwrap(),
                    ass_time_to_duration(&e).unwrap(),
                );
                {
                    let clean = strip_ass_tags(&txt);
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

/// subtitle flow
/// get subtitle parses it wiht the function and sets the global var of subtiles as 
/// a vec because in the function we return a Vec of SubtitleEntry then in the function 
/// on the app we just check if the time is after the start or before the end and we keep it on 
/// the screen and since its updateing per frame 60 per sec this is probably accurate 
/// then this just update the global active subtitle var which we just print to the screen 
