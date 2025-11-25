// Library interface for prometheus-cli
// This exposes modules for integration testing

pub mod app;
pub mod backend;
pub mod commands;
pub mod config;
pub mod conversation;
pub mod error;
pub mod exit_codes;
pub mod input;
pub mod markdown_renderer;
pub mod mode;
pub mod non_interactive;
pub mod ollama_service;
pub mod output;
pub mod streaming;
pub mod terminal;
pub mod update;
pub mod url_validator;