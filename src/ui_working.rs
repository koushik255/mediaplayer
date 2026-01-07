use iced::widget::{Button, Column, Container, Row, Scrollable, Slider, Stack, Text};
use iced::{Element, Length};
use std::path::{Path, PathBuf};

use iced_aw::style::colors::WHITE;
use iced_video_player::VideoPlayer;

use crate::app::App;
use crate::app_types::Message;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let main_content = self.main_view();

        let notification_container = Container::new(self.notification_area())
            .align_x(iced::Alignment::End)
            .align_y(iced::Alignment::End)
            .width(Length::Fill)
            .height(Length::Fill);

        let content_with_notifications =
            Stack::new().push(main_content).push(notification_container);

        if self.settings_open {
            Container::new(
                Stack::new().push(content_with_notifications).push(
                    Container::new(self.settings_window())
                        .align_x(iced::Alignment::Center)
                        .align_y(iced::Alignment::Center),
                ),
            )
            .into()
        } else if self.video_info_open {
            Container::new(
                Stack::new().push(content_with_notifications).push(
                    Container::new(self.video_info_window())
                        .align_x(iced::Alignment::Center)
                        .align_y(iced::Alignment::Center),
                ),
            )
            .into()
        } else if self.file_panel_open {
            Container::new(
                Stack::new().push(content_with_notifications).push(
                    Container::new(self.file_panel_window())
                        .align_x(iced::Alignment::Center)
                        .align_y(iced::Alignment::Center),
                ),
            )
            .into()
        } else {
            Container::new(content_with_notifications).into()
        }
    }

    fn notification_area(&self) -> Element<'_, Message> {
        if self.notifications.is_empty() {
            return Container::new(Text::new("")).into();
        }

        let notification_text = &self.notifications[0].message;
        Container::new(Text::new(notification_text).size(16).color(WHITE))
            .padding(10)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.0, 0.0, 0.0, 0.8,
                ))),
                border: iced::border::Border {
                    color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.3),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow::default(),
                text_color: None,
            })
            .into()
    }

    fn main_view(&self) -> Element<'_, Message> {
        let filename_text = match self.video_url.file_name() {
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
        }

        let title_text = Container::new(Text::new(filename_text).size(16).color(WHITE))
            .padding(6)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.0, 0.0, 0.0, 0.6,
                ))),
                border: iced::border::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: 6.0.into(),
                },
                shadow: iced::Shadow::default(),
                text_color: Some(WHITE),
            });

        let video_layer = Container::new(
            VideoPlayer::new(&self.video)
                .on_end_of_stream(Message::EndOfStream)
                .on_new_frame(Message::NewFrame)
                .on_subtitle_text(Message::NewSub),
        )
        .width(iced::Length::Fixed(self.video_width))
        .height(iced::Length::Fixed(self.video_height));

        let subtitle_text = Container::new(Text::new(heresubdudebud).size(32).color(WHITE))
            .padding(8)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.0, 0.0, 0.0, 0.6,
                ))),
                border: iced::border::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: 6.0.into(),
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

        let title_layer = Container::new(title_text)
            .width(iced::Length::Fixed(self.video_width))
            .height(iced::Length::Fixed(self.video_height))
            .align_x(iced::Alignment::End)
            .align_y(iced::Alignment::Start)
            .padding(iced::Padding::new(5.0));

        let overlay_stack = Stack::new()
            .push(video_layer)
            .push(title_layer)
            .push(subtitle_layer);

        let video_centered = Container::new(overlay_stack)
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center);

        let top_bar = Container::new(
            Row::new()
                .spacing(8)
                .padding(5)
                .push(
                    Button::new(Text::new(if self.video.paused() { "▶" } else { "⏸" }))
                        .width(40.0)
                        .height(30.0)
                        .on_press(Message::TogglePause),
                )
                .push(
                    Button::new(Text::new(if self.video.looping() {
                        "🔁"
                    } else {
                        "🔁"
                    }))
                    .width(40.0)
                    .height(30.0)
                    .on_press(Message::ToggleLoop),
                )
                .push(
                    Button::new(Text::new("⚙"))
                        .width(40.0)
                        .height(30.0)
                        .on_press(Message::ToggleSettings),
                )
                .push(
                    Button::new(Text::new("📁"))
                        .width(40.0)
                        .height(30.0)
                        .on_press(Message::ToggleFilePanel),
                )
                .push(
                    Button::new(Text::new("ℹ"))
                        .width(40.0)
                        .height(30.0)
                        .on_press(Message::ToggleVideoInfo),
                ),
        )
        .width(iced::Length::Fixed(self.video_width))
        .align_x(iced::Alignment::Center)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.0, 0.0, 0.0, 0.7,
            ))),
            border: iced::border::Border {
                color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: iced::Shadow::default(),
            text_color: None,
        });

        let slider_container = Container::new(
            Slider::new(
                0.0..=self.video.duration().as_secs_f64(),
                self.position,
                Message::Seek,
            )
            .step(0.1)
            .on_release(Message::SeekRelease),
        )
        .width(iced::Length::Fixed(self.video_width))
        .align_x(iced::Alignment::Center)
        .padding(iced::Padding::new(10.0))
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.0, 0.0, 0.0, 0.7,
            ))),
            border: iced::border::Border {
                color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                width: 1.5,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            text_color: None,
        });

        Column::new()
            .push(top_bar)
            .push(video_centered)
            .push(slider_container)
            .into()
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
                                Container::new(
                                    Row::new()
                                        .spacing(10)
                                        .push(Text::new("Volume").color(WHITE))
                                        .push(
                                            Slider::new(
                                                0.0..=1.0,
                                                self.volume,
                                                Message::VolumeChanged,
                                            )
                                            .step(0.01)
                                            .width(200.0),
                                        )
                                        .push(
                                            Button::new(Text::new(if self.muted {
                                                "🔇"
                                            } else {
                                                "🔊"
                                            }))
                                            .width(40.0)
                                            .on_press(Message::ToggleMute),
                                        )
                                        .push(
                                            Text::new(format!(
                                                "{:.0}%",
                                                if self.muted { 0.0 } else { self.volume * 100.0 }
                                            ))
                                            .color(WHITE),
                                        ),
                                )
                                .padding(10)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                    Column::new()
                                        .spacing(8)
                                        .push(Text::new("Video Size").color(WHITE).size(14))
                                        .push(
                                            Row::new()
                                                .spacing(10)
                                                .push(Text::new("Width:").color(WHITE))
                                                .push(
                                                    Slider::new(
                                                        800.0..=1920.0,
                                                        self.video_width,
                                                        Message::VideoWidthChanged,
                                                    )
                                                    .step(10.0)
                                                    .width(200.0),
                                                )
                                                .push(
                                                    Text::new(format!("{:.0}px", self.video_width))
                                                        .color(WHITE),
                                                ),
                                        )
                                        .push(
                                            Row::new()
                                                .spacing(10)
                                                .push(Text::new("Height:").color(WHITE))
                                                .push(
                                                    Slider::new(
                                                        450.0..=1080.0,
                                                        self.video_height,
                                                        Message::VideoHeightChanged,
                                                    )
                                                    .step(10.0)
                                                    .width(200.0),
                                                )
                                                .push(
                                                    Text::new(format!("{:.0}px", self.video_height))
                                                        .color(WHITE),
                                                ),
                                        ),
                                )
                                .padding(10)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                                .spacing(10)
                                                .push(Text::new("Width:").color(WHITE))
                                                .push(
                                                    Slider::new(
                                                        800.0..=1920.0,
                                                        self.video_width,
                                                        Message::VideoWidthChanged,
                                                    )
                                                    .step(10.0)
                                                    .width(200.0),
                                                )
                                                .push(
                                                    Text::new(format!("{:.0}px", self.video_width))
                                                        .color(WHITE),
                                                ),
                                        )
                                        .push(
                                            Row::new()
                                                .spacing(10)
                                                .push(Text::new("Height:").color(WHITE))
                                                .push(
                                                    Slider::new(
                                                        450.0..=1080.0,
                                                        self.video_height,
                                                        Message::VideoHeightChanged,
                                                    )
                                                    .step(10.0)
                                                    .width(200.0),
                                                )
                                                .push(
                                                    Text::new(format!("{:.0}px", self.video_height))
                                                        .color(WHITE),
                                                ),
                                        ),
                                )
                                .padding(10)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                                            width: 1.0,
                                            radius: 8.0.into(),
                                        },
                                        shadow: iced::Shadow::default(),
                                        text_color: None,
                                    }
                                }),
                            )
                                .padding(10)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                        .spacing(10)
                                        .push(Text::new("Video Size").color(WHITE))
                                        .push(
                                            Text::new(format!(
                                                "{:.0}x{:.0}",
                                                self.video_width, self.video_height
                                            ))
                                            .color(WHITE),
                                        ),
                                )
                                .padding(10)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                    Column::new()
                                        .spacing(8)
                                        .push(Text::new("Subtitle Position").color(WHITE).size(14))
                                        .push(
                                            Row::new()
                                                .spacing(10)
                                                .push(Text::new("Offset:").color(WHITE))
                                                .push(
                                                    Slider::new(
                                                        -30.0..=30.0,
                                                        self.subtitle_offset,
                                                        Message::SubtitleOffsetChanged,
                                                    )
                                                    .step(0.1)
                                                    .width(200.0),
                                                )
                                                .push(
                                                    Text::new(format!(
                                                        "{:.1}s",
                                                        self.subtitle_offset
                                                    ))
                                                    .color(WHITE),
                                                ),
                                        )
                                        .push(
                                            Row::new()
                                                .spacing(10)
                                                .push(Text::new("Vertical:").color(WHITE))
                                                .push(
                                                    Slider::new(
                                                        0.0..=500.0,
                                                        self.subtitle_offset_vertical,
                                                        Message::SubtitleOffsetVerticalChanged,
                                                    )
                                                    .step(5.0)
                                                    .width(200.0),
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
                                                .push(Text::new("Horizontal:").color(WHITE))
                                                .push(
                                                    Slider::new(
                                                        -400.0..=200.0,
                                                        self.subtitle_offset_horizontal,
                                                        Message::SubtitleOffsetHorizontalChanged,
                                                    )
                                                    .step(5.0)
                                                    .width(200.0),
                                                )
                                                .push(
                                                    Text::new(format!(
                                                        "{:.0}px",
                                                        self.subtitle_offset_horizontal
                                                    ))
                                                    .color(WHITE),
                                                ),
                                        ),
                                )
                                .padding(10)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                .padding(10)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                )
                                .padding(10)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                                            width: 1.0,
                                            radius: 8.0.into(),
                                        },
                                        shadow: iced::Shadow::default(),
                                        text_color: None,
                                    }
                                }),
                            ),
                    )
                    .padding(15)
                    .style(|_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.15, 0.15, 0.15, 0.8,
                        ))),
                        border: iced::border::Border {
                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                            width: 1.0,
                            radius: 10.0.into(),
                        },
                        shadow: iced::Shadow::default(),
                        text_color: None,
                    }),
                )
                .width(600)
                .height(iced::Length::Shrink),
        )
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.0, 0.0, 0.0, 0.95,
            ))),
            border: iced::border::Border {
                color: iced::Color::from_rgba(0.4, 0.4, 0.4, 0.8),
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
        .width(600)
        .into()
    }

    fn video_info_window(&self) -> Element<'_, Message> {
        let info_text = self
            .video_info_text
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Running gst-discoverer-1.0...");

        Container::new(
            Column::new()
                .spacing(15)
                .padding(20)
                .push(
                    Row::new()
                        .spacing(10)
                        .align_y(iced::Alignment::Center)
                        .push(Text::new("Video Info").size(24).color(WHITE))
                        .push(Row::new().width(iced::Length::Fill))
                        .push(
                            Button::new(Text::new("✕"))
                                .on_press(Message::ToggleVideoInfo)
                                .width(40.0),
                        ),
                )
                .push(
                    Container::new(
                        Scrollable::new(Text::new(info_text).size(13).color(WHITE).font(
                            iced::Font {
                                family: iced::font::Family::Monospace,
                                ..Default::default()
                            },
                        ))
                        .height(500)
                        .width(700)
                        .direction(
                            iced::widget::scrollable::Direction::Vertical(
                                iced::widget::scrollable::Scrollbar::new(),
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
                ),
        )
        .width(800)
        .height(iced::Length::Shrink)
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
        .width(750)
        .into()
    }

    fn file_panel_window(&self) -> Element<'_, Message> {
        Container::new(
            Column::new()
                .spacing(15)
                .padding(20)
                .push(
                    Row::new()
                        .spacing(10)
                        .align_y(iced::Alignment::Center)
                        .push(Text::new("File Options").size(24).color(WHITE))
                        .push(Row::new().width(iced::Length::Fill))
                        .push(
                            Button::new(Text::new("✕"))
                                .on_press(Message::ToggleFilePanel)
                                .width(40.0),
                        ),
                )
                .push(
                    Container::new(
                        Column::new()
                            .spacing(12)
                            .push(
                                Container::new(
                                    Row::new()
                                        .spacing(10)
                                        .align_y(iced::Alignment::Center)
                                        .push(Text::new("Open Video File").color(WHITE))
                                        .push(Row::new().width(iced::Length::Fill))
                                        .push(
                                            Button::new(Text::new("Open"))
                                                .on_press(Message::Open)
                                                .width(100.0),
                                        ),
                                )
                                .padding(12)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                        .spacing(10)
                                        .align_y(iced::Alignment::Center)
                                        .push(Text::new("Open Video Folder").color(WHITE))
                                        .push(Row::new().width(iced::Length::Fill))
                                        .push(
                                            Button::new(Text::new("Open"))
                                                .on_press(Message::OpenVidFolder)
                                                .width(100.0),
                                        ),
                                )
                                .padding(12)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                        .spacing(10)
                                        .align_y(iced::Alignment::Center)
                                        .push(Text::new("GTK Chooser").color(WHITE))
                                        .push(Row::new().width(iced::Length::Fill))
                                        .push(
                                            Button::new(Text::new("Open"))
                                                .on_press(Message::SpawnGtkChooser(
                                                    self.video_folder_better.folder.clone(),
                                                ))
                                                .width(100.0),
                                        ),
                                )
                                .padding(12)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                        .spacing(10)
                                        .align_y(iced::Alignment::Center)
                                        .push(Text::new("Open Subtitle File").color(WHITE))
                                        .push(Row::new().width(iced::Length::Fill))
                                        .push(
                                            Button::new(Text::new("Open"))
                                                .on_press(Message::OpenSubtitle)
                                                .width(100.0),
                                        ),
                                )
                                .padding(12)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                        .spacing(10)
                                        .align_y(iced::Alignment::Center)
                                        .push(Text::new("Open Subtitle Folder").color(WHITE))
                                        .push(Row::new().width(iced::Length::Fill))
                                        .push(
                                            Button::new(Text::new("Open"))
                                                .on_press(Message::OpenSubFolder)
                                                .width(100.0),
                                        ),
                                )
                                .padding(12)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                        .spacing(10)
                                        .align_y(iced::Alignment::Center)
                                        .push(Text::new("Screenshot").color(WHITE))
                                        .push(Row::new().width(iced::Length::Fill))
                                        .push(
                                            Button::new(Text::new("Capture"))
                                                .on_press(Message::TakeScreenshotURI)
                                                .width(100.0),
                                        ),
                                )
                                .padding(12)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.2, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
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
                                        .spacing(10)
                                        .align_y(iced::Alignment::Center)
                                        .push(Text::new("Quit").color(WHITE))
                                        .push(Row::new().width(iced::Length::Fill))
                                        .push(
                                            Button::new(Text::new("Quit"))
                                                .on_press(Message::Quit)
                                                .width(100.0),
                                        ),
                                )
                                .padding(12)
                                .style(|_theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.6, 0.2, 0.2, 0.6),
                                        )),
                                        border: iced::border::Border {
                                            color: iced::Color::from_rgba(1.0, 0.5, 0.5, 0.3),
                                            width: 1.0,
                                            radius: 8.0.into(),
                                        },
                                        shadow: iced::Shadow::default(),
                                        text_color: None,
                                    }
                                }),
                            ),
                    )
                    .padding(15)
                    .style(|_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.15, 0.15, 0.15, 0.8,
                        ))),
                        border: iced::border::Border {
                            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                            width: 1.0,
                            radius: 10.0.into(),
                        },
                        shadow: iced::Shadow::default(),
                        text_color: None,
                    }),
                )
                .width(500)
                .height(iced::Length::Shrink),
        )
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.0, 0.0, 0.0, 0.95,
            ))),
            border: iced::border::Border {
                color: iced::Color::from_rgba(0.4, 0.4, 0.4, 0.8),
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
        .width(500)
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
