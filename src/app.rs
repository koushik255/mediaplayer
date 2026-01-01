use iced::Task;
use iced_video_player::Video;
use rusqlite::Connection;
use rusqlite::Error;
use rusqlite::Result;
use rusqlite::params;
use std::clone;
use std::io;
use std::path;
use std::time::{Duration, SystemTime};

use url::Url;

use ass_parser::{AssFile, Dialogue};
use srtlib::{Subtitles, Timestamp};
use std::fs::{File, read_dir};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub enum Message {
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
    NewSub(Option<String>),
    LanguageSelected(usize, String),
    AddAtSelection,
    ManualSelection,
    OverlayPressed,
    UsingOwnSubs,
}

pub struct App {
    pub video: Video,
    pub position: f64,
    pub dragging: bool,
    pub volume: f64,
    pub muted: bool,

    pub subtitles: Vec<SubtitleEntry>,
    pub active_subtitle: Option<String>,
    pub prev_sub: Option<String>,

    pub video_url: PathBuf,
    pub subtitle_file: PathBuf,
    pub last_from_db: Dbchoose,
    pub value: String,
    pub parsed: Option<f64>,

    pub subtitle_folder: String,
    pub subtitle_folder_position: usize,
    pub subtitle_folder_current_sub: PathBuf,
    pub video_folder_better: VideoFolder,
    pub sorted_folders: SortedFolder,
    pub vec: Vec<String>,
    pub selected_lang: String,
    pub selected_index: usize,
    pub manual_select: Option<usize>,
    pub is_built_in_subs: bool,
    pub file_is_loaded: bool,
}

#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    start: Duration,
    end: Duration,
    text: String,
}

// todos
// i really need to clean this code up because its so un enjoybale to work on TBH also the ui
// should probably be re done but first i need to handle like all the pnaics and make it not so
// many buttons

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
        // let default_vid = "/home/koushikk/Documents/Rust2/iced-video-crate/src/defvid.mp4";
        let default_vid = "/home/koushikk/Downloads/foden.mkv";
        let default_sub = "/home/koushikk/Documents/Rust2/iced-video-crate/src/simple.srt";

        let mut video = Video::new(&url::Url::from_file_path(default_vid).unwrap()).unwrap();

        let subtitle_file = PathBuf::from(default_sub);

        let subtitles = parse_example_subs(default_sub).unwrap();

        let path = PathBuf::from(default_vid);

        let def_pos = 0.0;
        println!("{}", def_pos.clone());

        video
            .set_subtitle_url(&url::Url::from_file_path(default_sub).unwrap())
            .unwrap();

        let def_vid_folder = VideoFolder {
            folder: "None".to_string(),
            position: 0,
            current_video: PathBuf::from("."),
        };
        //let random: Vec<PathBuf> = Vec::new();
        let sorted_def = SortedFolder {
            video: Vec::new(),
            subs: Vec::new(),
        };
        let mut vec = Vec::with_capacity(10);

        Self {
            video,
            position: def_pos,
            dragging: false,
            volume: 1.0,
            muted: false,
            subtitles,
            subtitle_file: subtitle_file.clone(),
            active_subtitle: None,
            prev_sub: None,
            video_url: path,
            last_from_db: lastdbdb,
            parsed: Some(0.0),
            value: "".to_string(),
            video_folder_better: def_vid_folder,
            subtitle_folder: "none".to_string(),
            subtitle_folder_position: 0,
            subtitle_folder_current_sub: subtitle_file,
            sorted_folders: sorted_def,
            vec,
            selected_lang: "".to_string(),
            selected_index: 0,
            manual_select: None,
            is_built_in_subs: true,
            file_is_loaded: false,
        }
    }
}
#[derive(Debug)]
pub struct VideoFolder {
    folder: String,
    position: usize,
    current_video: PathBuf,
}

