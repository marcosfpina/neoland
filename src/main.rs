mod hyprland_ops;

use hyprland_ops::HyprlandOps;
use adw::prelude::*;
use gtk4::{
    gdk, glib::{self, clone},
    prelude::*,
    Box as GtkBox, Button, CssProvider, Entry, Label,
    ScrolledWindow, TextBuffer, TextView, Orientation, Switch, StyleContext,
};
use libadwaita as adw;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
// use tonic::transport::Channel;

// Include generated gRPC code
pub mod llamachat {
    tonic::include_proto!("llamachat");
}
use llamachat::llama_service_client::LlamaServiceClient;
use llamachat::ChatRequest;

const APP_ID: &str = "com.exemplo.llamachat";
const SERVER_URL: &str = "http://[::1]:50051";

#[tokio::main]
async fn main() {
    glib::set_application_name("LlamaChat PoC");
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        load_css();
        HyprlandOps::init_rules(APP_ID);
    });

    app.connect_activate(build_ui);
    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(
        "\n        /* Global Reset & Base */\n        window, box, textview {\n            font-family: 'Inter', 'Segoe UI', sans-serif;\n        }\n\n        /* Glassmorphism Background */\n        window.background {\n            background: linear-gradient(145deg, #1a1b26 0%, #24283b 100%);\n            color: #c0caf5;\n        }\n\n        /* Header Bar Glass */\n        headerbar {\n            background-color: rgba(26, 27, 38, 0.85);\n            border-bottom: 1px solid rgba(255, 255, 255, 0.08);\n            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);\n        }\n\n        headerbar label {\n            font-weight: bold;\n            color: #7aa2f7;\n        }\n\n        /* Sidebar Glass */\n        .sidebar {\n            background-color: rgba(16, 16, 20, 0.4);\n            border-right: 1px solid rgba(255, 255, 255, 0.05);\n        }\n\n        row {\n            padding: 10px;\n            margin: 4px 8px;\n            border-radius: 10px;\n            transition: all 0.2s ease;\n            color: #a9b1d6;\n        }\n\n        row:hover {\n            background-color: rgba(255, 255, 255, 0.05);\n            color: white;\n        }\n\n        row:selected {\n            background-color: rgba(122, 162, 247, 0.2);\n            border: 1px solid rgba(122, 162, 247, 0.3);\n            color: #7aa2f7;\n        }\n\n        .chat-area {\n            background-color: rgba(30, 30, 40, 0.3);\n        }\n\n        textview {\n            background-color: transparent;\n            color: #c0caf5;\n        }\n\n        textview text {\n            background-color: transparent;\n        }\n\n        entry {\n            background-color: rgba(0, 0, 0, 0.2);\n            border: 1px solid rgba(255, 255, 255, 0.1);\n            border-radius: 15px;\n            color: white;\n            padding: 0 10px;\n            box-shadow: inset 0 2px 4px rgba(0,0,0,0.2);\n            transition: all 0.2s;\n        }\n\n        entry:focus {\n            background-color: rgba(0, 0, 0, 0.3);\n            border-color: #7aa2f7;\n            box-shadow: 0 0 0 2px rgba(122, 162, 247, 0.2);\n        }\n\n        button.suggested-action {\n            background: linear-gradient(135deg, #7aa2f7 0%, #3d59a1 100%);\n            color: white;\n            border-radius: 15px;\n            border: none;\n            box-shadow: 0 4px 10px rgba(61, 89, 161, 0.4);\n            font-weight: bold;\n            padding: 8px 16px;\n        }\n\n        button.hypr-btn {\n            background-color: rgba(255, 255, 255, 0.1);\n            color: #c0caf5;\n            border-radius: 10px;\n            padding: 4px 8px;\n        }\n        \n        "
    );

    if let Some(display) = gdk::Display::default() {
        StyleContext::add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("LlamaChat Enterprise - gRPC Client")
        .default_width(1200)
        .default_height(800)
        .build();

    let _use_local = Arc::new(Mutex::new(true)); // Always true for this gRPC PoC

    let main_box = GtkBox::new(Orientation::Vertical, 0);
    
    // Header
    let header = adw::HeaderBar::new();
    let toggle_box = GtkBox::new(Orientation::Horizontal, 10);
    toggle_box.set_margin_end(10);
    
    let scratch_btn = Button::with_label("󰖯 Scratchpad");
    scratch_btn.add_css_class("hypr-btn");
    scratch_btn.connect_clicked(|_| {
        HyprlandOps::toggle_scratchpad();
    });

    let switch_label = Label::new(Some("Microservice Mode (gRPC)"));
    let switch = Switch::new();
    switch.set_active(true);
    switch.set_sensitive(false);
    
    toggle_box.append(&scratch_btn);
    toggle_box.append(&switch_label);
    toggle_box.append(&switch);
    header.pack_end(&toggle_box);
    
    main_box.append(&header);

    let flap = adw::Flap::builder()
        .fold_policy(adw::FlapFoldPolicy::Auto)
        .vexpand(true)
        .build();
    main_box.append(&flap);

    // SIDEBAR
    let sidebar_box = GtkBox::new(Orientation::Vertical, 0);
    sidebar_box.set_width_request(280); 
    sidebar_box.add_css_class("sidebar"); 
    
    let status_label = Label::new(Some("CONECTADO AO SERVIDOR"));
    status_label.set_margin_top(20);
    status_label.add_css_class("caption-heading");
    status_label.set_opacity(0.7);
    sidebar_box.append(&status_label);

    // CHAT AREA
    let chat_box = GtkBox::new(Orientation::Vertical, 0);
    chat_box.add_css_class("chat-area");

    let text_buffer = TextBuffer::new(None);
    let text_view = TextView::builder()
        .buffer(&text_buffer)
        .editable(false)
        .wrap_mode(gtk4::WrapMode::WordChar)
        .left_margin(20)
        .right_margin(20)
        .top_margin(20)
        .bottom_margin(20)
        .build();

    let scrolled_messages = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&text_view)
        .build();

    let input_container = GtkBox::new(Orientation::Vertical, 0);
    input_container.set_margin_start(20);
    input_container.set_margin_end(20);
    input_container.set_margin_bottom(20);

    let input_box = GtkBox::new(Orientation::Horizontal, 10);
    
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Digite sua mensagem..."));
    entry.set_hexpand(true);

    let send_btn = Button::with_label("Enviar");
    send_btn.add_css_class("suggested-action");
    
    // Send Logic
    send_btn.connect_clicked(clone!(@weak entry, @weak text_buffer => move |_| {
        let message = entry.text().to_string();
        if !message.is_empty() {
            append_message(&text_buffer, "user", &message);
            entry.set_text("");
            
            send_message_grpc(message, text_buffer.clone());
        }
    }));

    entry.connect_activate(clone!(@weak send_btn => move |_| {
        send_btn.emit_clicked();
    }));

    input_box.append(&entry);
    input_box.append(&send_btn);
    
    input_container.append(&input_box);

    chat_box.append(&scrolled_messages);
    chat_box.append(&input_container);

    flap.set_content(Some(&chat_box));
    flap.set_flap(Some(&sidebar_box));

    window.set_content(Some(&main_box));
    window.present();
}

