use leptos::prelude::*;

use crate::model::SESSIONS;

/// Sessions sidebar — lists all conversation threads with status indicators.
#[component]
pub fn SessionsPanel(
    active: ReadSignal<usize>,
    set_active: WriteSignal<usize>,
    open: ReadSignal<bool>,
    on_new: Callback<()>,
) -> impl IntoView {
    view! {
        <aside
            class=move || if open.get() { "panel sessions-panel is-open" } else { "panel sessions-panel" }
            aria-label="Sessões"
        >
            <div class="panel-heading">
                <span>"01"</span>
                <h2>"SESSIONS"</h2>
                <small>"04"</small>
            </div>
            <button class="new-chat" on:click=move |_| on_new.run(())>
                <span>"+"</span>
                " NEW THREAD"
                <kbd>"⌘N"</kbd>
            </button>
            <nav class="session-list" aria-label="Lista de conversas">
                {SESSIONS
                    .iter()
                    .enumerate()
                    .map(|(index, session)| {
                        view! {
                            <button
                                class=move || {
                                    if active.get() == index { "session active" } else { "session" }
                                }
                                on:click=move |_| set_active.set(index)
                            >
                                <span class=format!("session-state {}", session.state)></span>
                                <span class="session-copy">
                                    <strong>{session.title}</strong>
                                    <small>{session.time}</small>
                                </span>
                                <span class="chevron">"›"</span>
                            </button>
                        }
                    })
                    .collect_view()}
            </nav>
            <div class="panel-footer">
                <span>"WORKSPACE"</span>
                <strong>"neoland/core"</strong>
                <small>"main · clean"</small>
            </div>
        </aside>
    }
}
