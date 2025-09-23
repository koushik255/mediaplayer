use iced::widget::{Button, Column, Container, Row, Slider, Text, button, text_input};
use iced::{Element, Task};
use iced_video_player::{Video, VideoPlayer};
use rusqlite::Connection;
use rusqlite::Error;
use rusqlite::Result;
use rusqlite::params;
use std::io;
use std::time::{Duration, SystemTime};

use url::Url;

use ass_parser::{AssFile, Dialogue};
use srtlib::{Subtitles, Timestamp};
use std::fs::{File, read_dir};
use std::io::Read;
use std::path::{Path, PathBuf};

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
    Quit,
    ValueChanged(String),
    SubmitPressed,
    OpenVidFolder,
    OpenedFolder(Result<std::path::PathBuf, String>),
    Next,
    OpenSubFolder,
    OpenedSubFolder(Result<std::path::PathBuf, String>),
    OpenLast,
}

struct App {
    video: Video,
    position: f64,
    dragging: bool,
    volume: f64,
    muted: bool,

    subtitles: Vec<SubtitleEntry>,
    active_subtitle: Option<String>,

    video_url: PathBuf,
    subtitle_file: PathBuf,
    last_from_db: Dbchoose,
    value: String,
    parsed: Option<f64>,
    video_folder: String,
    video_folder_positon: usize,
    video_folder_current_video: PathBuf,
    subtitle_folder: String,
    subtitle_folder_position: usize,
    subtitle_folder_current_sub: PathBuf,
}

#[derive(Debug, Clone)]
struct SubtitleEntry {
    start: Duration,
    end: Duration,
    text: String,
}

