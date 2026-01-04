use iced::Task;
use iced_video_player::Video;
use std::time::Duration;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use crate::app_types::*;
use crate::config::{load_config, save_config, AppConfig};
use crate::database::{db, db_get_last, save_settings, load_settings};
use crate::subtitles::parse_example_subs;

use gstreamer::prelude::*;
use url::Url;

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

    pub subtitle_folder: String,
    pub subtitle_folder_position: usize,
    pub subtitle_folder_current_sub: PathBuf,
    pub video_folder_better: VideoFolder,
    pub sorted_folders: SortedFolder,
    pub video_entries: Vec<VideoEntry>,
    pub vec: Vec<String>,
    pub selected_lang: String,
    pub selected_index: usize,
    pub manual_select: Option<usize>,
    pub is_built_in_subs: bool,
    pub file_is_loaded: bool,
    pub available_audio_tracks: Vec<String>,
    pub current_audio_track: usize,
    pub available_subtitle_tracks: Vec<String>,
    pub current_subtitle_track: usize,
    pub has_embedded_subtitles: bool,
    pub subtitle_offset: f64,
    pub subtitle_offset_vertical: f64,
    pub subtitle_offset_horizontal: f64,
    pub video_width: f32,
    pub video_height: f32,
    pub settings_open: bool,
    pub default_video_path: Option<String>,
    pub screenshot_folder: Option<String>,
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

        let (
            _subtitle_offset,
            _subtitle_offset_vertical,
            _subtitle_offset_horizontal,
            video_width,
            video_height,
            _volume,
        ) = match load_settings() {
            Ok(settings) => settings,
            Err(_) => (100.0, 100.0, 0.0, 1450.0, 1080.0, 1.0),
        };

        let app_config = match load_config() {
            Ok(config) => config,
            Err(e) => {
                println!("Failed to load config: {}", e);
                AppConfig::default()
            }
        };

        println!("Video initialized with volume: 1.0");
        let def_sub = lastdbdb.subfile.as_str();
        println!("CHECKING IF DEFSUB FUCK UP AS STR {}", def_sub);
        // you would need to change this to the dir
        // let default_vid = "/home/koushikk/Documents/Rust2/iced-video-crate/src/defvid.mp4";
        let hardcoded_default_vid = "/home/koushikk/Downloads/foden.mkv";
        let default_sub = "/home/koushikk/Documents/Rust2/iced-video-crate/src/simple.srt";

        let default_vid = app_config
            .default_video_path
            .as_ref()
            .and_then(|p| {
                if Path::new(p).exists() {
                    Some(p.as_str())
                } else {
                    None
                }
            })
            .unwrap_or(hardcoded_default_vid);

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
        let vec = Vec::with_capacity(10);

        let mut app = Self {
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
            video_folder_better: def_vid_folder,
            subtitle_folder: "none".to_string(),
            subtitle_folder_position: 0,
            subtitle_folder_current_sub: subtitle_file,
            sorted_folders: sorted_def,
            video_entries: Vec::new(),
            vec,
            selected_lang: "".to_string(),
            selected_index: 0,
            manual_select: None,
            is_built_in_subs: true,
            file_is_loaded: false,
            available_audio_tracks: Vec::new(),
            current_audio_track: 0,
            available_subtitle_tracks: Vec::new(),
            current_subtitle_track: 0,
            has_embedded_subtitles: false,
            subtitle_offset: app_config.subtitle_offset,
            subtitle_offset_vertical: app_config.subtitle_offset_vertical,
            subtitle_offset_horizontal: app_config.subtitle_offset_horizontal,
            video_width,
            video_height,
            settings_open: false,
            default_video_path: app_config.default_video_path,
            screenshot_folder: app_config.screenshot_folder,
        };

        app.detect_audio_tracks();
        app.detect_subtitle_tracks();

        app
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LanguageSelected(index, _language) => {
                self.selected_index = index;
                self.manual_select = None;

                if let Some(video_entry) = self.video_entries.get(index) {
                    if !video_entry.full_path.exists() {
                        eprintln!("Video file does not exist: {:?}", video_entry.full_path);
                        return Task::none();
                    }

                    self.selected_lang = video_entry.display_name.clone();
                    println!(
                        "you selected : {} {}",
                        self.selected_lang, self.selected_index
                    );

                    let url = match Url::from_file_path(&video_entry.full_path) {
                        Ok(url) => url,
                        Err(e) => {
                            eprintln!("Error creating URL from path: {:?}", e);
                            return Task::none();
                        }
                    };

                    let new_video = match Video::new(&url) {
                        Ok(video) => video,
                        Err(e) => {
                            eprintln!("Error creating new video: {:?}", e);
                            return Task::none();
                        }
                    };
                    self.video_url = video_entry.full_path.clone();
                    self.video = new_video;

                    self.detect_audio_tracks();
                    self.detect_subtitle_tracks();
                }

                Task::none()
            }
            Message::AddAtSelection => {
                println!("Now playing slecetd video");
                println!("{}", self.selected_lang);
                self.video_folder_better.position = self.selected_index + 1;

                if let Some(video_entry) = self.video_entries.get(self.selected_index) {
                    if !video_entry.full_path.exists() {
                        eprintln!("Video file does not exist: {:?}", video_entry.full_path);
                        return Task::none();
                    }

                    let url = match Url::from_file_path(&video_entry.full_path) {
                        Ok(url) => url,
                        Err(e) => {
                            eprintln!("Error creating URL from path: {:?}", e);
                            return Task::none();
                        }
                    };

                    let new_video = match Video::new(&url) {
                        Ok(video) => video,
                        Err(e) => {
                            eprintln!("Error creating new video: {:?}", e);
                            return Task::none();
                        }
                    };
                    self.video_url = video_entry.full_path.clone();
                    self.video = new_video;

                    self.detect_audio_tracks();
                    self.detect_subtitle_tracks();
                }

                Task::none()
            }
            Message::ManualSelection => {
                if let Some(option) = self.video_entries.get(2) {
                    self.selected_lang = option.display_name.clone();
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

                if !self.video_folder_better.current_video.exists() {
                    eprintln!(
                        "Video file does not exist: {:?}",
                        self.video_folder_better.current_video
                    );
                    return Task::none();
                }

                let url = match Url::from_file_path(self.video_folder_better.current_video.clone())
                {
                    Ok(url) => url,
                    Err(e) => {
                        eprintln!("Error creating URL from path: {:?}", e);
                        return Task::none();
                    }
                };

                let new_video = match Video::new(&url) {
                    Ok(video) => video,
                    Err(e) => {
                        eprintln!("Error creating new video: {:?}", e);
                        return Task::none();
                    }
                };
                self.video_url = self
                    .video_folder_better
                    .current_video
                    .to_string_lossy()
                    .into_owned()
                    .into();
                self.video = new_video;
                self.detect_audio_tracks();
                self.detect_subtitle_tracks();

                Task::none()
            }
            Message::OpenLast => {
                let last_vid = self.last_from_db.vid_file.clone();
                let last_sub = self.last_from_db.subfile.clone();
                let last_time = self.last_from_db.time;

                let last_vid_path = PathBuf::from(&last_vid);
                if !last_vid_path.exists() {
                    eprintln!("Last video file does not exist: {:?}", last_vid_path);
                    return Task::none();
                }

                let url = match url::Url::from_file_path(&last_vid) {
                    Ok(url) => url,
                    Err(e) => {
                        eprintln!("Error creating URL from last video path: {:?}", e);
                        return Task::none();
                    }
                };

                let video = match Video::new(&url) {
                    Ok(video) => video,
                    Err(e) => {
                        eprintln!("Error creating last video: {:?}", e);
                        return Task::none();
                    }
                };
                self.video = video;
                self.position = last_time;

                if !PathBuf::from(&last_sub).exists() {
                    eprintln!("Last subtitle file does not exist: {:?}", last_sub);
                } else {
                    self.subtitles = match parse_example_subs(last_sub.as_str()) {
                        Ok(subs) => subs,
                        Err(e) => {
                            eprintln!("Error parsing subtitles: {:?}", e);
                            Vec::new()
                        }
                    };
                }
                self.video_url = PathBuf::from(last_vid.clone());
                self.subtitle_file = PathBuf::from(last_sub);
                self.detect_audio_tracks();
                self.detect_subtitle_tracks();

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
                            println!("📺 NEW SUBTITLE DISPLAYED: {}", text);
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

                save_settings(
                    self.subtitle_offset,
                    self.subtitle_offset_vertical,
                    self.subtitle_offset_horizontal,
                    self.video_width,
                    self.video_height,
                    self.volume,
                );

                let app_config = AppConfig {
                    default_video_path: self.default_video_path.clone(),
                    screenshot_folder: self.screenshot_folder.clone(),
                    subtitle_offset: self.subtitle_offset,
                    subtitle_offset_vertical: self.subtitle_offset_vertical,
                    subtitle_offset_horizontal: self.subtitle_offset_horizontal,
                };

                if let Err(e) = save_config(&app_config) {
                    eprintln!("Failed to save config: {}", e);
                }

                println!("both dbed worked");

                iced::exit()
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

                            // Detect audio tracks
                            self.detect_audio_tracks();
                            // Detect subtitle tracks
                            self.detect_subtitle_tracks();

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

                self.video_entries.clear();
                self.vec.clear();

                for (_i, vid) in herebro {
                    let display_name = vid
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();
                    self.vec.push(display_name.clone());
                    self.video_entries.push(VideoEntry {
                        display_name,
                        full_path: vid,
                    });
                }

                self.video_folder_better.folder = bomba.clone().to_string_lossy().into_owned();

                if self.video_entries.is_empty() {
                    self.file_is_loaded = false;
                    self.selected_index = 0;
                    self.selected_lang = String::new();
                    self.manual_select = None;
                } else {
                    self.file_is_loaded = true;
                    self.selected_index = 0;
                    self.selected_lang = String::new();
                    self.manual_select = None;
                }

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
            Message::AudioTrackSelected(track_index) => {
                self.current_audio_track = track_index;
                let pipeline = self.video.pipeline();
                pipeline.set_property("current-audio", track_index as i32);
                println!("Switched to audio track {}", track_index);
                Task::none()
            }
            Message::SubtitleOffsetChanged(offset) => {
                self.subtitle_offset = offset;
                Task::none()
            }
            Message::VideoWidthChanged(width) => {
                self.video_width = width.clamp(800.0, 1920.0);
                Task::none()
            }
            Message::VideoHeightChanged(height) => {
                self.video_height = height.clamp(450.0, 1080.0);
                Task::none()
            }
            Message::SubtitleOffsetVerticalChanged(offset) => {
                self.subtitle_offset_vertical = offset;
                Task::none()
            }
            Message::SubtitleOffsetHorizontalChanged(offset) => {
                self.subtitle_offset_horizontal = offset;
                Task::none()
            }
            Message::ToggleSettings => {
                self.settings_open = !self.settings_open;
                Task::none()
            }
            Message::TakeScreenshotURI => {
                let filename = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let default_name = format!("screenshot_uri_{}.png", filename);

                let use_auto_save = if let Some(folder) = &self.screenshot_folder {
                    let folder_path = Path::new(folder);
                    folder_path.exists() && folder_path.is_dir()
                } else {
                    false
                };

                if use_auto_save {
                    let folder = self.screenshot_folder.as_ref().unwrap();
                    let filename = format!("screenshot_{}.png", filename);
                    let save_path = PathBuf::from(folder).join(&filename);
                    self.capture_and_save_screenshot(&save_path);
                    Task::none()
                } else {
                    Task::perform(
                        async move {
                            let handle = rfd::AsyncFileDialog::new()
                                .set_title("Save Screenshot (URI Method)")
                                .set_file_name(&default_name)
                                .add_filter("PNG Image", &["png"])
                                .save_file()
                                .await;

                            match handle {
                                Some(path) => path.path().to_path_buf(),
                                None => std::path::PathBuf::from(""),
                            }
                        },
                        Message::ScreenshotSaved,
                    )
                }
            }
            Message::ScreenshotSaved(path) => {
                if !path.as_os_str().is_empty() {
                    self.capture_and_save_screenshot(&path);
                }
                Task::none()
            }
            Message::SubtitleTrackSelected(track_index) => {
                self.current_subtitle_track = track_index;
                let pipeline = self.video.pipeline();
                pipeline.set_property("current-text", track_index as i32);
                println!("Switched to subtitle track {}", track_index);
                Task::none()
            }
            Message::OpenDefaultVideoPicker => Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("Select Default Video")
                        .add_filter("Video files", &["mp4", "avi", "mkv", "mov", "wmv"])
                        .pick_file()
                        .await;
                    match handle {
                        Some(path) => Ok(path.path().to_path_buf()),
                        None => Err("No file selected".to_string()),
                    }
                },
                Message::SetDefaultVideo,
            ),
            Message::SetDefaultVideo(result) => {
                self.default_video_path = match result {
                    Ok(path) => {
                        if path.exists() {
                            Some(path.to_string_lossy().into_owned())
                        } else {
                            eprintln!("Selected file does not exist: {:?}", path);
                            None
                        }
                    }
                    Err(_) => None,
                };

                let app_config = AppConfig {
                    default_video_path: self.default_video_path.clone(),
                    screenshot_folder: self.screenshot_folder.clone(),
                    subtitle_offset: self.subtitle_offset,
                    subtitle_offset_vertical: self.subtitle_offset_vertical,
                    subtitle_offset_horizontal: self.subtitle_offset_horizontal,
                };

                if let Err(e) = save_config(&app_config) {
                    eprintln!("Failed to save config: {}", e);
                }

                Task::none()
            }
            Message::OpenScreenshotFolderPicker => Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("Select Screenshot Folder")
                        .pick_folder()
                        .await;
                    match handle {
                        Some(path) => Ok(path.path().to_path_buf()),
                        None => Err("No folder selected".to_string()),
                    }
                },
                Message::SetScreenshotFolder,
            ),
            Message::SetScreenshotFolder(result) => {
                self.screenshot_folder = match result {
                    Ok(path) => {
                        if path.exists() && path.is_dir() {
                            Some(path.to_string_lossy().into_owned())
                        } else {
                            eprintln!("Selected path is not a valid directory: {:?}", path);
                            None
                        }
                    }
                    Err(_) => None,
                };

                let app_config = AppConfig {
                    default_video_path: self.default_video_path.clone(),
                    screenshot_folder: self.screenshot_folder.clone(),
                    subtitle_offset: self.subtitle_offset,
                    subtitle_offset_vertical: self.subtitle_offset_vertical,
                    subtitle_offset_horizontal: self.subtitle_offset_horizontal,
                };

                if let Err(e) = save_config(&app_config) {
                    eprintln!("Failed to save config: {}", e);
                }

                Task::none()
            }
        }
    }
    fn detect_audio_tracks(&mut self) {
        self.available_audio_tracks.clear();
        self.current_audio_track = 0;

        let pipeline = self.video.pipeline();

        for attempt in 0..5 {
            let num_audio = pipeline.property::<i32>("n-audio");

            if num_audio > 0 {
                for i in 0..num_audio {
                    let track_name = format!("Audio Track {}", i + 1);
                    self.available_audio_tracks.push(track_name);
                }
                println!(
                    "Found {} audio track(s) on attempt {}",
                    num_audio,
                    attempt + 1
                );
                return;
            }

            if attempt < 4 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        println!("No audio tracks detected after multiple attempts");
    }

    fn detect_subtitle_tracks(&mut self) {
        self.available_subtitle_tracks.clear();
        self.current_subtitle_track = 0;

        let pipeline = self.video.pipeline();

        for attempt in 0..5 {
            let num_subs = pipeline.property::<i32>("n-text");

            if num_subs > 0 {
                for i in 0..num_subs {
                    let track_name = format!("Subtitle Track {}", i + 1);
                    self.available_subtitle_tracks.push(track_name);
                }
                self.has_embedded_subtitles = true;
                println!(
                    "Found {} subtitle track(s) on attempt {}",
                    num_subs,
                    attempt + 1
                );
                return;
            }

            if attempt < 4 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        self.has_embedded_subtitles = false;
        println!("No embedded subtitle tracks detected after multiple attempts");
    }

    fn update_active_subtitle(&mut self) {
        let current_time = Duration::from_secs_f64(self.position);

        self.active_subtitle = self
            .subtitles
            .iter()
            .find(|sub| current_time >= sub.start && current_time <= sub.end)
            .map(|sub| sub.text.clone());
    }

    fn capture_and_save_screenshot(&mut self, save_path: &PathBuf) {
        if let Err(e) = self.capture_screenshot_with_gstreamer(save_path) {
            eprintln!("Failed to capture screenshot: {}", e);
        }
    }

    fn capture_screenshot_with_gstreamer(&mut self, save_path: &PathBuf) -> Result<(), String> {
        use gstreamer as gst;

        gstreamer::init().map_err(|e| format!("Failed to initialize GStreamer: {}", e))?;

        let video_path = self.video_url.clone();

        let position_nanos = (self.position * 1_000_000_000.0) as u64;

        let filesrc = gst::ElementFactory::make("filesrc")
            .build()
            .map_err(|e| format!("Failed to create filesrc: {}", e))?;

        let decodebin = gst::ElementFactory::make("decodebin")
            .build()
            .map_err(|e| format!("Failed to create decodebin: {}", e))?;

        let videoconvert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|e| format!("Failed to create videoconvert: {}", e))?;

        let videoscale = gst::ElementFactory::make("videoscale")
            .build()
            .map_err(|e| format!("Failed to create videoscale: {}", e))?;

        let pngenc = gst::ElementFactory::make("pngenc")
            .build()
            .map_err(|e| format!("Failed to create pngenc: {}", e))?;

        let filesink = gst::ElementFactory::make("filesink")
            .build()
            .map_err(|e| format!("Failed to create filesink: {}", e))?;

        let video_path_str = video_path.to_str().ok_or("Invalid video path")?;
        filesrc.set_property("location", &video_path_str);

        let save_path_str = save_path.to_str().ok_or("Invalid save path")?;
        filesink.set_property("location", &save_path_str);

        let pipeline = gst::Pipeline::new();
        pipeline
            .add_many(&[&filesrc, &decodebin, &videoconvert, &videoscale, &pngenc, &filesink])
            .map_err(|e| format!("Failed to add elements to pipeline: {}", e))?;

        filesrc
            .link(&decodebin)
            .map_err(|e| format!("Failed to link filesrc to decodebin: {}", e))?;

        let videoconvert_weak = videoconvert.downgrade();
        decodebin
            .connect_pad_added(move |_, src_pad| {
                let videoconvert = match videoconvert_weak.upgrade() {
                    Some(vc) => vc,
                    None => return,
                };

                if let Some(sink_pad) = videoconvert.static_pad("sink") {
                    if src_pad.current_caps().map_or(false, |caps| {
                        caps.iter().any(|c| c.name().as_str() == "video/x-raw")
                    }) {
                        let _ = src_pad.link(&sink_pad);
                    }
                }
            });

        videoconvert
            .link(&videoscale)
            .map_err(|e| format!("Failed to link videoconvert to videoscale: {}", e))?;

        videoscale
            .link(&pngenc)
            .map_err(|e| format!("Failed to link videoscale to pngenc: {}", e))?;

        pngenc
            .link(&filesink)
            .map_err(|e| format!("Failed to link pngenc to filesink: {}", e))?;

        pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| format!("Failed to set pipeline to playing: {}", e))?;

        let pipeline_clone = pipeline.clone();
        let seek_pos = gst::ClockTime::from_nseconds(position_nanos);

        std::thread::sleep(std::time::Duration::from_millis(100));

        if let Err(e) =
            pipeline.seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT, seek_pos)
        {
            eprintln!("Warning: Failed to seek to position: {}", e);
        }

        let bus = pipeline.bus().ok_or("Failed to get bus from pipeline")?;

        let _timeout = gst::ClockTime::from_seconds(5);
        let mut eos_received = false;
        let start_time = std::time::Instant::now();

        while !eos_received && start_time.elapsed() < std::time::Duration::from_secs(3) {
            let msg = bus.timed_pop(gst::ClockTime::from_mseconds(100));

            if let Some(msg) = msg {
                match msg.view() {
                    gst::MessageView::Eos(..) => {
                        eos_received = true;
                    }
                    gst::MessageView::Error(err) => {
                        pipeline_clone.set_state(gst::State::Null).ok();
                        return Err(format!(
                            "Error from GStreamer: {} ({})",
                            err.error(),
                            err.debug().unwrap_or_default()
                        ));
                    }
                    _ => {}
                }
            }
        }

        pipeline
            .set_state(gst::State::Null)
            .map_err(|e| format!("Failed to set pipeline to null: {}", e))?;

        println!(
            "Screenshot saved successfully to: {:?} (position: {:.2}s)",
            save_path, self.position
        );
        Ok(())
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
