use leptos::prelude::*;

/// Application top bar — brand, telemetry, and action buttons.
#[component]
pub fn Topbar(on_sessions: Callback<()>, on_reasoning: Callback<()>) -> impl IntoView {
    view! {
        <header class="topbar">
            <div class="brand-lockup">
                <span class="brand-mark">"N"</span>
                <strong>"NEOLAND"</strong>
                <span class="version">"// v0.9.4"</span>
            </div>
            <div class="telemetry" aria-label="Telemetria do sistema">
                <span><i class="status-dot"></i>"LOCAL"</span>
                <span>"14ms"</span>
                <span>"873343 tok"</span>
                <span>"BALANCED"</span>
            </div>
            <div class="top-actions">
                <button
                    class="mobile-control sessions-toggle"
                    on:click=move |_| on_sessions.run(())
                    aria-label="Alternar sessões"
                >
                    "[ S ]"
                </button>
                <button class="icon-button" aria-label="Notificações">
                    "BELL:0"
                </button>
                <button
                    class="mobile-control"
                    on:click=move |_| on_reasoning.run(())
                    aria-label="Alternar raciocínio"
                >
                    "[ R ]"
                </button>
                <button class="icon-button" aria-label="Configurações">
                    "CFG"
                </button>
            </div>
        </header>
    }
}
