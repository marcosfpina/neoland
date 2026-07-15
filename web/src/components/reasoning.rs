use leptos::prelude::*;

/// Agent reasoning panel — shows the pipeline tree, ADR card, and confidence metrics.
#[component]
pub fn ReasoningPanel(open: ReadSignal<bool>) -> impl IntoView {
    view! {
        <aside
            class=move || if open.get() { "panel reasoning-panel is-open" } else { "panel reasoning-panel" }
            aria-label="Raciocínio do agente"
        >
            <div class="panel-heading">
                <span>"03"</span>
                <h2>"REASONING"</h2>
                <small class="live-label">"LIVE"</small>
            </div>
            <div class="reasoning-body">
                <div class="task-title">
                    <small>"ACTIVE OBJECTIVE"</small>
                    <strong>"Refatorar módulo de auth"</strong>
                    <p>"Preservar ordem, reduzir latência e manter compatibilidade."</p>
                </div>

                <div class="agent-tree">
                    <div class="tree-root">
                        <span class="pulse"></span>
                        "ORCHESTRATOR"
                    </div>
                    <div class="tree-line done">
                        <span>"├─"</span>
                        <i>"J"</i>
                        <div><strong>"Jade"</strong><small>"analysis · 92%"</small></div>
                        <em>"0xA3F"</em>
                    </div>
                    <div class="tree-line done">
                        <span>"├─"</span>
                        <i>"A"</i>
                        <div><strong>"Apex"</strong><small>"benchmark · 88%"</small></div>
                        <em>"0x7C1"</em>
                    </div>
                    <div class="tree-line running">
                        <span>"├─"</span>
                        <i>"R"</i>
                        <div><strong>"Rune"</strong><small>"synthesizing"</small></div>
                        <em>"0.4s"</em>
                    </div>
                    <div class="tree-line">
                        <span>"└─"</span>
                        <i>"W"</i>
                        <div><strong>"Ward"</strong><small>"waiting"</small></div>
                        <em>"—"</em>
                    </div>
                </div>

                <div class="adr-card">
                    <div>
                        <span>"ADR / 024"</span>
                        <b>"ACCEPTED"</b>
                    </div>
                    <strong>"Process pool boundary"</strong>
                    <p>"Use ordered map at CPU-bound boundary. Keep async IO on main executor."</p>
                    <footer>
                        <span>"VERIFIED"</span>
                        <span>"3/3 AGENTS"</span>
                    </footer>
                </div>

                <div class="metrics">
                    <div>
                        <small>"CONFIDENCE"</small>
                        <strong>"94.2%"</strong>
                    </div>
                    <div>
                        <small>"DELTA"</small>
                        <strong>"−41ms"</strong>
                    </div>
                </div>
            </div>
        </aside>
    }
}
