use anyhow::{Ok, Result};
use iced::widget::checkbox;
use iced::Length::Fill;
use iced::widget::column;
use iced::widget::container;
use iced::widget::{button, text, text_input};
use iced::{Element, Task};
use serde_json::json;

#[derive(Debug, Clone)]
enum Message {
    Send,
    Response,
    ChangeHookUrl(String),
    ChangeMessage(String),
    ChangeAvatarUrl(String),
    ChangeUsername(String),
    HasEmbed(bool)
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

#[derive(serde::Serialize, Default, Clone)]
struct Field {
    name: String,
    value: String,
    inline: bool
}
#[derive(serde::Serialize, Default, Clone)]
struct Embed {
    title: String,
    description: String,
    fields: Vec<Field>,
}

async fn request(message: String, avatar_url: String, username: String, hook_url: String,
embed: Option<Embed>) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        let e = if embed.is_none() {
            json!([])
        } 
        else {
            let embed_some = embed.unwrap();
            json!([
                {
                    "title": embed_some.title,
                    "type": "rich",
                    "description": embed_some.description,
                    "fields": embed_some.fields
                }
            ])
        };

        let payload = json!({
            "content": message,
            "avatar_url": avatar_url,
            "username": username,
            "embeds": e
        });

        let client = reqwest::Client::new();
        let res = client.post(hook_url).json(&payload).send().await.unwrap();
        println!("{:?}", res.text().await);
    });

    return Ok(());
}


impl Hook {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Send => {
                let message = self.message.clone();
                let avatar_url = self.avatar_url.clone().unwrap_or_default();
                let username = self.username.clone().unwrap_or_default();
                let hook_url = self.hook_url.clone();
                let embed = if !self.has_embed {
                    None
                } else {
                    Some(Embed::default())
                };

                let embed_title = embed.clone().unwrap().title;

                return Task::perform(request(message, avatar_url, username, hook_url, embed), |_| {
                    Message::Response
                });
            }
            Message::Response => {
                return Task::none();
            }
            Message::ChangeHookUrl(hook) => self.hook_url = hook,
            Message::ChangeAvatarUrl(url) => self.avatar_url = Some(url), 
            Message::ChangeMessage(msg) => self.message = msg,
            Message::ChangeUsername(usr) => self.username = Some(usr),
            Message::HasEmbed(has) => self.has_embed = has
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let btn = button(text("Send")).on_press(Message::Send);

        let embed_checkbox = checkbox("Has Embed", self.has_embed)
        .on_toggle(Message::HasEmbed);

        let clmn = column![
            text_input("Webhook URL", &self.hook_url).on_input(Message::ChangeHookUrl),
            text_input("Message", &self.message).on_input(Message::ChangeMessage),
            text_input("Avatar URL", self.avatar_url.as_deref().unwrap_or("")).on_input(Message::ChangeAvatarUrl),
            text_input("Username", self.username.as_deref().unwrap_or("")).on_input(Message::ChangeUsername),
            embed_checkbox,
            btn
        ];
        container(clmn)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center(Fill)
            .into()
    }
}

#[tokio::main]
async fn main() -> iced::Result {
    let theme = |_s: &Hook| iced::Theme::Dark;

    //user_input().await;
    iced::application("Hook Multitool", Hook::update, Hook::view)
        .theme(theme)
        .window_size((400.0, 400.0))
        .centered()
        .run()
}
