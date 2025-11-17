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
    Tick,
}

pub struct ChatApp {
    config: AppConfig,
    prompt_input: String,
    chat_history: Vec<ChatMessage>,
    is_loading: bool,
    error_message: Option<String>,
    scroll_id: scrollable::Id,
    animation_offset: usize,
}

impl ChatApp {
    fn create(config: AppConfig) -> Self {
        Self {
            config,
            prompt_input: String::new(),
            chat_history: Vec::new(),
            is_loading: false,
            error_message: None,
            scroll_id: scrollable::Id::unique(),
            animation_offset: 0,
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
            Message::Tick => {
                // Animate the empty state text
                if self.chat_history.is_empty() {
                    self.animation_offset = (self.animation_offset + 1) % 50;
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Dark mode only - hacker aesthetic colors
        let text_color = Color::from_rgb(0.0, 1.0, 0.6); // Neon green
        let muted_color = Color::from_rgb(0.0, 0.7, 0.5); // Dimmer green

        let chat_display: Element<_> = if self.chat_history.is_empty() {
            // Animated text - wave effect
            let base_text = "> system ready. awaiting input...";
            let animated_text: String = base_text
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if c.is_alphabetic() {
                        // Create wave effect - capitalize letters in a moving window
                        let wave_position = self.animation_offset % base_text.len();
                        let distance = if i >= wave_position {
                            i - wave_position
                        } else {
                            base_text.len() + i - wave_position
                        };
                        
                        // Capitalize if within the wave (3 characters wide)
                        if distance < 3 {
                            c.to_uppercase().to_string()
                        } else {
                            c.to_string()
                        }
                    } else {
                        c.to_string()
                    }
                })
                .collect();

            container(
                text(animated_text)
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
                .spacing(12)
                .padding(20)
                .width(Length::Fill);

            for message in &self.chat_history {
                let is_user = message.role.as_str() == "user";

                // Ollama-style: user messages on right, AI on left
                // Shrink-wrap to content with max width
                let message_bubble = container(
                    text(&message.content)
                        .size(self.config.ui.font_size)
                        .style(iced::theme::Text::Color(text_color))
                )
                .padding(16)
                .max_width(600)
                .style(if is_user {
                    iced::theme::Container::Custom(Box::new(UserMessageStyle))
                } else {
                    iced::theme::Container::Custom(Box::new(AIMessageStyle))
                });

                // Align user messages to the right, AI to the left
                let message_row = if is_user {
                    container(message_bubble)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Right)
                } else {
                    container(message_bubble)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Left)
                };

                chat_column = chat_column.push(message_row);
            }

            scrollable(chat_column)
                .id(self.scroll_id.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let send_text = if self.is_loading { "↻" } else { "↑" };

        // Ollama-style floating input box
        let input_box = container(
            row![
                text_input("Ask anything...", &self.prompt_input)
                    .id(iced::widget::text_input::Id::unique())
                    .on_input(Message::PromptChanged)
                    .on_submit(Message::SendPrompt)
                    .size(self.config.ui.font_size)
                    .width(Length::Fill)
                    .padding(16)
                    .style(iced::theme::TextInput::Custom(Box::new(HackerInputStyle))),
                button(text(send_text).size(self.config.ui.font_size + 6))
                    .on_press(Message::SendPrompt)
                    .padding(14)
                    .width(Length::Fixed(50.0))
                    .style(iced::theme::Button::Primary)
            ]
            .spacing(10)
            .align_items(Alignment::Center)
            .width(Length::Fixed(750.0)),
        )
        .padding(14)
        .style(iced::theme::Container::Custom(Box::new(HackerInputContainerStyle)))
        .center_x()
        .width(Length::Fill);

        let input_row = container(input_box)
            .padding(20)
            .width(Length::Fill);

        let error_color = Color::from_rgb(1.0, 0.2, 0.4); // Hot pink/red

        let error_display = if let Some(error) = &self.error_message {
            container(
                text(format!("⚠ ERROR: {}", error))
                    .size(self.config.ui.font_size - 2)
                    .style(iced::theme::Text::Color(error_color)),
            )
            .padding(10)
            .style(iced::theme::Container::Custom(Box::new(HackerErrorStyle)))
            .width(Length::Fixed(750.0))
            .center_x()
            .into()
        } else {
            Element::from(container(text("")).width(Length::Fill).height(Length::Shrink))
        };

        let header_text_color = Color::from_rgb(0.0, 1.0, 0.6); // Neon green

        let clear_button = button(
            text("New Chat")
                .size(self.config.ui.font_size - 2)
        )
        .on_press(Message::ClearChat)
        .padding(8)
        .style(iced::theme::Button::Text);

        // Ollama-style minimal header
        let header = container(
            row![
                text("NEURAL INTERFACE")
                    .size(self.config.ui.font_size)
                    .style(iced::theme::Text::Color(header_text_color)),
                clear_button
            ]
            .spacing(15)
            .align_items(Alignment::Center)
        )
        .padding(15)
        .width(Length::Fill)
        .center_x();

        // Ollama-style layout: minimal header, chat in middle, input floating at bottom
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
            .style(iced::theme::Container::Custom(Box::new(HackerContainerStyle)))
            .into()
    }

    fn theme(&self) -> Theme {
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
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::time;
        use std::time::Duration;
        
        // Only animate when chat is empty
        if self.chat_history.is_empty() {
            time::every(Duration::from_millis(200)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
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
            background: Some(Background::Color(Color::from_rgba(0.08, 0.08, 0.12, 1.0))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.5),
                width: 1.5,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

// User message style - lighter, on the right
struct UserMessageStyle;

impl iced::widget::container::StyleSheet for UserMessageStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.8, 1.0, 0.12))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.3),
                width: 1.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

// AI message style - darker, on the left
struct AIMessageStyle;

impl iced::widget::container::StyleSheet for AIMessageStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.8, 1.0, 0.04))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.15),
                width: 1.0,
                radius: 16.0.into(),
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

