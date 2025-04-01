use anyhow::Result;
use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{button, checkbox, column, container, row, text, text_input},
};
use serde::Serialize;
use std::sync::OnceLock;
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
    ChangeThumbnailUrl(String),
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

// impl Into<String> indicates any type that can be converted into a String, such as
// &str, String, &String, etc.
async fn request(
    message: impl Into<String>,
    avatar_url: impl Into<String>,
    username: impl Into<String>,
    hook_url: impl Into<String>,
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
            hook_url.into(),
            Some(avatar_url.into()),
            Some(username.into()),
            embed.into_iter().collect(),
        );

        client.send_message(message.into()).await.unwrap();
    });

    Ok(())
}

impl Hook {
    fn default_embed() -> &'static Embed {
        static DEFAULT: OnceLock<Embed> = OnceLock::new();
        DEFAULT.get_or_init(|| Embed::default())
    }

    /// Returns a reference to the embed or a default embed.
    fn embed_or_default(&self) -> &Embed {
        self.embed.as_ref().unwrap_or_else(|| Self::default_embed())
    }

    /// Returns a reference to the footer (if any) or a default footer.
    fn footer_or_default(&self) -> &Footer {
        static DEFAULT: OnceLock<Footer> = OnceLock::new();
        let default_footer = DEFAULT.get_or_init(Footer::default);
        self.embed_or_default()
            .footer
            .as_ref()
            .unwrap_or(default_footer)
    }

    /// Returns the URL for the image if available.
    fn image_url(&self) -> &str {
        self.embed
            .as_ref()
            .and_then(|e| e.image.as_ref())
            .map(|i| i.url.as_str())
            .unwrap_or("")
    }

    /// Returns the URL for the thumbnail if available.
    fn thumbnail_url(&self) -> &str {
        self.embed
            .as_ref()
            .and_then(|e| e.thumbnail.as_ref())
            .map(|t| t.url.as_str())
            .unwrap_or("")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Send => {
                let default_embed = &Embed::default();
                let embed_ref = self.embed.as_ref().unwrap_or(default_embed);

                let mut api_embed = ApiEmbed {
                    title: embed_ref.title.clone(),
                    description: Some(embed_ref.description.clone()),
                    ..ApiEmbed::default()
                };

                if self.advanced_mode {
                    api_embed.fields = embed_ref.fields.clone();
                    api_embed.image = embed_ref.image.clone();
                    api_embed.thumbnail = embed_ref.thumbnail.clone();
                    api_embed.footer = embed_ref.footer.clone();
                }

                let optional_embed = self.has_embed.then_some(api_embed);

                return Task::perform(
                    request(
                        &self.message,
                        self.avatar_url.as_deref().unwrap_or(""),
                        self.username.as_deref().unwrap_or(""),
                        &self.hook_url,
                        optional_embed,
                    ),
                    |_| Message::Response,
                );
            }
            Message::Response => return Task::none(),
            Message::ChangeHookUrl(hook) => {
                self.hook_url = hook;
            }
            Message::ChangeAvatarUrl(url) => {
                self.avatar_url = Some(url);
            }
            Message::ChangeHookContent(msg) => {
                self.message = msg;
            }
            Message::ChangeUsername(usr) => {
                self.username = Some(usr);
            }
            Message::HasEmbed(has) => {
                self.has_embed = has;
                self.embed = has.then(Embed::default);
            }
            Message::ChangeEmbedTitle(title) => {
                if let Some(embed) = &mut self.embed {
                    embed.title = title;
                }
            }
            Message::ChangeEmbedDescription(description) => {
                if let Some(embed) = &mut self.embed {
                    embed.description = description;
                }
            }
            Message::AdvancedToggle(toggled) => {
                self.advanced_mode = toggled;
            }
            Message::HasImage(has) => {
                self.has_image = has && self.has_embed;
            }
            Message::ChangeImageUrl(url) => {
                if let Some(embed) = &mut self.embed {
                    embed.image = Some(Image {
                        url,
                        height: Some(32),
                        width: Some(32),
                    });
                }
            }
            Message::HasThumbnail(has) => {
                self.has_thumbnail = has && self.has_embed;
            }
            Message::ChangeThumbnailUrl(url) => {
                if let Some(embed) = &mut self.embed {
                    embed.thumbnail = Some(Thumbnail {
                        url,
                        height: Some(32),
                        width: Some(32),
                    });
                }
            }
            Message::HasFooter(has) => {
                self.has_footer = has;
                if let Some(embed) = &mut self.embed {
                    embed.footer = has.then(Footer::default);
                }
            }
            Message::ChangeFooterText(text) => {
                if let Some(embed) = &mut self.embed {
                    if self.has_footer {
                        let footer = embed.footer.get_or_insert_with(Footer::default);
                        footer.text = Some(text);
                    }
                }
            }
            Message::ChangeFooterIcon(url) => {
                if let Some(embed) = &mut self.embed {
                    if self.has_footer {
                        let footer = embed.footer.get_or_insert_with(Footer::default);
                        footer.icon_url = Some(url);
                    }
                }
            }
        };
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        const WIDTH: f32 = 400.0 * 0.95;

        let btn = button(text("Send")).on_press(Message::Send);
        let embed_checkbox = checkbox("Has Embed", self.has_embed).on_toggle(Message::HasEmbed);
        let advanced_mode =
            checkbox("Advanced Mode", self.advanced_mode).on_toggle(Message::AdvancedToggle);

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
            column = column.push(container(column![
                text_input("Embed Title", self.embed_or_default().title.as_str())
                    .width(WIDTH)
                    .on_input(Message::ChangeEmbedTitle),
                text_input(
                    "Embed Description",
                    self.embed_or_default().description.as_str()
                )
                .width(WIDTH)
                .on_input(Message::ChangeEmbedDescription),
                advanced_mode
            ]));
        }

        if self.advanced_mode {
            column = column.push(container(column![
                checkbox("Has Footer", self.has_footer).on_toggle(Message::HasFooter),
                checkbox("Has Image", self.has_image).on_toggle(Message::HasImage),
                checkbox("Has Thumbnail", self.has_thumbnail).on_toggle(Message::HasThumbnail)
            ]));
        }

        if self.has_footer {
            column = column.push(container(column![
                text_input(
                    "Footer Text",
                    self.footer_or_default().text.as_deref().unwrap_or("")
                )
                .on_input(Message::ChangeFooterText)
                .width(WIDTH),
                text_input(
                    "Footer Icon URL",
                    self.footer_or_default().icon_url.as_deref().unwrap_or("")
                )
                .on_input(Message::ChangeFooterIcon)
                .width(WIDTH)
            ]));
        }

        if self.has_image {
            column = column.push(container(column![
                text_input("Image URL", self.image_url())
                    .on_input(Message::ChangeImageUrl)
                    .width(WIDTH)
            ]));
        }

        if self.has_thumbnail {
            column = column.push(container(column![
                text_input("Thumbnail URL", self.thumbnail_url())
                    .on_input(Message::ChangeThumbnailUrl)
                    .width(WIDTH)
            ]));
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
