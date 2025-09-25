use iced::widget::{Button, Column, Container, Row, Slider, Text, button, text_input};
use iced::{Element, Padding};
use iced_video_player::VideoPlayer;
use std::path::PathBuf;

use crate::app::{App, Message};

impl App {
    pub fn view(&self) -> Element<Message> {
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

        // so basically the subtitles are updateing every frame when we dont need that we only need
        // them to update when their is a new subtitles, when the subtitles message is given
        // ok on NewSub i need to update the actince text
        let mut heresubdudebud = String::new();
        match self.active_subtitle.clone() {
            Some(text) => {
                //let sub_text = text;
                heresubdudebud = text.replace("&apos;", "'");
                println!("{heresubdudebud}");
            }
            None => {
                //println!("no text yet bub");
            }
        }

        Column::new()
            .push(
                Container::new(
                    VideoPlayer::new(&self.video)
                        .on_end_of_stream(Message::EndOfStream)
                        .on_new_frame(Message::NewFrame)
                        .on_subtitle_text(Message::NewSub),
                )
                .align_x(iced::Alignment::Start)
                .align_y(iced::Alignment::Center)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .padding(Padding::new(0.0).left(150.0).top(100.0)),
                //  .padding(Padding::new(0.0).top(100.0)),
            )
            // .push(
            //     Container::new(Text::new(filestodisplay).size(8))
            //         .align_x(iced::Alignment::End)
            //         .align_y(iced::Alignment::End),
            // )
            // .push(my_column())
            .push(
                Container::new(Text::new(heresubdudebud).size(50))
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center)
                    .padding(iced::Padding::new(0.0).left(150.0)),
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
                        .push(
                            Column::new()
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
}

fn my_column<'a>() -> Element<'a, Message> {
    Column::new()
        .push("a column can be used to ")
        .push("lay out widgets vertically")
        .spacing(10)
        .into()
}
