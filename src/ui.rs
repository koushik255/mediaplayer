use iced::Alignment;
use iced::Font;
use iced::Length;
use std::path::{Path, PathBuf};

use iced::widget::{button, Button, Column, Container, Row, Slider, Stack, Text};
use iced::{Element, Padding};
use iced_aw::style::colors::WHITE;
use iced_aw::{selection_list::SelectionList, style::selection_list::primary};
use iced_video_player::VideoPlayer;

use crate::app::App;
use crate::app_types::Message;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let main_content = self.main_view();

        if self.settings_open {
            Container::new(
                Stack::new().push(main_content).push(
                    Container::new(self.settings_window())
                        .align_x(iced::Alignment::Center)
                        .align_y(iced::Alignment::Center),
                ),
            )
            .into()
        } else {
            main_content
        }
    }

    fn main_view(&self) -> Element<'_, Message> {
        let _filename_text = match self.video_url.file_name() {
            Some(name) => name.to_string_lossy().into_owned(),
            None => {
                eprintln!(
                    "Error: no filename found in the path {}",
                    self.video_url.display()
                );
                String::from("unknown_filename")
            }
        };

        let _subtitles_file = self
            .subtitle_file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        let mut heresubdudebud = String::new();
        if let Some(text) = &self.active_subtitle {
            heresubdudebud = text.replace("&apos;", "'").replace("&quot;", "\"");
            // println!("{heresubdudebud}");
        }

        let video_layer = Container::new(
            VideoPlayer::new(&self.video)
                .on_end_of_stream(Message::EndOfStream)
                .on_new_frame(Message::NewFrame)
                .on_subtitle_text(Message::NewSub),
        )
        .width(iced::Length::Fixed(self.video_width))
        .height(iced::Length::Fixed(self.video_height));

        let subtitle_text = Container::new(
            Text::new(heresubdudebud).size(35).color(WHITE), // the subtitle
        )
        .padding(10)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.0, 0.0, 0.0, 0.8,
            ))),
            border: iced::border::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 5.0.into(),
            },
            shadow: iced::Shadow::default(),
            text_color: Some(WHITE),
        });

        let subtitle_layer = Container::new(subtitle_text)
            .width(iced::Length::Fixed(self.video_width))
            .height(iced::Length::Fixed(self.video_height))
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::End)
            .padding(
                iced::Padding::new(0.0)
                    .bottom(self.subtitle_offset_vertical as f32)
                    .left(self.subtitle_offset_horizontal as f32),
            );

        let overlay_stack = Stack::new().push(video_layer).push(subtitle_layer);

        let video_with_list = Row::new()
            .push(
                Container::new(overlay_stack)
                    .align_x(iced::Alignment::Start)
                    .align_y(iced::Alignment::Center)
                    .padding(Padding::new(0.0).left(20.0).top(60.0)),
            )
            .push(self.list());

        let controls_row = Container::new(
            Row::new()
                .spacing(15)
                .align_y(iced::Alignment::End)
                .padding(10)
                .push(
                    Container::new(
                        Row::new()
                            .spacing(5)
                            .push(
                                Button::new(Text::new(if self.video.paused() {
                                    "Play"
                                } else {
                                    "Pause"
                                }))
                                .width(120.0)
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
                            .push(self.next_button())
                            .push(
                                Button::new(Text::new("Previous"))
                                    .width(120.0)
                                    .on_press(Message::OpenLast),
                            )
                            .push(
                                Button::new(Text::new("Settings"))
                                    .width(120.0)
                                    .on_press(Message::ToggleSettings),
                            )
                            .push(
                                Text::new(format!(
                                    "{}:{:02} / {}:{:02}",
                                    self.position as u64 / 60,
                                    self.position as u64 % 60,
                                    self.video.duration().as_secs() / 60,
                                    self.video.duration().as_secs() % 60,
                                ))
                                .width(120.0)
                                .align_x(iced::Alignment::Center),
                            ),
                    )
                    .padding(10)
                    .style(|_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.05, 0.05, 0.05,
                        ))),
                        border: iced::border::Border {
                            color: iced::Color::from_rgb(0.2, 0.2, 0.2),
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        shadow: iced::Shadow::default(),
                        text_color: None,
                    }),
                )
                .push(
                    Container::new(
                        Column::new()
                            .spacing(10)
                            .push(
                                Container::new(
                                    Row::new()
                                        .spacing(8)
                                        .push(
                                            Button::new(Text::new(if self.muted {
                                                "Mute"
                                            } else {
                                                "Unmute"
                                            }))
                                            .width(150.0)
                                            .on_press(Message::ToggleMute),
                                        )
                                        .push(
                                            button("Own Subs")
                                                .width(150.0)
                                                .on_press(Message::UsingOwnSubs),
                                        )
                                        .push(
                                            button("Add At Selection")
                                                .width(150.0)
                                                .on_press(Message::AddAtSelection),
                                        )
                                        .push(self.audio_track_button())
                                        .push(self.subtitle_track_button()),
                                )
                                .padding(10)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgb(0.05, 0.05, 0.05),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgb(0.2, 0.2, 0.2),
                                            width: 1.0,
                                            radius: 8.0.into(),
                                        },
                                        shadow: iced::Shadow::default(),
                                        text_color: None,
                                    }
                                }),
                            )
                            .push(
                                Container::new(
                                    Row::new()
                                        .spacing(5)
                                        .push(button("Open").width(120.0).on_press(Message::Open))
                                        .push(
                                            button("Open Video Folder")
                                                .width(140.0)
                                                .on_press(Message::OpenVidFolder),
                                        )
                                        .push(
                                            button("Open Subtitle File")
                                                .width(150.0)
                                                .on_press(Message::OpenSubtitle),
                                        )
                                        .push(
                                            button("Open Subtitle Folder")
                                                .width(160.0)
                                                .on_press(Message::OpenSubFolder),
                                        )
                                        .push(
                                            button("Screenshot (URI)")
                                                .width(140.0)
                                                .on_press(Message::TakeScreenshotURI),
                                        )
                                        .push(button("Quit").width(120.0).on_press(Message::Quit)),
                                )
                                .padding(10)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgb(0.05, 0.05, 0.05),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgb(0.2, 0.2, 0.2),
                                            width: 1.0,
                                            radius: 8.0.into(),
                                        },
                                        shadow: iced::Shadow::default(),
                                        text_color: None,
                                    }
                                }),
                            ),
                    )
                    .padding(10)
                    .style(|_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.02, 0.02, 0.02,
                        ))),
                        border: iced::border::Border {
                            color: iced::Color::from_rgb(0.15, 0.15, 0.15),
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        shadow: iced::Shadow::default(),
                        text_color: None,
                    }),
                ),
        );

        Column::new()
            .push(video_with_list)
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
            .push(controls_row)
            .into()
    }

    fn list(&self) -> Element<'_, Message> {
        let selection_list = SelectionList::new_with(
            &self.vec,
            Message::LanguageSelected,
            9.0,
            6.0,
            primary,
            self.manual_select,
            Font::default(),
        )
        .width(Length::Shrink)
        .height(Length::Fill);

        let content = Column::new()
            .width(Length::Fill)
            .align_x(Alignment::End)
            .spacing(10)
            .push(selection_list)
            .push(Text::new(format!("{:?}", self.selected_lang)))
            .push(button("Manual select Index 2").on_press(Message::ManualSelection));

        Container::new(content)
            .width(Length::Fill)
            .height(iced::Length::Fixed(900.0))
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::End)
            .align_y(Alignment::Start)
            .padding(20)
            .into()
    }

    pub fn next_button(&self) -> Element<'_, Message> {
        let mut next_button = Button::new(Text::new(if self.file_is_loaded {
            "Next video"
        } else {
            "No next video"
        }));

        if self.file_is_loaded {
            next_button = next_button.on_press(Message::Next);
        }

        next_button.into()
    }

    pub fn audio_track_button(&self) -> Element<'_, Message> {
        if self.available_audio_tracks.len() > 1 {
            let next_track = (self.current_audio_track + 1) % self.available_audio_tracks.len();
            Button::new(Text::new(format!(
                "Audio: {}",
                self.available_audio_tracks[self.current_audio_track]
            )))
            .on_press(Message::AudioTrackSelected(next_track))
            .into()
        } else {
            Container::new(Text::new("")).into()
        }
    }

    pub fn subtitle_track_button(&self) -> Element<'_, Message> {
        if self.available_subtitle_tracks.len() > 1 {
            let next_track =
                (self.current_subtitle_track + 1) % self.available_subtitle_tracks.len();
            Button::new(Text::new(format!(
                "Sub: {}",
                self.available_subtitle_tracks[self.current_subtitle_track]
            )))
            .on_press(Message::SubtitleTrackSelected(next_track))
            .into()
        } else {
            Container::new(Text::new("")).into()
        }
    }

    fn settings_window(&self) -> Element<'_, Message> {
        Container::new(
            Column::new()
                .spacing(15)
                .padding(20)
                .push(
                    Row::new()
                        .spacing(10)
                        .align_y(iced::Alignment::Center)
                        .push(Text::new("Settings").size(24).color(WHITE))
                        .push(Row::new().width(iced::Length::Fill))
                        .push(
                            Button::new(Text::new("✕"))
                                .on_press(Message::ToggleSettings)
                                .width(40.0),
                        ),
                )
                .push(
                    Container::new(
                        Column::new()
                            .spacing(12)
                            .push(
                                Row::new()
                                    .spacing(10)
                                    .push(Text::new("Volume:").color(WHITE))
                                    .push(
                                        Slider::new(0.0..=1.0, self.volume, Message::VolumeChanged)
                                            .step(0.01)
                                            .width(280.0),
                                    )
                                    .push(
                                        Text::new(format!(
                                            "{:.0}%",
                                            if self.muted { 0.0 } else { self.volume * 100.0 }
                                        ))
                                        .color(WHITE),
                                    ),
                            )
                            .push(
                                Row::new()
                                    .spacing(10)
                                    .push(Text::new("Video Width:").color(WHITE))
                                    .push(
                                        Slider::new(
                                            800.0..=1920.0,
                                            self.video_width,
                                            Message::VideoWidthChanged,
                                        )
                                        .step(10.0)
                                        .width(280.0),
                                    )
                                    .push(
                                        Text::new(format!("{:.0}px", self.video_width))
                                            .color(WHITE),
                                    ),
                            )
                            .push(
                                Row::new()
                                    .spacing(10)
                                    .push(Text::new("Video Height:").color(WHITE))
                                    .push(
                                        Slider::new(
                                            450.0..=1080.0,
                                            self.video_height,
                                            Message::VideoHeightChanged,
                                        )
                                        .step(10.0)
                                        .width(280.0),
                                    )
                                    .push(
                                        Text::new(format!("{:.0}px", self.video_height))
                                            .color(WHITE),
                                    ),
                            )
                            .push(
                                Row::new()
                                    .spacing(10)
                                    .push(Text::new("Subtitle Offset (sec):").color(WHITE))
                                    .push(
                                        Slider::new(
                                            -30.0..=30.0,
                                            self.subtitle_offset,
                                            Message::SubtitleOffsetChanged,
                                        )
                                        .step(0.1)
                                        .width(280.0),
                                    )
                                    .push(
                                        Text::new(format!("{:.1}s", self.subtitle_offset))
                                            .color(WHITE),
                                    ),
                            )
                            .push(
                                Row::new()
                                    .spacing(10)
                                    .push(Text::new("Subtitle Offset V:").color(WHITE))
                                    .push(
                                        Slider::new(
                                            0.0..=500.0,
                                            self.subtitle_offset_vertical,
                                            Message::SubtitleOffsetVerticalChanged,
                                        )
                                        .step(5.0)
                                        .width(280.0),
                                    )
                                    .push(
                                        Text::new(format!(
                                            "{:.0}px",
                                            self.subtitle_offset_vertical
                                        ))
                                        .color(WHITE),
                                    ),
                            )
                            .push(
                                Row::new()
                                    .spacing(10)
                                    .push(Text::new("Subtitle Offset H:").color(WHITE))
                                    .push(
                                        Slider::new(
                                            -400.0..=200.0,
                                            self.subtitle_offset_horizontal,
                                            Message::SubtitleOffsetHorizontalChanged,
                                        )
                                        .step(5.0)
                                        .width(280.0),
                                    )
                                    .push(
                                        Text::new(format!(
                                            "{:.0}px",
                                            self.subtitle_offset_horizontal
                                        ))
                                        .color(WHITE),
                                    ),
                            )
                            .push(
                                Row::new()
                                    .spacing(10)
                                    .push(Text::new("Default Video:").color(WHITE))
                                    .push(
                                        Button::new(Text::new(
                                            self.default_video_path
                                                .as_ref()
                                                .map(|p| {
                                                    PathBuf::from(p)
                                                        .file_name()
                                                        .and_then(|n| n.to_str())
                                                        .unwrap_or("None selected")
                                                        .to_string()
                                                })
                                                .unwrap_or_else(|| "None selected".to_string()),
                                        ))
                                        .width(280.0)
                                        .on_press(Message::OpenDefaultVideoPicker),
                                    ),
                            )
                            .push(
                                Row::new()
                                    .spacing(10)
                                    .push(Text::new("Screenshot Folder:").color(WHITE))
                                    .push(
                                        Button::new(Text::new(
                                            self.screenshot_folder
                                                .as_ref()
                                                .map(|p| {
                                                    let path = Path::new(p);
                                                    path.file_name()
                                                        .and_then(|n| n.to_str())
                                                        .unwrap_or_else(|| {
                                                            path.to_str().unwrap_or("None")
                                                        })
                                                        .to_string()
                                                })
                                                .unwrap_or_else(|| "None selected".to_string()),
                                        ))
                                        .width(280.0)
                                        .on_press(Message::OpenScreenshotFolderPicker),
                                    ),
                            ),
                    )
                    .padding(15)
                    .style(|_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.1, 0.1, 0.1,
                        ))),
                        border: iced::border::Border {
                            color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                            width: 2.0,
                            radius: 8.0.into(),
                        },
                        shadow: iced::Shadow::default(),
                        text_color: None,
                    }),
                )
                .width(800)
                .height(iced::Length::Shrink),
        )
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.0, 0.0, 0.0, 0.95,
            ))),
            border: iced::border::Border {
                color: iced::Color::from_rgb(0.4, 0.4, 0.4),
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::BLACK,
                offset: iced::Vector::new(8.0, 8.0),
                blur_radius: 25.0,
            },
            text_color: None,
        })
        .width(550)
        .into()
    }
}

fn my_column<'a>() -> Element<'a, Message> {
    Column::new()
        .push("a column can be used to ")
        .push("lay out widgets vertically")
        .spacing(10)
        .into()
}
