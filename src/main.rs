use adw::prelude::*;
use gtk4::{
    glib::{self, clone},
    prelude::*,
    Box as GtkBox, Button, Entry, Label, ListBox, ListBoxRow,
    ScrolledWindow, TextBuffer, TextView, Orientation,
};
use libadwaita as adw;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

#[derive(Deserialize, Serialize, Clone)]
struct Model {
    id: String,
    object: String,
    #[serde(rename = "created")]
    created_at: i64,
    #[serde(rename = "owned_by")]
    owned_by: String,
}

#[derive(Deserialize, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Deserialize, Serialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponseChunk {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
}

const LLAMA_SERVER: &str = "http://localhost:8080/v1";

fn main() {
    glib::set_application_name("LlamaChat PoC");
    let app = adw::Application::builder()
        .application_id("com.exemplo.llamachat")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("LlamaChat - Native PoC")
        .default_width(1200)
        .default_height(800)
        .build();

    // Use OverlaySplitView for modern sidebar behavior
    let split_view = adw::OverlaySplitView::builder()
        .sidebar_width_fraction(0.25)
        .min_sidebar_width(250.0)
        .build();

    // SIDEBAR: Model List
    let sidebar_box = GtkBox::new(Orientation::Vertical, 0);
    let model_list = ListBox::new();
    // Add CSS class for styling if needed, or margin
    model_list.set_margin_top(10);
    model_list.set_margin_bottom(10);
    model_list.set_margin_start(10);
    model_list.set_margin_end(10);

    let model_label = Label::new(Some("Modelos (carregando...)"));
    model_label.set_margin_top(10);
    
    sidebar_box.append(&model_label);
    
    let scrolled_sidebar = ScrolledWindow::builder()
        .vexpand(true)
        .child(&model_list)
        .build();
    sidebar_box.append(&scrolled_sidebar);

    // Load models
    load_models(&model_list, model_label.clone());

    // CHAT AREA
    let chat_box = GtkBox::new(Orientation::Vertical, 5);
    chat_box.set_margin_start(12);
    chat_box.set_margin_end(12);
    chat_box.set_margin_top(12);
    chat_box.set_margin_bottom(12);

    // Messages Area
    let text_buffer = TextBuffer::new(None);
    let text_view = TextView::builder()
        .buffer(&text_buffer)
        .editable(false)
        .wrap_mode(gtk4::WrapMode::WordChar)
        .left_margin(10)
        .right_margin(10)
        .top_margin(10)
        .bottom_margin(10)
        .build();

    let scrolled_messages = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&text_view)
        .build();

    // Input Area
    let input_box = GtkBox::new(Orientation::Horizontal, 5);
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Digite sua mensagem..."));
    entry.set_hexpand(true);

    let send_btn = Button::with_label("Enviar");
    send_btn.add_css_class("suggested-action");
    
    send_btn.connect_clicked(clone!(@weak entry, @weak text_buffer => move |_| {
        let message = entry.text().to_string();
        if !message.is_empty() {
            append_message(&text_buffer, "user", &message);
            entry.set_text("");
            send_to_llama(message, text_buffer.clone());
        }
    }));

    // Allow pressing Enter to send
    entry.connect_activate(clone!(@weak send_btn => move |_| {
        send_btn.emit_clicked();
    }));

    input_box.append(&entry);
    input_box.append(&send_btn);

    chat_box.append(&scrolled_messages);
    chat_box.append(&input_box);

    // Assemble SplitView
    split_view.set_sidebar(Some(&sidebar_box));
    split_view.set_content(Some(&chat_box));

    window.set_content(Some(&split_view));
    window.present();
}

// Use spawn_blocking for one-shot fetch (cleanest pattern)
fn load_models(list: &ListBox, label: Label) {
    let future = gtk4::gio::spawn_blocking(move || {
        let client = Client::new();
        match client.get(format!("{}/models", LLAMA_SERVER)).send() {
            Ok(resp) => resp.json::<HashMap<String, Vec<Model>>>().ok(),
            Err(_) => None,
        }
    });

    glib::MainContext::default().spawn_local(clone!(@weak list, @weak label => async move {
        match future.await {
            Ok(Some(models_resp)) => {
                // Clear list
                while let Some(child) = list.first_child() {
                    list.remove(&child);
                }
                
                let models = models_resp.get("data").cloned().unwrap_or_default();
                label.set_text(&format!("Modelos ({})", models.len()));
                
                for model in models {
                    let row = ListBoxRow::new();
                    let lbl = Label::builder()
                        .label(&model.id)
                        .halign(gtk4::Align::Start)
                        .margin_start(10)
                        .margin_end(10)
                        .margin_top(5)
                        .margin_bottom(5)
                        .build();
                    row.set_child(Some(&lbl));
                    list.append(&row);
                }
            }
            Ok(None) | Err(_) => {
                label.set_text("Erro ao carregar modelos.");
            }
        }
    }));
}

fn append_message(buffer: &TextBuffer, role: &str, content: &str) {
    let mut end = buffer.end_iter();
    let color = if role == "user" { "**Você:** " } else { "**AI:** " };
    let formatted = format!("{}\n{}\n\n", color, content.replace("```", "\n```\n"));
    buffer.insert(&mut end, &formatted);
    // Scroll to end (basic)
    // Note: In real app, use Mark for scrolling
}

// Use thread + channel for streaming (robust pattern)
fn send_to_llama(message: String, buffer: TextBuffer) {
    let (sender, receiver) = glib::MainContext::channel(glib::Priority::default());

    // Receiver on Main Thread
    receiver.attach(None, clone!(@weak buffer => @default-return glib::ControlFlow::Break, move |content: String| {
        let mut end = buffer.end_iter();
        buffer.insert(&mut end, &content);
        glib::ControlFlow::Continue
    }));

    // Worker Thread
    std::thread::spawn(move || {
        let client = Client::new();
        let request = ChatRequest {
            model: "llama3.1".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content,
            }],
            stream: true,
        };

        if let Ok(mut resp) = client.post(format!("{}/chat/completions", LLAMA_SERVER))
            .json(&request)
            .send() 
        {
            // Read chunk by chunk
            // Note: reqwest::blocking::Response implements Read.
            // But SSE lines might be split across chunks.
            // For a robust implementation, we should use a BufReader and read lines.
            let mut reader = std::io::BufReader::new(resp);
            let mut line = String::new();
            
            while let Ok(len) = reader.read_line(&mut line) {
                if len == 0 { break; } // EOF
                
                if let Some(data) = line.strip_prefix("data: ") {
                    let data = data.trim();
                    if data == "[DONE]" { break; }
                    if let Ok(resp_chunk) = serde_json::from_str::<ChatResponseChunk>(data) {
                        if let Some(content) = resp_chunk.choices.first()
                            .and_then(|c| c.delta.content.as_ref()) 
                        {
                            let _ = sender.send(content.clone());
                        }
                    }
                }
                line.clear();
            }
        }
    });
}