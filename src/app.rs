use crate::config::AppConfig;
use anyhow::Result;
use iced::{
    alignment, executor,
    widget::{
        button, column, container, row, scrollable, text, text_input, Column,
    },
    Alignment, Application, Command, Element, Length, Subscription, Theme,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

impl ChatMessage {
    pub fn new(role: String, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    PromptChanged(String),
    SendPrompt,
    ResponseReceived(Result<String, String>),
    LoadHistory,
    HistoryLoaded(Result<Vec<ChatMessage>, String>),
}

pub struct ChatApp {
    config: AppConfig,
    prompt_input: String,
    chat_history: Vec<ChatMessage>,
    is_loading: bool,
    error_message: Option<String>,
}

impl ChatApp {
    fn create(config: AppConfig) -> Self {
        Self {
            config,
            prompt_input: String::new(),
            chat_history: Vec::new(),
            is_loading: false,
            error_message: None,
        }
    }

    fn send_request(prompt: String, backend_url: String, timeout: u64) -> Command<Message> {
        Command::perform(
            async move {
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(timeout))
                    .build()
                    .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

                let request_body = serde_json::json!({
                    "prompt": prompt
                });

                let response = client
                    .post(&backend_url)
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| {
                        format!(
                            "Network error: {}. Is the backend server running at {}?",
                            e,
                            backend_url
                        )
                    })?;

                if !response.status().is_success() {
                    return Err(format!(
                        "Server error: {} - {}",
                        response.status(),
                        response.text().await.unwrap_or_default()
                    ));
                }

                let json: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                // Try to extract response from common JSON structures
                let ai_response = json
                    .get("response")
                    .or_else(|| json.get("text"))
                    .or_else(|| json.get("content"))
                    .or_else(|| json.get("message"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        format!(
                            "Unexpected response format. Expected 'response', 'text', 'content', or 'message' field. Got: {}",
                            json
                        )
                    })?;

                Ok(ai_response.to_string())
            },
            Message::ResponseReceived,
        )
    }

    fn log_error(&self, error: &str) {
        error!("{}", error);
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/error.log")
        {
            let log_entry = format!(
                "[{}] ERROR: {}\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                error
            );
            let _ = file.write_all(log_entry.as_bytes());
        }
    }

    fn save_history(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.chat_history) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open("chat_history.json")
            {
                let _ = file.write_all(json.as_bytes());
                info!("Chat history saved");
            }
        }
    }

    fn load_history() -> Command<Message> {
        Command::perform(
            async move {
                match std::fs::read_to_string("chat_history.json") {
                    Ok(content) => {
                        match serde_json::from_str::<Vec<ChatMessage>>(&content) {
                            Ok(history) => Ok(history),
                            Err(e) => Err(format!("Failed to parse history: {}", e)),
                        }
                    }
                    Err(_) => Ok(Vec::new()), // File doesn't exist, return empty history
                }
            },
            Message::HistoryLoaded,
        )
    }
}

impl Application for ChatApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = AppConfig::load().unwrap_or_default();
        let app = Self::create(config);
        (app, Self::load_history())
    }

    fn title(&self) -> String {
        self.config.app.window_title.clone()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PromptChanged(value) => {
                self.prompt_input = value;
                self.error_message = None;
                Command::none()
            }
            Message::SendPrompt => {
                if self.is_loading {
                    return Command::none();
                }

                let prompt = self.prompt_input.trim().to_string();
                if prompt.is_empty() {
                    return Command::none();
                }

                // Add user message to history
                let user_message = ChatMessage::new("user".to_string(), prompt.clone());
                self.chat_history.push(user_message);

                // Limit chat history size
                if self.chat_history.len() > self.config.ui.max_chat_history {
                    self.chat_history.remove(0);
                }

                // Clear input and set loading state
                self.prompt_input.clear();
                self.is_loading = true;
                self.error_message = None;

                // Save history
                self.save_history();

                // Send request
                info!("Sending prompt to backend: {}", prompt);
                Self::send_request(
                    prompt,
                    self.config.backend.url.clone(),
                    self.config.backend.timeout_seconds,
                )
            }
            Message::ResponseReceived(result) => {
                self.is_loading = false;

                match result {
                    Ok(response) => {
                        info!("Received response from backend");
                        let ai_message = ChatMessage::new("assistant".to_string(), response);
                        self.chat_history.push(ai_message);

                        // Limit chat history size
                        if self.chat_history.len() > self.config.ui.max_chat_history {
                            self.chat_history.remove(0);
                        }

                        self.save_history();
                        self.error_message = None;
                    }
                    Err(e) => {
                        let error_msg = format!("Error: {}", e);
                        error!("{}", error_msg);
                        self.log_error(&error_msg);
                        self.error_message = Some(error_msg);
                    }
                }

                Command::none()
            }
            Message::LoadHistory => Self::load_history(),
            Message::HistoryLoaded(result) => {
                match result {
                    Ok(history) => {
                        self.chat_history = history;
                        info!("Loaded {} messages from history", self.chat_history.len());
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to load history: {}", e);
                        error!("{}", error_msg);
                        self.log_error(&error_msg);
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let chat_display: Element<_> = if self.chat_history.is_empty() {
            container(
                text("No messages yet. Start a conversation!")
                    .size(self.config.ui.font_size)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
        } else {
            let mut chat_column = Column::new()
                .spacing(10)
                .padding(10)
                .width(Length::Fill);

            for message in &self.chat_history {
                let role_text = match message.role.as_str() {
                    "user" => text("You:").size(self.config.ui.font_size).style(iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.5, 0.9))),
                    _ => text("AI:").size(self.config.ui.font_size).style(iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.8, 0.4))),
                };

                let content_text = text(&message.content)
                    .size(self.config.ui.font_size)
                    .width(Length::Fill);

                let timestamp_text = text(&message.timestamp)
                    .size(self.config.ui.font_size - 4)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5)));

                let message_container = container(
                    column![
                        row![role_text, timestamp_text].spacing(5).align_items(Alignment::Start),
                        content_text
                    ]
                    .spacing(5)
                    .width(Length::Fill),
                )
                .padding(10)
                .style(iced::theme::Container::Box)
                .width(Length::Fill);

                chat_column = chat_column.push(message_container);
            }

            scrollable(chat_column)
                .id(iced::widget::scrollable::Id::unique())
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let input_row = row![
            text_input("Enter your prompt...", &self.prompt_input)
                .id(iced::widget::text_input::Id::unique())
                .on_input(Message::PromptChanged)
                .on_submit(Message::SendPrompt)
                .size(self.config.ui.font_size)
                .width(Length::Fill)
                .padding(10),
            button(if self.is_loading { "Sending..." } else { "Send" })
                .on_press(Message::SendPrompt)
                .padding(10)
                .width(Length::Shrink)
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill);

        let error_display = if let Some(error) = &self.error_message {
            container(
                text(error)
                    .size(self.config.ui.font_size - 2)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.2, 0.2))),
            )
            .padding(5)
            .style(iced::theme::Container::Box)
            .width(Length::Fill)
            .into()
        } else {
            Element::from(container(text("")).width(Length::Fill).height(Length::Shrink))
        };

        let content = column![
            text("AI Chat Interface")
                .size(self.config.ui.font_size + 4)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.2, 0.2))),
            error_display,
            chat_display,
            input_row
        ]
        .spacing(10)
        .padding(15)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