impl Default for App {
    fn default() -> Self {
        let mut lastdbdb = Dbchoose {
            time: (0.0),
            vid_file: ("none".to_string()),
            subfile: ("none".to_string()),
        };

        match db_get_last() {
            Ok(last_db) => {
                println!("Found last row: {:?}", last_db);
                lastdbdb = last_db
            }
            Err(e) => {
                println!("Error {}", e);
            }
        }
        println!("Video initialized with volume: 1.0");
        let def_sub = lastdbdb.subfile.as_str();
        println!("CHECKING IF DEFSUB FUCK UP AS STR {}", def_sub);
        // you would need to change this to the dir
        let default_vid = "/home/koushikk/Documents/Rust2/iced-video-crate/src/defvid.mp4";
        let default_sub = "/home/koushikk/Documents/Rust2/iced-video-crate/src/defsub.ass";

        let mut video = Video::new(&url::Url::from_file_path(default_vid).unwrap()).unwrap();

        let subtitle_file = PathBuf::from(default_sub);

        let subtitles = parse_example_subs(default_sub).unwrap();

        let path = PathBuf::from(default_vid);

        let def_pos = 0.0;
        println!("{}", def_pos.clone());

        video
            .seek(Duration::from_secs_f64(def_pos), false)
            .expect("seek");

        Self {
            video,
            position: def_pos,
            dragging: false,
            volume: 1.0,
            muted: false,
            subtitles,
            subtitle_file: subtitle_file.clone(),
            active_subtitle: None,
            video_url: path,
            last_from_db: lastdbdb,
            parsed: Some(0.0),
            value: "".to_string(),
            video_folder: "none".to_string(),
            video_folder_positon: 0,
            video_folder_current_video: PathBuf::from("."),
            subtitle_folder: "none".to_string(),
            subtitle_folder_position: 0,
            subtitle_folder_current_sub: subtitle_file,
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
                println!(
                    "{} {} {}",
                    self.last_from_db.vid_file, self.last_from_db.time, self.last_from_db.subfile
                );

                println!("subtitle file currently {:?}", self.subtitle_file.clone());
                //let alldbs = db_get_all();
                // println!("{:?}", alldbs.unwrap());
                //

                Task::none()
            }
            Message::Next => {
                let mut videos = read_dir(self.video_folder.clone())
                    .expect("error reading video folder ")
                    .map(|res| res.map(|e| e.path()))
                    .collect::<Result<Vec<_>, io::Error>>()
                    .expect("error collecting vids");
                videos.sort();

                let mut subtitles = read_dir(self.subtitle_folder.clone())
                    .expect("error reading subtitles fodler")
                    .map(|e| e.map(|r| r.path()))
                    .collect::<Result<Vec<_>, io::Error>>()
                    .expect("Error collect subtitles");
                subtitles.sort();

                let herebro: Vec<(usize, std::path::PathBuf)> =
                    videos.into_iter().enumerate().collect();
                let heresub: Vec<(usize, std::path::PathBuf)> =
                    subtitles.into_iter().enumerate().collect();
                // println!("your folder better print {:?}", herebro);

                if let Some((i, vid)) = herebro.get(self.video_folder_positon) {
                    println!("first video {} {}", i, vid.display());
                    self.video_folder_positon = *i + 1;
                    self.video_folder_current_video = vid.clone();
                }
                if let Some((i, sub)) = heresub.get(self.subtitle_folder_position) {
                    println!("first subtitle {} {}", i, sub.display());
                    self.subtitle_folder_position = *i + 1;
                    self.subtitle_folder_current_sub = sub.clone();
                }

                self.subtitle_file = self.subtitle_folder_current_sub.clone();
                println!("updated subtitle file");

                self.update_active_subtitle();

                let path_str = self
                    .video_folder_current_video
                    .to_path_buf()
                    .to_string_lossy()
                    .into_owned();
                println!(" path string beofre to url {}", path_str);

                let url_her = path_str.as_str();
                println!("Path string after as str {}", url_her);
                let url = Url::from_file_path(self.video_folder_current_video.clone())
                    .expect("error URL");

                let new_video = Video::new(&url).expect("Error creating new video in pause");
                self.video_url = self
                    .video_folder_current_video
                    .to_string_lossy()
                    .into_owned()
                    .into();
                self.video = new_video;

                Task::none()
            }
            Message::OpenLast => {
                let last_vid = self.last_from_db.vid_file.clone();
                let last_sub = self.last_from_db.subfile.clone();
                let last_time = self.last_from_db.time;
                let video = Video::new(&url::Url::from_file_path(&last_vid).unwrap()).unwrap();
                self.video = video;
                self.position = last_time;
                self.subtitles = parse_example_subs(last_sub.as_str()).unwrap();
                self.video_url = PathBuf::from(last_vid.clone());
                self.subtitle_file = PathBuf::from(last_sub);

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
                //println!("{}, {:?}", self.position.clone(), self.video_url.clone());
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
            Message::Quit => {
                //db_get_last();
                //
                //
                //
                //
                println!(
                    "THINGS TO SAVE TO DB FILE:{:?}\n TIME: {:?} SUBTILTLE-FILE: {:?}",
                    self.video_url,
                    self.position,
                    self.subtitle_file.clone()
                );
                let (new_url, new_pos, new_subfile) = (
                    self.video_url.clone().to_string_lossy().into_owned(),
                    self.position,
                    self.subtitle_file.clone().to_string_lossy().into_owned(),
                );
                db(new_pos, new_url, new_subfile);

                println!("both dbed worked");

                iced::exit()
            }
            Message::ValueChanged(val) => {
                self.value = val.clone();

                // try parse
                self.parsed = val.parse::<f64>().ok();
                Task::none()
            }
            Message::SubmitPressed => {
                if let Some(number) = self.parsed {
                    println!("user number : {number}");
                } else {
                    println!("user number no work");
                }

                Task::none()
            }

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
                self.video_url = result.clone().unwrap().to_file_path().unwrap();
                println!("updated video_url");

                match result {
                    Ok(url) => match Video::new(&url) {
                        Ok(new_video) => {
                            self.video = new_video;
                            self.position = 0.0;
                            self.dragging = false;
                            //self.video.set_subtitle_url()

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
                },
                Message::OpenedSubtitles,
            ),
            Message::OpenedSubtitles(file) => {
                println!("url before  opedn sub() {:?}", file);
                self.subtitle_file = file.clone().unwrap();
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
            Message::OpenVidFolder => Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("chose a video folder")
                        //.add_filter("video files", &["mp4"])
                        .pick_folder()
                        .await;
                    match handle {
                        Some(folder_handle) => Ok(folder_handle.path().to_path_buf()),
                        None => Err("no folder chosen".to_string()),
                    }
                },
                Message::OpenedFolder,
            ),
            Message::OpenedFolder(folder) => {
                println!("folder location {:?}", folder);
                let folder = folder.unwrap().to_string_lossy().into_owned();

                self.video_folder = folder.clone();
                Task::none()
            }
            Message::OpenSubFolder => Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("select a subtitle folder")
                        .pick_folder()
                        .await;

                    match handle {
                        Some(folder_handle) => Ok(folder_handle.path().to_path_buf()),
                        None => Err("no folder chosen".to_string()),
                    }
                },
                Message::OpenedSubFolder,
            ),
            Message::OpenedSubFolder(folder) => {
                println!("subtitle folder location {:?}", folder);

                let folder = folder.unwrap().to_string_lossy().into_owned();
                self.subtitle_folder = folder;
                println!("current subtitle file {:?}", self.subtitle_file.clone());
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let subtitle_text = self.active_subtitle.as_deref().unwrap_or("");
        let filename_text = self
            .video_url
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let subtitles_file = self
            .subtitle_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

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
                Container::new(Text::new(subtitles_file).size(20))
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center)
                    .padding(iced::Padding::new(10.0).left(20.0).right(100.0)),
            )
            .push(
                Container::new(Text::new(filename_text).size(20))
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center)
                    .padding(iced::Padding::new(10.0).left(20.0).right(100.0)),
            )
            .push(
                Container::new(
                    text_input("Enter a number...", &self.value)
                        .on_input(Message::ValueChanged)
                        .on_submit(Message::SubmitPressed)
                        .padding(5)
                        .size(15), // font size
                )
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center)
                .padding(iced::Padding::new(10.0)),
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
                        .push(
                            Button::new(Text::new("QUIT"))
                                .width(40.0)
                                .on_press(Message::Quit),
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
                    .push(button("OPEN VID FOLDER").on_press(Message::OpenVidFolder))
                    .push(button("OPEN SUB FOLDER").on_press(Message::OpenSubFolder))
                    .push(button("Open Subtitles").on_press(Message::OpenSubtitle))
                    .push(button("next video").on_press(Message::Next))
                    .push(button("last vid").on_press(Message::OpenLast))
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
        let mut t = Duration::from_secs_f64(self.position);
        t += Duration::from_secs_f64(self.parsed.unwrap());
        println!("updated t {:?} + {:?} ", t, self.parsed.unwrap());

        if let Some(entry) = self.subtitles.iter().find(|s| {
            s.start + Duration::from_millis(000) <= t && t <= s.end + Duration::from_millis(0000)
        }) {
            self.active_subtitle = Some(entry.text.clone());

            let herebro = entry.text.clone();
            println!("{:?}, TIME: {:?}", herebro.clone(), SystemTime::now());
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
    Some(Duration::from_millis(millis) + Duration::from_secs(2))
}

fn strip_ass_tags(s: &str) -> String {
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

    if let Ok(ass_file) = AssFile::from_file(file) {
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

#[derive(Debug, Clone)]
struct Dbchoose {
    time: f64,
    vid_file: String,
    subfile: String,
}
fn db(time: f64, vid_file: String, subfile: String) {
    let conn = Connection::open("mydb.sqlite3").expect("error connecting to db");
    println!(
        "DB: params which are being inserted {} {} {}",
        time, vid_file, subfile
    );

    conn.execute(
        "CREATE TABLE IF NOT EXISTS last (
                    time  REAL,
                    file  TEXT NOT NULL,
                    subfile  TEXT NOT NULL
)",
        [],
    )
    .expect("erroring creating db table");
    conn.execute("DELETE from last", [])
        .expect("Error deleting lsat table");

    conn.execute(
        "INSERT INTO last (time,file,subfile) VALUES (?1,?2,?3)",
        params![time, vid_file, subfile],
    )
    .expect("erroing inserting last");

    println!("succesfully added last to db");
}

fn db_for_each(time: f64, vid_file: String, subfile: String) {
    let conn = Connection::open("allvids.sqlite3").expect("Error connecting to db");
    println!("db for each now");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS all (
                    time REAL,
                    file TEXT NOT NULL UNIQUE,
                    subfile TEXT NOT NULL UNIQUE
)",
        [],
    )
    .expect("error creaing db table");

    conn.execute(
        "INSERT INTO all(time,file,subfile) VALUES(?1,?2,?3)",
        params![time, vid_file, subfile],
    )
    .expect("Erroring inserting last");

    println!("after inserting to everything")
}

fn db_get_all() -> Result<Vec<Dbchoose>> {
    let conn = Connection::open("allvids.sqlite3").expect("errpr connecting ot db");
    println!("db get all");

    let mut stmt = conn
        .prepare("SELECT time,file,subfile FROM all")
        .expect("error selcting db all");

    let all = stmt
        .query_map([], |row| {
            Ok(Dbchoose {
                time: row.get(0).expect("error time db all"),
                vid_file: row.get(1).expect("error file db all"),
                subfile: row.get(2).expect("errrpr sub db"),
            })
        })
        .expect("error getting query ")
        .collect::<Result<Vec<_>, _>>()
        .expect("error collect");
    Ok(all)
}

fn db_get_last() -> Result<Dbchoose, String> {
    let conn =
        Connection::open("mydb.sqlite3").map_err(|e| format!("Erorr connecting to db: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT time,file,subfile FROM last")
        .map_err(|e| format!("Statement error {}", e))?;

    match stmt.query_row([], |row| {
        Ok(Dbchoose {
            time: row.get(0)?,
            vid_file: row.get(1)?,
            subfile: row.get(2)?,
        })
    }) {
        Ok(last) => {
            println!("{:?}", last);
            Ok(last)
        }
        Err(Error::QueryReturnedNoRows) => {
            println!("Could not find last");
            Err("Could not find last".to_string())
        }
        Err(e) => Err(format!("Database error {}", e)),
    }
}

#[allow(dead_code)]
fn read_file(path: &Path) -> String {
    let mut f = File::open(path).expect("failed to open file");
    let mut s = String::new();
    f.read_to_string(&mut s)
        .expect("failed to read file as utf-8 text");
    s
}