fn append_message(buffer: &TextBuffer, role: &str, content: &str) {
    let mut end = buffer.end_iter();
    let prefix = match role {
        "user" => "👤 VOCÊ",
        "system" => "⚙️ SYSTEM",
        _ => "🤖 AI"
    };
    let formatted = format!("\n{} ────────────────────────\n{}\n", prefix, content);
    buffer.insert(&mut end, &formatted);
}

fn try_execute_command(buffer: &TextBuffer, command_str: &str) {
    let content = command_str.trim_matches(|c| c == '[' || c == ']');
    if let Some(stripped) = content.strip_prefix("CMD:") {
        let parts: Vec<&str> = stripped.splitn(2, ':').collect();
        if !parts.is_empty() {
            let action = parts[0];
            let args = parts.get(1).unwrap_or(&"");

            match action {
                "move_ws" => {
                    if let Ok(ws_id) = args.parse::<i32>() {
                        append_message(buffer, "system", &format!("Movendo para Workspace {}", ws_id));
                        HyprlandOps::move_to_workspace(ws_id);
                    }
                },
                "scratchpad" => {
                    append_message(buffer, "system", "Alternando Scratchpad");
                    HyprlandOps::toggle_scratchpad();
                },
                _ => {}
            }
        }
    }
}

fn send_message_grpc(message: String, buffer: TextBuffer) {
    // We use glib::MainContext::spawn_local to handle async within GTK
    glib::MainContext::default().spawn_local(async move {
        let mut client = match LlamaServiceClient::connect(SERVER_URL).await {
            Ok(c) => c,
            Err(e) => {
                append_message(&buffer, "system", &format!("ERRO DE CONEXÃO: {}", e));
                return;
            }
        };

        let request = ChatRequest {
            prompt: message,
            model_id: "qwen-1.8b".to_string(),
            use_local: true,
        };

        match client.chat_stream(request).await {
            Ok(response) => {
                let mut stream = response.into_inner();
                let mut response_buffer = String::new();
                
                while let Ok(Some(chunk)) = stream.message().await {
                    response_buffer.push_str(&chunk.content);
                    
                    // Command detection
                    if chunk.is_command {
                        // In a real app, we'd handle the full command after stream ends or with markers
                        if let Some(start) = response_buffer.find("[[CMD:") {
                            if let Some(end) = response_buffer[start..].find("]]") {
                                let cmd = &response_buffer[start..start + end + 2];
                                try_execute_command(&buffer, cmd);
                            }
                        }
                    }

                    let mut end = buffer.end_iter();
                    buffer.insert(&mut end, &chunk.content);
                }
            }
            Err(e) => {
                append_message(&buffer, "system", &format!("ERRO RPC: {}", e));
            }
        }
    });
}
