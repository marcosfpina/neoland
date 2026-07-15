use leptos::prelude::*;

use crate::model::Message;

/// Individual chat bubble — user messages right-aligned, agent messages left-aligned.
#[component]
fn MessageBubble(message: Message, index: usize) -> impl IntoView {
    let is_user = message.role == "user";
    view! {
        <article class=if is_user { "message user-message" } else { "message agent-message" }>
            <div class="message-meta">
                <span class=if is_user { "identity user-id" } else { "identity agent-id" }>
                    {if is_user { "YOU" } else { "NEOLAND" }}
                </span>
                <span>{format!("0{} // 14:2{}:0{}", index + 1, index + 2, index)}</span>
            </div>
            <div class="bubble">
                <p>{message.body}</p>
                {message.code.map(|code| {
                    view! {
                        <div class="code-block">
                            <div class="code-head">
                                <span>"PYTHON"</span>
                                <span>"optimized.py"</span>
                            </div>
                            <pre><code>{code}</code></pre>
                        </div>
                    }
                })}
            </div>
        </article>
    }
}

/// Main conversation panel — renders message history with streaming animation.
#[component]
pub fn Conversation(
    messages: ReadSignal<Vec<Message>>,
    streaming: ReadSignal<bool>,
    stream_text: ReadSignal<String>,
) -> impl IntoView {
    view! {
        <main class="panel conversation-panel">
            <div class="panel-heading">
                <span>"02"</span>
                <h1>"CONVERSATION"</h1>
                <small>"AUTH / PERF"</small>
            </div>
            <div class="conversation-scroll" aria-live="polite">
                <div class="thread-marker">
                    <span>"THREAD 873343"</span>
                    <i></i>
                    <span>"TODAY"</span>
                </div>
                {move || {
                    messages
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, m)| view! { <MessageBubble message=m index=i /> })
                        .collect_view()
                }}
                <Show when=move || streaming.get()>
                    <article class="message agent-message streaming-message">
                        <div class="message-meta">
                            <span class="identity agent-id">"NEOLAND"</span>
                            <span class="live-label">"STREAMING"</span>
                        </div>
                        <div class="bubble">
                            <p>{move || stream_text.get()}<span class="cursor">"█"</span></p>
                        </div>
                    </article>
                </Show>
            </div>
        </main>
    }
}
