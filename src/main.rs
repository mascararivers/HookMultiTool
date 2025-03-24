use anyhow::Result;
use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{button, checkbox, column, container, text, text_input},
};
use log::info;
use weboxide::api::{Embed as ApiEmbed, WebhookClient};

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
}

#[derive(Default)]
struct Hook {
    hook_url: String,
    message: String,
    avatar_url: Option<String>,
    username: Option<String>,
    has_embed: bool,
    embed: Option<Embed>,
}

#[derive(serde::Serialize, Default, Clone, Debug)]
struct Field {
    name: String,
    value: String,
    inline: bool,
}
#[derive(serde::Serialize, Default, Clone, Debug)]
struct Embed {
    title: String,
    description: String,
    fields: Vec<Field>,
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
                    ..Default::default()
                };

                let optional_embed = if self.has_embed {
                    Some(api_embed)
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
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let btn: button::Button<'_, Message, iced::Theme, iced::Renderer> =
            button(text("Send")).on_press(Message::Send);

        let embed_checkbox = checkbox("Has Embed", self.has_embed).on_toggle(Message::HasEmbed);

        let mut clmn = column![
            text_input("Webhook URL", &self.hook_url).on_input(Message::ChangeHookUrl),
            text_input("Message", &self.message).on_input(Message::ChangeHookContent),
            text_input("Avatar URL", self.avatar_url.as_deref().unwrap_or(""))
                .on_input(Message::ChangeAvatarUrl),
            text_input("Username", self.username.as_deref().unwrap_or(""))
                .on_input(Message::ChangeUsername),
            embed_checkbox
        ];
        if self.has_embed {
            clmn = clmn.push({
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
                        .on_input(Message::ChangeEmbedTitle),
                        text_input(
                            "Embed Description",
                            &self.embed.as_ref().unwrap_or(&Embed::default()).description
                        )
                        .on_input(Message::ChangeEmbedDescription),
                    ]
                })
            });
        }
        clmn = clmn.push(btn);
        container(clmn)
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

    //user_input().await;
    iced::application("Hook Multitool", Hook::update, Hook::view)
        .theme(theme)
        .window_size((400.0, 400.0))
        .centered()
        .run()
}
