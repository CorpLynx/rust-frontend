use crate::config::AppConfig;
use iced::{
    alignment, executor,
    widget::{
        button, column, container, row, scrollable, text, text_input, Column,
    },
    Alignment, Application, Command, Element, Length, Subscription, Theme,
    Color, Background, Border,
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
    HistoryLoaded(Result<Vec<ChatMessage>, String>),
    ClearChat,
    ToggleDarkMode,
    CopyMessage(usize),
}

pub struct ChatApp {
    config: AppConfig,
    prompt_input: String,
    chat_history: Vec<ChatMessage>,
    is_loading: bool,
    error_message: Option<String>,
    dark_mode: bool,
    scroll_id: scrollable::Id,
}

impl ChatApp {
    fn create(config: AppConfig) -> Self {
        Self {
            config,
            prompt_input: String::new(),
            chat_history: Vec::new(),
            is_loading: false,
            error_message: None,
            dark_mode: false,
            scroll_id: scrollable::Id::unique(),
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
            |result: Result<String, String>| Message::ResponseReceived(result),
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
            |result: Result<Vec<ChatMessage>, String>| Message::HistoryLoaded(result),
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
                Command::batch(vec![
                    Self::send_request(
                        prompt,
                        self.config.backend.url.clone(),
                        self.config.backend.timeout_seconds,
                    ),
                    scrollable::snap_to(self.scroll_id.clone(), scrollable::RelativeOffset::END),
                ])
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
                        
                        // Auto-scroll to bottom after receiving response
                        return scrollable::snap_to(self.scroll_id.clone(), scrollable::RelativeOffset::END);
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
            Message::ClearChat => {
                self.chat_history.clear();
                self.save_history();
                self.error_message = None;
                info!("Chat history cleared");
                Command::none()
            }
            Message::ToggleDarkMode => {
                self.dark_mode = !self.dark_mode;
                info!("Dark mode: {}", self.dark_mode);
                Command::none()
            }
            Message::CopyMessage(index) => {
                if let Some(message) = self.chat_history.get(index) {
                    // Use arboard for clipboard access
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&message.content);
                        info!("Copied message to clipboard");
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Lo-fi hacker aesthetic colors
        let text_color = if self.dark_mode {
            Color::from_rgb(0.0, 1.0, 0.6) // Neon green
        } else {
            Color::from_rgb(0.1, 0.1, 0.1)
        };

        let muted_color = if self.dark_mode {
            Color::from_rgb(0.0, 0.7, 0.5) // Dimmer green
        } else {
            Color::from_rgb(0.5, 0.5, 0.5)
        };

        let user_color = if self.dark_mode {
            Color::from_rgb(1.0, 0.3, 0.6) // Hot pink
        } else {
            Color::from_rgb(0.6, 0.2, 0.8)
        };

        let ai_color = if self.dark_mode {
            Color::from_rgb(0.0, 1.0, 0.6) // Neon green
        } else {
            Color::from_rgb(0.2, 0.8, 0.4)
        };

        let chat_display: Element<_> = if self.chat_history.is_empty() {
            container(
                text(if self.dark_mode { 
                    "> SYSTEM READY. AWAITING INPUT..." 
                } else { 
                    "No messages yet. Start a conversation!" 
                })
                    .size(self.config.ui.font_size)
                    .style(iced::theme::Text::Color(muted_color)),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
        } else {
            let mut chat_column = Column::new()
                .spacing(16)
                .padding(20)
                .width(Length::Fill)
                .align_items(Alignment::Center);

            for (index, message) in self.chat_history.iter().enumerate() {
                let (role_prefix, role_color) = match message.role.as_str() {
                    "user" => (if self.dark_mode { "USER>" } else { "You:" }, user_color),
                    _ => (if self.dark_mode { "AI>" } else { "AI:" }, ai_color),
                };

                let role_text = text(role_prefix)
                    .size(self.config.ui.font_size - 2)
                    .style(iced::theme::Text::Color(role_color));

                let content_text = text(&message.content)
                    .size(self.config.ui.font_size)
                    .style(iced::theme::Text::Color(text_color))
                    .width(Length::Fill);

                let timestamp_text = text(if self.dark_mode {
                    format!("[{}]", &message.timestamp)
                } else {
                    message.timestamp.clone()
                })
                    .size(self.config.ui.font_size - 6)
                    .style(iced::theme::Text::Color(muted_color));

                let copy_button = button(
                    text(if self.dark_mode { "[COPY]" } else { "📋" })
                        .size(self.config.ui.font_size - 6)
                )
                .on_press(Message::CopyMessage(index))
                .padding(4)
                .style(iced::theme::Button::Secondary);

                // ChatGPT-style: centered column with max width
                let message_content = container(
                    column![
                        row![
                            role_text,
                            timestamp_text,
                            copy_button
                        ]
                        .spacing(8)
                        .align_items(Alignment::Center),
                        content_text
                    ]
                    .spacing(8)
                    .width(Length::Fill),
                )
                .padding(16)
                .style(if self.dark_mode {
                    iced::theme::Container::Custom(Box::new(HackerMessageStyle))
                } else {
                    iced::theme::Container::Box
                })
                .width(Length::Fixed(700.0)) // Max width like ChatGPT
                .center_x();

                chat_column = chat_column.push(message_content);
            }

            scrollable(chat_column)
                .id(self.scroll_id.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let input_placeholder = if self.dark_mode {
            "> ENTER COMMAND..."
        } else {
            "Enter your prompt..."
        };

        let send_text = if self.dark_mode {
            if self.is_loading { "↻" } else { "↑" }
        } else {
            if self.is_loading { "..." } else { "↑" }
        };

        // ChatGPT-style floating input box
        let input_box = container(
            row![
                text_input(input_placeholder, &self.prompt_input)
                    .id(iced::widget::text_input::Id::unique())
                    .on_input(Message::PromptChanged)
                    .on_submit(Message::SendPrompt)
                    .size(self.config.ui.font_size)
                    .width(Length::Fill)
                    .padding(14)
                    .style(if self.dark_mode {
                        iced::theme::TextInput::Custom(Box::new(HackerInputStyle))
                    } else {
                        iced::theme::TextInput::Default
                    }),
                button(text(send_text).size(self.config.ui.font_size + 4))
                    .on_press(Message::SendPrompt)
                    .padding(12)
                    .width(Length::Fixed(50.0))
                    .style(if self.dark_mode {
                        iced::theme::Button::Primary
                    } else {
                        iced::theme::Button::Primary
                    })
            ]
            .spacing(8)
            .align_items(Alignment::Center)
            .width(Length::Fixed(700.0)),
        )
        .padding(12)
        .style(if self.dark_mode {
            iced::theme::Container::Custom(Box::new(HackerInputContainerStyle))
        } else {
            iced::theme::Container::Box
        })
        .center_x()
        .width(Length::Fill);

        let input_row = container(input_box)
            .padding(20)
            .width(Length::Fill);

        let error_color = if self.dark_mode {
            Color::from_rgb(1.0, 0.2, 0.4) // Hot pink/red
        } else {
            Color::from_rgb(0.9, 0.2, 0.2)
        };

        let error_display = if let Some(error) = &self.error_message {
            container(
                text(if self.dark_mode {
                    format!("⚠ ERROR: {}", error)
                } else {
                    error.clone()
                })
                    .size(self.config.ui.font_size - 2)
                    .style(iced::theme::Text::Color(error_color)),
            )
            .padding(8)
            .style(if self.dark_mode {
                iced::theme::Container::Custom(Box::new(HackerErrorStyle))
            } else {
                iced::theme::Container::Box
            })
            .width(Length::Fill)
            .into()
        } else {
            Element::from(container(text("")).width(Length::Fill).height(Length::Shrink))
        };

        let header_text_color = if self.dark_mode {
            Color::from_rgb(0.0, 1.0, 0.6) // Neon green
        } else {
            Color::from_rgb(0.2, 0.2, 0.2)
        };

        let header_title = if self.dark_mode {
            "[ NEURAL INTERFACE v0.2.0 ]"
        } else {
            "AI Chat Interface"
        };

        let dark_mode_button = button(
            text(if self.dark_mode { "[LIGHT]" } else { "🌙" })
                .size(self.config.ui.font_size - 4)
        )
        .on_press(Message::ToggleDarkMode)
        .padding(6)
        .style(iced::theme::Button::Secondary);

        let clear_button = button(
            text(if self.dark_mode { "[CLEAR]" } else { "Clear" })
                .size(self.config.ui.font_size - 4)
        )
        .on_press(Message::ClearChat)
        .padding(6)
        .style(iced::theme::Button::Secondary);

        // Minimal header like ChatGPT
        let header = container(
            row![
                text(header_title)
                    .size(self.config.ui.font_size + 2)
                    .style(iced::theme::Text::Color(header_text_color)),
                dark_mode_button,
                clear_button
            ]
            .spacing(12)
            .align_items(Alignment::Center)
        )
        .padding(12)
        .width(Length::Fill)
        .center_x();

        // ChatGPT-style layout: header at top, chat in middle, input floating at bottom
        let content = column![
            header,
            error_display,
            chat_display,
            input_row
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(if self.dark_mode {
                iced::theme::Container::Custom(Box::new(HackerContainerStyle))
            } else {
                iced::theme::Container::default()
            })
            .into()
    }

    fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::custom(
                "hacker".to_string(),
                iced::theme::Palette {
                    background: Color::from_rgb(0.05, 0.05, 0.08),
                    text: Color::from_rgb(0.0, 1.0, 0.6),
                    primary: Color::from_rgb(0.0, 0.8, 1.0),
                    success: Color::from_rgb(0.0, 1.0, 0.6),
                    danger: Color::from_rgb(1.0, 0.2, 0.4),
                },
            )
        } else {
            Theme::Light
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

// Custom styles for hacker aesthetic
struct HackerContainerStyle;

impl iced::widget::container::StyleSheet for HackerContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.05, 0.05, 0.08))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.0),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

struct HackerInputContainerStyle;

impl iced::widget::container::StyleSheet for HackerInputContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.8, 1.0, 0.08))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.4),
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
}

