mod api;
mod components;
mod model;

use components::{
    composer::Composer, conversation::Conversation, reasoning::ReasoningPanel,
    sessions::SessionsPanel, topbar::Topbar,
};
use leptos::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use wasm_bindgen_futures::spawn_local;

/// Returns the backend base URL.
///
/// In dev, Trunk proxies `/v1/*` → `localhost:3001`, so the empty string
/// (same origin) hits the backend through the proxy.  In production the
/// Neoland server serves the WASM bundle directly — same origin again.
fn backend_url() -> String {
    // Same-origin: Trunk proxy in dev, Neoland static serve in prod
    String::new()
}

#[component]
fn App() -> impl IntoView {
    let base = backend_url();

    let (sessions, set_sessions) = signal(Vec::new());
    let (active_idx, set_active_idx) = signal(0usize);
    let (messages, set_messages) = signal(Vec::new());
    let (input, set_input) = signal(String::new());
    let (streaming, set_streaming) = signal(false);
    let (stream_text, set_stream_text) = signal(String::new());
    let (sessions_open, set_sessions_open) = signal(false);
    let (reasoning_open, set_reasoning_open) = signal(false);
    let (active_session_id, set_active_session_id) = signal(None::<String>);

    // Monotonically-increasing stream generation counter.
    // Each SSE connection is tagged with a generation; callbacks from old
    // (superseded) connections silently discard their events.
    let stream_gen: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));

    // Load sessions on mount
    {
        let base = base.clone();
        spawn_local(async move {
            match api::list_sessions(&base).await {
                Ok(list) => set_sessions.set(list),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load sessions: {e}").into())
                },
            }
        });
    }

    let new_chat = {
        let base = base.clone();
        Callback::new(move |_| {
            set_messages.set(Vec::new());
            set_input.set(String::new());
            set_active_session_id.set(None);
            set_streaming.set(false);
            // Reload session list
            let base = base.clone();
            spawn_local(async move {
                if let Ok(list) = api::list_sessions(&base).await {
                    set_sessions.set(list);
                }
            });
        })
    };

    let select_session = {
        let base = base.clone();
        Callback::new(move |idx: usize| {
            set_active_idx.set(idx);
            if let Some(session) = sessions.get().get(idx) {
                let sid = session.session_id.clone();
                set_active_session_id.set(Some(sid.clone()));
                let base = base.clone();
                spawn_local(async move {
                    if let Ok(msgs) = api::fetch_messages(&base, &sid).await {
                        set_messages.set(
                            msgs.into_iter()
                                .map(|m| model::Message {
                                    role: if m.role == "user" { "user" } else { "agent" },
                                    body: m.content,
                                    code: None,
                                })
                                .collect(),
                        );
                    }
                });
            }
        })
    };

    let submit = {
        let base = base.clone();
        let stream_gen = stream_gen.clone();
        Callback::new(move |_| {
            let prompt = input.get_untracked().trim().to_string();
            if prompt.is_empty() || streaming.get_untracked() {
                return;
            }
            // Push user message immediately
            set_messages.update(|items| {
                items.push(model::Message { role: "user", body: prompt.clone(), code: None });
            });
            set_input.set(String::new());
            set_streaming.set(true);
            set_stream_text.set(String::new());

            // Bump generation — invalidates any still-firing callbacks from a previous SSE.
            let current_gen = stream_gen.fetch_add(1, Ordering::Relaxed);

            let base = base.clone();
            let sid = active_session_id.get_untracked();
            let stream_gen = stream_gen.clone();
            spawn_local(async move {
                match api::submit_task(&base, &prompt, sid.as_deref()).await {
                    Ok(resp) => {
                        set_active_session_id.set(Some(resp.session_id.clone()));
                        // The EventSource handle is dropped at the end of this block,
                        // but the JS object lives on thanks to the `.forget()`-ed
                        // closure.  The generation guard below ensures only the most
                        // recent stream writes to the UI.
                        let _source = api::connect_sse(&base, &resp.session_id, {
                            let stream_gen = stream_gen.clone();
                            move |ev: api::SseEvent| {
                                // Discard events from a superseded connection.
                                if stream_gen.load(Ordering::Relaxed) != current_gen {
                                    return;
                                }
                                if let Some(content) = ev.content() {
                                    let prefix = if let Some(stage) = ev.stage() {
                                        format!("[{}] ", stage)
                                    } else {
                                        String::new()
                                    };
                                    set_stream_text.update(|t| {
                                        t.push_str(&format!("{}{}\n", prefix, content))
                                    });
                                }
                                if ev.is_terminal() {
                                    set_streaming.set(false);
                                }
                            }
                        });
                    },
                    Err(e) => {
                        set_streaming.set(false);
                        web_sys::console::error_1(&format!("Task failed: {e}").into());
                    },
                }
            });
        })
    };

    let cancel = Callback::new(move |_| {
        set_streaming.set(false);
    });

    view! {
        <div class="app-shell">
            <div class="scanlines" aria-hidden="true"></div>
            <Topbar
                on_sessions=Callback::new(move |_| set_sessions_open.update(|v| *v = !*v))
                on_reasoning=Callback::new(move |_| set_reasoning_open.update(|v| *v = !*v))
            />
            <div class="workspace">
                <SessionsPanel
                    sessions=sessions
                    active=active_idx
                    set_active=set_active_idx
                    on_select=select_session
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
                <span>"BUILD 2026.07"</span>
            </div>
        </div>
    }
}

fn main() {
    mount_to_body(App);
}
