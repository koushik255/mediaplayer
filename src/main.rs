use iced::widget::{Button, Column, Container, Row, Slider, Text, button};
use iced::{Element, Task, executor};
use iced_video_player::{Video, VideoPlayer};
use rusqlite::Connection;
use rusqlite::Result;
use rusqlite::params;
use std::borrow::Cow;
use std::thread::{self, sleep};
use std::time::{self, Duration, SystemTime};
use tokio::time::timeout;

use ass_parser::{AssFile, Dialogue, Dialogues};
use srtlib::{Subtitles, Timestamp};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
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
    Quit,
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
    video_url: PathBuf,
    subtitle_file: PathBuf,
    last_from_db: Dbchoose,
}

#[derive(Debug, Clone)]
struct SubtitleEntry {
    start: Duration,
    end: Duration,
    text: String,
}

impl Default for App {
    fn default() -> Self {
        let last_db = db_get_last();

        //let mut video = Video::new( &url::Url::from_file_path(std::path::PathBuf::from(
        //       "/home/koushikk/Downloads/Download(1).mp4",
        //   ))
        //   .unwrap(),
        //  )
        //  .unwrap();

        println!("Video initialized with volume: 1.0");
        let def_sub = last_db.subfile.as_str();
        println!("CHECKING IF DEFSUB FUCK UP AS STR {}", def_sub);

        let mut video = Video::new(
            &url::Url::from_file_path(std::path::PathBuf::from(last_db.vid_file.clone())).unwrap(),
        )
        .unwrap();

        //let def_url = Url::from_file_path(
        //     "/home/koushikk/Documents/Rust2/parseingsrt/src/Darling in the FranXX - Ep 001.ass",
        //  );
        //  // maybe i make it so you can have multiple last played like for eaxh file its
        //  propritary
        //
        //  i could make it another db and just on exit it ques it into 2 dbs one for last and
        //  another for the perm save and that would save for if the file name is not already in
        //  the db
        //  and thats how i would make the save states work
        let subtitle_file = PathBuf::from(def_sub);

        let subtitles = parse_example_subs(def_sub).unwrap();

        let (tx, rx) = mpsc::channel();
        let path = PathBuf::from(last_db.vid_file.clone());
        let def_pos = last_db.time;
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
            subtitle_file,
            active_subtitle: None,
            video_url: path,
            last_from_db: last_db,
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
                println!(
                    "{} {} {}",
                    self.last_from_db.vid_file, self.last_from_db.time, self.last_from_db.subfile
                );
                //let alldbs = db_get_all();
                // println!("{:?}", alldbs.unwrap());
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
                //b_for_each(new_pos, new_url, new_subfile);
                println!("both dbed worked");
                // into owned turns lossy into String i mean i should have assumed so TBH

                iced::exit()
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
                // do i make it so that the file open is the same case as this or something
                // differnt i mean since it returns something it would be hard to make it the smae
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

                    // println!("url beofre to str {:?}", handle);

                    //url::Url::from_file_path(handle.path())
                    //    .map_err(|_| "Invalid file path".to_string())
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
            } // so i should choose a directory, it then reads the directory and then maybe in the
              // order it print it, you would know the next one because you could just enumerate your
              // position and +1 and ur good on the other and you could click next and it would go
              // next and after chosing ur folder for video you just chose your folder for the
              // subtites aswell
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
        let mut t = Duration::from_secs_f64(self.position);
        t += Duration::from_secs_f64(0.0);

        // if i make this run on a different thread, then it wouldnt conflict with the pauseing of
        // the video right?

        if let Some(entry) = self.subtitles.iter().find(|s| {
            s.start + Duration::from_millis(000) <= t && t <= s.end + Duration::from_millis(0000)
            // 3000 for fate/zero
            // i should make it so i can put like a scroll bar so you can do it dynamically within
            // the app easility
            // need to add feature which keeps your last played video and it gets the time aswell
            // i woudlny be able to do it on close because what if they are open multiple videos at
            // once it would have to be per video then also on close into the local db
        }) {
            self.active_subtitle = Some(entry.text.clone());
            //{
            //  thread::sleep(Duration::from_secs(1));
            //};
            //println!("{:?}", Some(entry.text.clone()));
            //println!("{:?} ", SystemTime::now());
            let herebro = entry.text.clone();
            println!("{:?}, TIME: {:?}", herebro.clone(), SystemTime::now());

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
    Some(Duration::from_millis(millis) + Duration::from_secs(2))
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
                // i could just add to time here
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

// before the for each db i need to make something which can somewhat sink the folders of the
// subttiles and video folder,
// first il make it display the episode and subtitle file on the screen
// subtitles are on screen but i should me able to open a folder then it plays the videos in the
// appropriate order
// you open a folder, it first would just print out the dir
// then i mean most folder are already in order so maybe it just works lowk
// acuallty i dont know if i want to add this because its like to much formatting and shit maybe if
// i find a different way
//
// needs to take the 3 params
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
    // for impl iterators all you have to do is collect them thats basically
    // all bruh
    // review this i have no fucking clue whats happening tbh

    Ok(all)
}

fn db_get_last() -> Dbchoose {
    let conn = Connection::open("mydb.sqlite3").expect("Error connecting to db");

    let mut stmt = conn
        .prepare("SELECT time,file,subfile FROM last")
        .expect("statement error");

    let last = stmt.query_one([], |row| {
        Ok(Dbchoose {
            time: row.get(0).expect("error time db"),
            vid_file: row.get(1).expect("error file db"),
            subfile: row.get(2).expect("error sub db"),
        })
    });

    let lastreturn = last.unwrap().clone();
    println!("{:?}", lastreturn);

    lastreturn
}

#[allow(dead_code)]
fn read_file(path: &Path) -> String {
    let mut f = File::open(path).expect("failed to open file");
    let mut s = String::new();
    f.read_to_string(&mut s)
        .expect("failed to read file as utf-8 text");
    s
}
