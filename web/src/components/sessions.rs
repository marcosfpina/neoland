use leptos::prelude::*;

use crate::api::Session;

/// Sessions sidebar — lists all conversation threads with status indicators.
#[component]
pub fn SessionsPanel(
    sessions: ReadSignal<Vec<Session>>,
    active: ReadSignal<usize>,
    set_active: WriteSignal<usize>,
    on_select: Callback<usize>,
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
                <small>{move || sessions.get().len().to_string()}</small>
            </div>
            <button class="new-chat" on:click=move |_| on_new.run(())>
                <span>"+"</span>
                " NEW THREAD"
                <kbd>"⌘N"</kbd>
            </button>
            <nav class="session-list" aria-label="Lista de conversas">
                {move || {
                    sessions
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(index, session)| {
                            let name = session.session_name.clone();
                            let last = session.last_activity.clone();
                            let active_flag = session.active;
                            view! {
                                <button
                                    class=move || {
                                        if active.get() == index { "session active" } else { "session" }
                                    }
                                    on:click=move |_| {
                                        set_active.set(index);
                                        on_select.run(index);
                                    }
                                >
                                    <span class=format!(
                                        "session-state {}",
                                        if active_flag { "live" } else { "idle" },
                                    )></span>
                                    <span class="session-copy">
                                        <strong>{name.clone()}</strong>
                                        <small>{last.clone()}</small>
                                    </span>
                                    <span class="chevron">"›"</span>
                                </button>
                            }
                        })
                        .collect_view()
                }}
            </nav>
            <div class="panel-footer">
                <span>"WORKSPACE"</span>
                <strong>"neoland/core"</strong>
                <small>"main · clean"</small>
            </div>
        </aside>
    }
}
