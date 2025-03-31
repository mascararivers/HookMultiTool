use anyhow::Result;
use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{button, checkbox, column, container, row, text, text_input},
};
use serde::Serialize;
use weboxide::api::{Embed as ApiEmbed, Field, Footer, Image, Thumbnail, WebhookClient};

#[derive(Debug, Clone)]
enum Message {
    Send,
    Response,
    ChangeHookUrl(String),
    ChangeHookContent(String),
    ChangeAvatarUrl(String),
    ChangeUsername(String),
    HasEmbed(bool),

    ChangeEmbedTitle(String),
    ChangeEmbedDescription(String),

    AdvancedToggle(bool),

    HasFooter(bool),
    ChangeFooterText(String),
    ChangeFooterIcon(String),

    HasImage(bool),
    ChangeImageUrl(String),

    HasThumbnail(bool),
    ChangeThumbnailUrl(String)
}

#[derive(Default)]
struct Hook {
    hook_url: String,
    message: String,
    avatar_url: Option<String>,
    username: Option<String>,
    has_embed: bool,
    embed: Option<Embed>,
    advanced_mode: bool,
    has_footer: bool,
    has_image: bool,
    has_thumbnail: bool,
}

#[derive(Serialize, Default, Clone, Debug)]
struct Embed {
    title: String,
    description: String,
    fields: Vec<Field>,
    image: Option<Image>,
    thumbnail: Option<Thumbnail>,
    footer: Option<Footer>,
    timestamp: Option<bool>,
}

async fn request(
    message: String,
    avatar_url: String,
    username: String,
    hook_url: String,
    embed: Option<ApiEmbed>,
) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        let http_client = reqwest::Client::new();
        let client = WebhookClient::new(
            http_client,
            hook_url,
            Some(avatar_url),
            Some(username),
            if embed.is_none() {
                vec![]
            } else {
                vec![embed.unwrap()]
            },
        );

        client.send_message(message).await.unwrap();
    });

    Ok(())
}

impl Hook {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Send => {
                let message = self.message.clone();
                let avatar_url = self.avatar_url.clone().unwrap_or_default();
                let username = self.username.clone().unwrap_or_default();
                let hook_url = self.hook_url.clone();
                let embed = &self.embed;

                let api_embed = ApiEmbed {
                    title: embed.as_ref().unwrap_or(&Embed::default()).title.clone(),
                    description: Some(
                        embed
                            .as_ref()
                            .unwrap_or(&Embed::default())
                            .description
                            .clone(),
                    ),
                    // footer: embed.as_ref().unwrap_or(&Embed::default()).footer.clone(),
                    ..Default::default()
                };

                let advanced_api_embed = ApiEmbed {
                    title: embed.as_ref().unwrap_or(&Embed::default()).title.clone(),
                    description: Some(
                        embed
                            .as_ref()
                            .unwrap_or(&Embed::default())
                            .description
                            .clone(),
                    ),
                    fields: self
                        .embed
                        .as_ref()
                        .unwrap_or(&Embed::default())
                        .fields
                        .clone(),
                    image: embed
                        .as_ref()
                        .unwrap_or(&Embed::default())
                        .image
                        .clone(),
                    thumbnail: embed
                        .as_ref()
                        .unwrap_or(&Embed::default())
                        .thumbnail
                        .clone(),
                    footer: embed.as_ref().unwrap_or(&Embed::default()).footer.clone(),
                    ..Default::default()
                };

                let optional_embed = if self.has_embed {
                    if self.advanced_mode {
                        Some(advanced_api_embed)
                    } else {
                        Some(api_embed)
                    }
                } else {
                    None
                };

                return Task::perform(
                    request(message, avatar_url, username, hook_url, optional_embed),
                    |_| Message::Response,
                );
            }
            Message::Response => {
                return Task::none();
            }
            Message::ChangeHookUrl(hook) => self.hook_url = hook,
            Message::ChangeAvatarUrl(url) => self.avatar_url = Some(url),
            Message::ChangeHookContent(msg) => self.message = msg,
            Message::ChangeUsername(usr) => self.username = Some(usr),
            Message::HasEmbed(has) => {
                self.has_embed = has;
                self.embed = if has { Some(Embed::default()) } else { None }
            }

            Message::ChangeEmbedTitle(title) => {
                if self.embed.is_none() {
                    return Task::none();
                }
                if let Some(embed) = &mut self.embed {
                    embed.title = title;
                }
            }
            Message::ChangeEmbedDescription(description) => {
                if self.embed.is_none() {
                    return Task::none();
                }

                if let Some(embed) = &mut self.embed {
                    embed.description = description;
                }
            }

            // ~ Advanced mode ~
            Message::AdvancedToggle(toggled) => {
                self.advanced_mode = toggled;
            }

            // ~ Image and Thumbnail ~ needs width and height customization

            Message::HasImage(has) => {
                if self.has_embed { 
                    self.has_image = has;       
                } else {
                    return Task::none();
                }
            }
            Message::ChangeImageUrl(url) => {
                if self.embed.is_none() {
                    return Task::none();
                } else {
                    self.embed.clone().unwrap().image = Some(Image { url: url, height: Some(32), width: Some(32) });
                }
            }

            Message::HasThumbnail(has) => {
                if self.has_embed {
                    self.has_thumbnail = has;
                } else {
                    return Task::none();
                }
            }

            Message::ChangeThumbnailUrl(url) => {
                if self.embed.is_none() {
                    return Task::none();
                } else {
                    self.embed.clone().unwrap().thumbnail = Some(Thumbnail { url: url, height: Some(32), width: Some(32) });
                }
            }