struct HackerMessageStyle;

impl iced::widget::container::StyleSheet for HackerMessageStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.8, 1.0, 0.05))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.3),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

struct HackerErrorStyle;

impl iced::widget::container::StyleSheet for HackerErrorStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(1.0, 0.2, 0.4, 0.1))),
            border: Border {
                color: Color::from_rgb(1.0, 0.2, 0.4),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

struct HackerInputStyle;

impl iced::widget::text_input::StyleSheet for HackerInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Background::Color(Color::from_rgba(0.08, 0.08, 0.12, 0.0)),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.0),
                width: 0.0,
                radius: 8.0.into(),
            },
            icon_color: Color::from_rgb(0.0, 1.0, 0.6),
        }
    }

    fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Background::Color(Color::from_rgba(0.08, 0.08, 0.12, 0.0)),
            border: Border {
                color: Color::from_rgba(0.0, 1.0, 0.6, 0.0),
                width: 0.0,
                radius: 8.0.into(),
            },
            icon_color: Color::from_rgb(0.0, 1.0, 0.6),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.0, 0.7, 0.5, 0.5)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.0, 1.0, 0.6)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.0, 0.8, 1.0, 0.3)
    }

    fn disabled(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        self.active(_style)
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.0, 0.7, 0.5, 0.3)
    }
}

