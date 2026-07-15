use leptos::{ev, prelude::*};

/// Input composer — text input with send/cancel and keyboard shortcuts bar.
#[component]
pub fn Composer(
    input: ReadSignal<String>,
    set_input: WriteSignal<String>,
    streaming: ReadSignal<bool>,
    submit: Callback<()>,
    cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <footer class="composer-wrap">
            <div class="composer-status">
                <span><i class="status-dot"></i>"LOCAL / BALANCED"</span>
                <span>
                    {move || {
                        if streaming.get() { "AGENT IS SYNTHESIZING" } else { "AWAITING INTERVENTION" }
                    }}
                </span>
            </div>
            <div class="composer">
                <span class="prompt">"›"</span>
                <input
                    aria-label="Mensagem para o Neoland"
                    placeholder="Digite uma instrução para o Neoland..."
                    prop:value=move || input.get()
                    on:input=move |ev| set_input.set(event_target_value(&ev))
                    on:keydown=move |ev: ev::KeyboardEvent| {
                        if ev.key() == "Enter" && !ev.is_composing() && ev.key_code() != 229 {
                            ev.prevent_default();
                            submit.run(());
                        }
                    }
                />
                <Show
                    when=move || streaming.get()
                    fallback=move || {
                        view! { <button class="send-button" on:click=move |_| submit.run(())>"RUN ↵"</button> }
                    }
                >
                    <button class="cancel-button" on:click=move |_| cancel.run(())>"STOP ×"</button>
                </Show>
            </div>
            <div class="shortcuts">
                <span>"ENTER  SEND"</span>
                <span>"⌘N  NEW"</span>
                <span>"/WHY  TRACE"</span>
                <span>"ESC  CLOSE"</span>
            </div>
        </footer>
    }
}
