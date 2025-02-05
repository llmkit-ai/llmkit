use serde::Serialize;
use tera::{Context, Tera};

use crate::{
    common::types::models::LlmModel, 
    db::types::prompt::PromptWithModel
};

use super::message::Message;


#[derive(Serialize)]
pub struct LlmProps {
    pub model: LlmModel,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub messages: Vec<Message>,
    pub prompt_id: i64,
    pub model_id: i64
}

impl LlmProps {
    pub fn new(prompt: PromptWithModel, context: serde_json::Value) -> Self {
        let mut tera = Tera::default();
        tera.add_raw_template("prompt", &prompt.prompt).unwrap();

        let mut tera_ctx = Context::new();
        if let serde_json::Value::Object(context) = context {
            for (k, v) in context {
                tera_ctx.insert(k, &v);
            }
        }
        let rendered_prompt = tera.render("prompt", &tera_ctx).unwrap();
        let messages = parse_prompt(&rendered_prompt);

        let model_name: LlmModel = prompt.model_name.into();

        LlmProps {
            model: model_name,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            messages,
            prompt_id: prompt.id,
            model_id: prompt.model_id,
        }
    }
}


pub fn parse_prompt(input: &str) -> Vec<Message> {
    let mut messages = Vec::new();
    let mut current_role: Option<String> = None;
    let mut current_content = Vec::new();

    for line in input.lines() {
        if let Some(role) = parse_role_line(line) {
            // Process previous role's content
            if let Some(prev_role) = current_role.take() {
                let content = current_content.join("\n");
                if !content.is_empty() {
                    let message = match prev_role.as_str() {
                        "system" => Message::System { content },
                        "user" => Message::User { content },
                        "assistant" => Message::Assistant { content },
                        _ => unreachable!(),
                    };
                    messages.push(message);
                }
                current_content.clear();
            }
            current_role = Some(role);
        } else if current_role.is_some() {
            current_content.push(line);
        }
    }

    // Process remaining content after last role
    if let Some(role) = current_role.take() {
        let content = current_content.join("\n");
        if !content.is_empty() {
            let message = match role.as_str() {
                "system" => Message::System { content },
                "user" => Message::User { content },
                "assistant" => Message::Assistant { content },
                _ => unreachable!(),
            };
            messages.push(message);
        }
    }

    messages
}

fn parse_role_line(line: &str) -> Option<String> {
    const PREFIX: &str = "<!-- role:";
    const SUFFIX: &str = "-->";

    if line.starts_with(PREFIX) && line.ends_with(SUFFIX) {
        let role = line[PREFIX.len()..line.len() - SUFFIX.len()]
            .trim()
            .to_string();
        match role.as_str() {
            "system" | "user" | "assistant" => Some(role),
            _ => None,
        }
    } else {
        None
    }
}
