use std::path::PathBuf;

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
    UsingOwnSubs,
    AudioTrackSelected(usize),
    SubtitleTrackSelected(usize),
    SubtitleOffsetChanged(f64),
    SubtitleOffsetVerticalChanged(f64),
    SubtitleOffsetHorizontalChanged(f64),
    VideoWidthChanged(f32),
    VideoHeightChanged(f32),
    ToggleSettings,
    TakeScreenshotURI,
    ScreenshotSaved(std::path::PathBuf),
    OpenDefaultVideoPicker,
    SetDefaultVideo(Result<std::path::PathBuf, String>),
    OpenScreenshotFolderPicker,
    SetScreenshotFolder(Result<std::path::PathBuf, String>),
    ShowNotification(String),
    DismissNotification,
}

#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    pub start: std::time::Duration,
    pub end: std::time::Duration,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct VideoEntry {
    pub display_name: String,
    pub full_path: PathBuf,
}

#[derive(Debug)]
pub struct VideoFolder {
    pub folder: String,
    pub position: usize,
    pub current_video: PathBuf,
}

#[derive(Debug)]
pub struct SortedFolder {
    pub video: Vec<(usize, PathBuf)>,
    pub subs: Vec<(usize, PathBuf)>,
}

#[derive(Debug, Clone)]
pub struct Dbchoose {
    pub time: f64,
    pub vid_file: String,
    pub subfile: String,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
}
