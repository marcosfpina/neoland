mod components;
mod model;

use components::{
    composer::Composer,
    conversation::Conversation,
    reasoning::ReasoningPanel,
    sessions::SessionsPanel,
    topbar::Topbar,
};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use model::{seed_messages, Message};
use wasm_bindgen_futures::spawn_local;

/// Root application shell — state management + layout composition.
#[component]
fn App() -> impl IntoView {
    let (active, set_active) = signal(0usize);
    let (messages, set_messages) = signal(seed_messages());
    let (input, set_input) = signal(String::new());
    let (streaming, set_streaming) = signal(false);
    let (stream_text, set_stream_text) = signal(String::new());
    let (sessions_open, set_sessions_open) = signal(false);
    let (reasoning_open, set_reasoning_open) = signal(false);

    let new_chat = Callback::new(move |_| {
        set_messages.set(Vec::new());
        set_input.set(String::new());
        set_active.set(0);
        set_streaming.set(false);
    });

    let submit = Callback::new(move |_| {
        let prompt = input.get_untracked().trim().to_string();
        if prompt.is_empty() || streaming.get_untracked() {
            return;
        }
        set_messages.update(|items| {
            items.push(Message {
                role: "user",
                body: prompt,
                code: None,
            });
        });
        set_input.set(String::new());
        set_streaming.set(true);
        set_stream_text.set(String::new());

        spawn_local(async move {
            let words = [
                "Vou",
                " comparar",
                " o",
                " perfil",
                " atual,",
                " validar",
                " a",
                " ordem",
                " dos",
                " resultados",
                " e",
                " propor",
                " o",
                " menor",
                " patch",
                " seguro.",
            ];
            for word in words {
                if !streaming.get_untracked() {
                    return;
                }
                TimeoutFuture::new(75).await;
                set_stream_text.update(|text| text.push_str(word));
            }
            TimeoutFuture::new(350).await;

            let final_text = stream_text.get_untracked();
            set_messages.update(|items| {
                items.push(Message {
                    role: "agent",
                    body: final_text,
                    code: None,
                });
            });
            set_streaming.set(false);
        });
    });

    let cancel = Callback::new(move |_| set_streaming.set(false));

    view! {
        <div class="app-shell">
            <div class="scanlines" aria-hidden="true"></div>
            <Topbar
                on_sessions=Callback::new(move |_| set_sessions_open.update(|v| *v = !*v))
                on_reasoning=Callback::new(move |_| set_reasoning_open.update(|v| *v = !*v))
            />
            <div class="workspace">
                <SessionsPanel
                    active=active
                    set_active=set_active
                    open=sessions_open
                    on_new=new_chat
                />
                <Conversation messages=messages streaming=streaming stream_text=stream_text />
                <ReasoningPanel open=reasoning_open />
            </div>
            <Composer
                input=input
                set_input=set_input
                streaming=streaming
                submit=submit
                cancel=cancel
            />
            <div class="system-line">
                <span>"NEOLAND://CORE"</span>
                <span>"WASM ONLINE"</span>
                <span>"MEM 24.8MB"</span>
                <span>"BUILD 2026.07"</span>
            </div>
        </div>
    }
}

fn main() {
    mount_to_body(App);
}