            // ~ Footer stuff ~
            Message::HasFooter(has) => {
                self.has_footer = has;
                self.embed.as_mut().unwrap().footer =
                    if has { Some(Footer::default()) } else { None };
            }
            Message::ChangeFooterText(text) => {
                if self.embed.is_none() {
                    return Task::none();
                }
                if let Some(embed) = &mut self.embed {
                    if self.has_footer {
                        if embed.footer.is_none() {
                            embed.footer = Some(Footer::default());
                        }
                        embed.footer.as_mut().unwrap().text = Some(text);
                        println!("Footer text: {:?}", embed.footer.as_ref().unwrap().text);
                    }
                }
            }
            Message::ChangeFooterIcon(url) => {
                if self.embed.is_none() {
                    return Task::none();
                }
                if let Some(embed) = &mut self.embed {
                    if self.has_footer {
                        if embed.footer.is_none() {
                            embed.footer = Some(Footer::default());
                        }
                        embed.footer.as_mut().unwrap().icon_url = Some(url);
                        println!("Footer icon: {:?}", embed.footer.as_ref().unwrap().icon_url);
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        const WIDTH: f32 = 400.0 * 0.95;

        let btn: button::Button<'_, Message, iced::Theme, iced::Renderer> =
            button(text("Send")).on_press(Message::Send);

        let embed_checkbox = checkbox("Has Embed", self.has_embed).on_toggle(Message::HasEmbed);

        let advanced_mode: iced::widget::Checkbox<'_, Message, iced::Theme, iced::Renderer> =
            checkbox("Advanced Mode", self.advanced_mode).on_toggle(Message::AdvancedToggle);

        /*let text: iced::widget::Text<iced::Theme, iced::Renderer> = text(if self.has_embed {
            self.embed.as_ref().unwrap().title.clone()
        } else {
            "".to_string()
        });
        */
 // For future embed preview

        let mut column = column![
            text_input("Webhook URL", &self.hook_url)
                .on_input(Message::ChangeHookUrl)
                .width(WIDTH),
            text_input("Message", &self.message)
                .on_input(Message::ChangeHookContent)
                .width(WIDTH),
            text_input("Avatar URL", self.avatar_url.as_deref().unwrap_or(""))
                .on_input(Message::ChangeAvatarUrl)
                .width(WIDTH),
            text_input("Username", self.username.as_deref().unwrap_or(""))
                .on_input(Message::ChangeUsername)
                .width(WIDTH),
            embed_checkbox
        ];
        column = column.push(btn);

        if self.has_embed {
            column = column.push({
                container({
                    column![
                        text_input(
                            "Embed Title",
                            self.embed
                                .as_ref()
                                .unwrap_or(&Embed::default())
                                .title
                                .as_str()
                        )
                        .width(WIDTH)
                        .on_input(Message::ChangeEmbedTitle),
                        text_input(
                            "Embed Description",
                            &self.embed.as_ref().unwrap_or(&Embed::default()).description
                        )
                        .width(WIDTH)
                        .on_input(Message::ChangeEmbedDescription),
                        advanced_mode
                    ]
                })
            });
        }
        // Advanced mode for extra embed stuff
        if self.advanced_mode {
            column = column.push({
                container({
                    column![
                        checkbox("Has Footer", self.has_footer).on_toggle(Message::HasFooter),
                        checkbox("Has Image", self.has_image).on_toggle(Message::HasImage),
                        checkbox("Has Thumbnail", self.has_thumbnail).on_toggle(Message::HasThumbnail)
                    ]
                })
            })
        }
        if self.has_footer {
            column = column.push({
                container({
                    column![
                        text_input(
                            "Footer Text",
                            self.embed
                                .as_ref()
                                .unwrap_or(&Embed::default())
                                .footer
                                .as_ref()
                                .unwrap_or(&Footer::default())
                                .text
                                .as_deref()
                                .unwrap_or("")
                        )
                        .on_input(Message::ChangeFooterText)
                        .width(WIDTH),
                        text_input(
                            "Footer Icon URL",
                            self.embed
                                .as_ref()
                                .unwrap_or(&Embed::default())
                                .footer
                                .as_ref()
                                .unwrap_or(&Footer::default())
                                .icon_url
                                .clone()
                                .unwrap_or("".to_string())
                                .as_str()
                        )
                        .on_input(Message::ChangeFooterIcon)
                        .width(WIDTH)
                    ]
                })
            })
        }
        if self.has_image {
            column = column.push({
                container({
                    column![
                    text_input(
                        "Image URL",
                        <std::string::String as AsRef<_>>::as_ref(&self.embed
                            .as_ref()
                            .unwrap_or(&Embed::default())
                            .image
                            .clone()
                            .unwrap_or(Image::default())
                            .url)
                    )
                    .on_input(Message::ChangeImageUrl)
                    .width(WIDTH)
                    ]
                })
            })
        }
        if self.has_thumbnail {
            column = column.push({
                container({
                    column![
                    text_input(
                        "Thumbnail URL",
                        <std::string::String as AsRef<_>>::as_ref(&self.embed
                            .as_ref()
                            .unwrap_or(&Embed::default())
                            .thumbnail
                            .clone()
                            .unwrap_or(Thumbnail::default())
                            .url)
                    )
                    .on_input(Message::ChangeThumbnailUrl)
                    .width(WIDTH)
                    ]
                })
            })
        }

        let row = row![column].spacing(10);

        container(row)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center(Fill)
            .into()
    }
}

#[tokio::main]
async fn main() -> iced::Result {
    env_logger::init();
    let theme = |_s: &Hook| iced::Theme::Dark;

    iced::application("Hook Multitool", Hook::update, Hook::view)
        .theme(theme)
        .window_size((800.0, 600.0))
        .centered()
        .run()
}
