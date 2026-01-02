use iced::Alignment;
use iced::Font;
use iced::Length;

use iced::widget::{button, text_input, Button, Column, Container, Row, Slider, Text};
use iced::{Element, Padding};

use iced::widget::Stack;
use iced_aw::style::colors::WHITE;
use iced_aw::{selection_list::SelectionList, style::selection_list::primary};
use iced_video_player::VideoPlayer;

use crate::app::{App, Message};

impl App {
    pub fn view(&self) -> Element<Message> {
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

        let subtitles_file = self
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
        .width(iced::Length::Fixed(1700.0))
        .height(iced::Length::Fixed(900.0));

        let subtitle_layer = Container::new(
            Text::new(heresubdudebud).size(35).color(WHITE), // the subtitle
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::End)
        .padding(iced::Padding::new(0.0).bottom(self.subtitle_offset as f32));

        // video first then text
        let overlay_stack = Stack::new().push(video_layer).push(subtitle_layer);

        Column::new()
            .push(
                Container::new(overlay_stack)
                    .align_x(iced::Alignment::Start)
                    .align_y(iced::Alignment::Center)
                    // Padding for the whole stack
                    .padding(Padding::new(00.0).left(20.0).top(60.0)),
            )
            .push(self.list())
            .push(
                Container::new(
                    text_input("Enter a number...", &self.value)
                        .on_input(Message::ValueChanged)
                        .on_submit(Message::SubmitPressed)
                        .padding(5)
                        .size(15),
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
            .push(Container::new(
                Column::new()
                    .push(
                        Row::new()
                            .spacing(5)
                            .push(
                                Button::new(Text::new(if self.muted { "Mute" } else { "Unmute" }))
                                    .on_press(Message::ToggleMute),
                            )
                            .push(Button::new(Text::new("quit")).on_press(Message::Quit))
                            .push(
                                Column::new()
                                    .push(
                                        Container::new(Text::new(subtitles_file).size(13))
                                            .align_x(iced::Alignment::Center)
                                            .align_y(iced::Alignment::Center)
                                            .padding(
                                                iced::Padding::new(0.0).left(20.0).right(100.0),
                                            ),
                                    )
                                    .push(
                                        Container::new(Text::new(filename_text).size(13))
                                            .align_x(iced::Alignment::Center)
                                            .align_y(iced::Alignment::Center)
                                            .padding(
                                                iced::Padding::new(0.0).left(20.0).right(100.0),
                                            ),
                                    ),
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
                            )))
                            .push(Text::new("Sub Offset:"))
                            .push(
                                Slider::new(
                                    0.0..=300.0,
                                    self.subtitle_offset,
                                    Message::SubtitleOffsetChanged,
                                )
                                .step(5.0)
                                .width(150.0),
                            )
                            .push(Text::new(format!("{:.0}px", self.subtitle_offset))),
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
                            .push(button("OWNSUBS").on_press(Message::UsingOwnSubs))
                            .push(button("Open").on_press(Message::Open))
                            .push(button("OPEN VID FOLDER").on_press(Message::OpenVidFolder))
                            .push(button("OPEN SUB FOLDER").on_press(Message::OpenSubFolder))
                            .push(button("Open Subtitles").on_press(Message::OpenSubtitle))
                            .push(self.next_button())
                            .push(button("last vid").on_press(Message::OpenLast))
                            .push(
                                button("press to add at selection")
                                    .on_press(Message::AddAtSelection),
                            )
                            .push(self.audio_track_button())
                            .push(self.subtitle_track_button())
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
                    ),
            ))
            .into()
    }

    fn list(&self) -> Element<'_, Message> {
        let selection_list = SelectionList::new_with(
            &self.vec[..],
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
            .height(Length::Fill)
            .align_x(Alignment::End)
            .align_y(Alignment::Start)
            .padding(20)
            .into()
    }

    pub fn next_button(&self) -> Element<Message> {
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
}

fn my_column<'a>() -> Element<'a, Message> {
    Column::new()
        .push("a column can be used to ")
        .push("lay out widgets vertically")
        .spacing(10)
        .into()
}