#[derive(Debug)]
pub struct SortedFolder {
    pub video: Vec<(usize, PathBuf)>,
    pub subs: Vec<(usize, PathBuf)>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OverlayPressed => {
                println!("Overlay button pressed");
                Task::none()
            }
            Message::LanguageSelected(index, language) => {
                self.selected_lang = language;
                self.selected_index = index;
                self.manual_select = None;

                if self.selected_lang == "Rust" {
                    self.vec.push("Rusty".into());
                }
                println!(
                    "you selected : {} {}",
                    self.selected_lang, self.selected_index
                );

                Task::none()
            }
            Message::AddAtSelection => {
                println!("Now playing slecetd video");
                println!("{}", self.selected_lang);
                self.video_folder_better.position = self.selected_index + 1;

                // can func this
                let url = Url::from_file_path(self.selected_lang.clone()).expect("error URL");

                let new_video = Video::new(&url).expect("Error creating new video in pause");
                self.video_url = PathBuf::from(self.selected_lang.clone());

                self.video = new_video;
                // func

                Task::none()
            }
            Message::ManualSelection => {
                if let Some(option) = self.vec.get(2) {
                    option.clone_into(&mut self.selected_lang);
                    self.selected_index = 2;
                    self.manual_select = Some(2);
                }
                Task::none()
            }
            Message::TogglePause => {
                self.video.set_paused(!self.video.paused());
                println!(
                    "{} {} {}",
                    self.last_from_db.vid_file, self.last_from_db.time, self.last_from_db.subfile
                );

                println!("subtitle file currently {:?}", self.subtitle_file.clone());
                println!("Video folder struct {:?}", self.video_folder_better);
                //let alldbs = db_get_all();
                // println!("{:?}", alldbs.unwrap());
                //
                for video in &self.sorted_folders.video {
                    println!("videos sorted {:?}", video);
                }

                println!("{}", self.video.framerate());

                Task::none()
            }
            Message::Next => {
                let path = PathBuf::from(self.subtitle_folder.clone());
                let bubbilites = read_videos_safely(path.as_path());

                let heresub: Vec<(usize, std::path::PathBuf)> =
                    bubbilites.into_iter().enumerate().collect();

                self.sorted_folders.subs = heresub.clone();
                let herebro = self.sorted_folders.video.clone();

                if let Some((i, vid)) = herebro.get(self.video_folder_better.position) {
                    println!("first video {} {}", i, vid.display());
                    self.video_folder_better.position = *i + 1;
                    self.video_folder_better.current_video = vid.clone();
                }
                if let Some((i, sub)) = heresub.get(self.subtitle_folder_position) {
                    println!("first subtitle {} {}", i, sub.display());
                    self.subtitle_folder_position = *i + 1;
                    self.subtitle_folder_current_sub = sub.clone();
                }

                self.subtitle_file = self.subtitle_folder_current_sub.clone();
                self.subtitles =
                    parse_example_subs(self.subtitle_file.clone().to_str().as_ref().unwrap())
                        .expect("error parse");
                println!("updated subtitle file");

                //self.update_active_subtitle();
                // 1.75

                let path_str = self
                    .video_folder_better
                    .current_video
                    .to_path_buf()
                    .to_string_lossy()
                    .into_owned();
                println!(" path string beofre to url {}", path_str);

                let url_her = path_str.as_str();
                println!("Path string after as str {}", url_her);
                let url = Url::from_file_path(self.video_folder_better.current_video.clone())
                    .expect("error URL");

                let new_video = Video::new(&url).expect("Error creating new video in pause");
                self.video_url = self
                    .video_folder_better
                    .current_video
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
            Message::NewSub(sub_text) => {
                // This message should only send IF we are not using our own subs
                // so if own subs = true do nothing,
                println!("subs from new sub {:?}", sub_text.clone());
                self.active_subtitle = sub_text.clone();

                if !self.is_built_in_subs {
                    println!("using built in subs");
                } else {
                    if self.prev_sub != sub_text {
                        self.prev_sub = self.active_subtitle.clone();
                        self.active_subtitle = sub_text.clone();
                        println!("in the else");
                    } else {
                        self.prev_sub = self.prev_sub.clone();
                    }
                    // none of the code after this is being usied in this message idk why maybe
                    // because wait its yeah i have no clue dude

                    self.active_subtitle = sub_text.clone();

                    match sub_text {
                        Some(text) => {
                            println!("sub {}", text);
                        }
                        None => {
                            println!("no subs blud");
                        }
                    }
                }

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
                    //self.update_active_subtitle();
                    // here decide the bool
                }
                if !self.is_built_in_subs {
                    println!("{}", self.is_built_in_subs);
                    self.update_active_subtitle();
                }
                // yeah i need to make it so that subtitles and subttiles with mkv is different
                // accpet type
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
                        .await;

                    match handle {
                        Some(folder_handle) => match url::Url::from_file_path(folder_handle.path())
                        {
                            Ok(url) => Ok(url),
                            Err(_) => Err("Invalid file path".to_string()),
                        },
                        None => Err("No file chosen".to_string()),
                    }
                },
                Message::Opened,
            ),
            Message::Opened(result) => {
                let svy = match result.clone() {
                    Ok(url) => url.to_file_path().unwrap().to_string_lossy().into_owned(),
                    Err(e) => {
                        println!("error string svy {}", e);
                        "ntohing".to_string()
                    }
                };

                self.video_url = PathBuf::from(svy);

                match result {
                    Ok(url) => match Video::new(&url.clone()) {
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
                // if i just made it so that if you open subtitles it would auto do that but thats
                // also leaving myself error prone
                // i really need to make the ui bettter because its fucking cooked rn
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
                // let subfile = file.clone().unwrap_or_else(|e| {
                //     eprintln!("no file chosen {}", e);
                //     "ntoihgnl".to_string()
                // });
                //
                let subfile = match file.clone() {
                    Ok(f) => f.to_string_lossy().into_owned(),
                    Err(e) => {
                        println!("dadwad {}", e);
                        "ntohing".to_string()
                    }
                };
                self.subtitle_file = PathBuf::from(subfile);

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
                // self.file_is_loaded = true;
                // let folder = folder.unwrap().to_string_lossy().into_owned();

                let bomba = match folder.clone() {
                    Ok(path) => path,
                    Err(e) => {
                        eprintln!("no folder chosen {}", e);
                        std::path::PathBuf::new()
                    }
                };

                // yah ok i like this way for error handling, acuallty i reallt dont tbh
                // this is gpt but idk i dont hate it, it just seems over complicated to put into a
                // closure, same thing as making a function for it
                //
                // let mut videos = read_dir(bomba.clone())
                //     .map_err(|e| eprintln!("Error reading video folder: {}", e))
                //     .ok()
                //     .map(|entries| {
                //         self.file_is_loaded = true;
                //         entries
                //             .map(|res| res.map(|e| e.path()))
                //             .collect::<Result<Vec<_>, io::Error>>()
                //             .map_err(|e| eprintln!("Error collecting paths: {}", e))
                //             .ok()
                //             .unwrap_or_default()
                //     })
                //     .unwrap_or_default();

                let mut videos = read_videos_safely(&bomba);
                if videos.is_empty() {
                    self.file_is_loaded = false;
                } else {
                    self.file_is_loaded = true;
                }

                videos.sort();

                // if i set this to a sels variables i can just use this in next
                let herebro: Vec<(usize, std::path::PathBuf)> =
                    videos.clone().into_iter().enumerate().collect();
                self.sorted_folders.video = herebro.clone();

                for (_i, vid) in herebro {
                    self.vec.push(vid.to_string_lossy().into_owned());
                }

                self.video_folder_better.folder = bomba.clone().to_string_lossy().into_owned();

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

                let folder = match folder {
                    Ok(folder) => {
                        let fol2 = folder.to_string_lossy().into_owned();
                        self.subtitle_folder = fol2;
                    }
                    Err(e) => {
                        println!("no subtitles folder selected blud{}", e);
                    }
                };
                // so what i did here what i handlpesd the error on the unwraps
                // let folder = folder.unwrap().to_string_lossy().into_owned();
                // self.subtitle_folder = folder;
                println!("folder {:?}", folder);
                println!("current subtitle file {:?}", self.subtitle_file.clone());
                Task::none()
            }
            Message::UsingOwnSubs => {
                if self.is_built_in_subs {
                    self.is_built_in_subs = false;
                } else {
                    self.is_built_in_subs = true;
                }

                Task::none()
            }
        }
    }
    fn update_active_subtitle(&mut self) {
        let mut t = Duration::from_secs_f64(self.position);
        t += Duration::from_secs_f64(self.parsed.unwrap());
        println!("updated t {:?} + {:?}", t, self.parsed.unwrap(),);
        // i need to have 2 different types of subtitles, one for the mkv video and another for the
        // default type, if the video is the default type then
        // ok this is easy, make a boolean if the file is .mkv then it has subtitles built in so
        // dont refresh the subtitles on everythign, otherwise if we are using our own subtitles
        // then we need to

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
pub struct Dbchoose {
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
// see now its for my design choice do i want to

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

fn read_videos_safely(path: &Path) -> Vec<PathBuf> {
    let entries = match read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error reading directory {:?}: {}", path, e);
            return Vec::new();
        }
    };
    let paths: Result<Vec<_>, _> = entries.map(|res| res.map(|e| e.path())).collect();

    match paths {
        Ok(mut vids) => {
            vids.sort();
            vids
        }
        Err(e) => {
            eprintln!("Error collecting paths: {}", e);
            Vec::new()
        }
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
