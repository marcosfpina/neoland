use leptos::prelude::*;

/// Metrics dashboard panel — displays pipeline performance data.
///
/// Shows latency, token usage, and confidence scores from the current
/// pipeline run.  Data comes from SSE events (`stage_done` provides
/// latency and confidence; `pipeline_done` provides total latency).
#[allow(dead_code)]
#[component]
pub fn MetricsPanel(
    /// Total pipeline latency in milliseconds (from `pipeline_done`)
    total_latency_ms: ReadSignal<Option<u64>>,
    /// Per-stage entries: (stage_name, latency_ms, confidence)
    stages: ReadSignal<Vec<(String, u64, Option<f32>)>>,
    /// Total tokens consumed (estimated from response length)
    token_count: ReadSignal<u64>,
) -> impl IntoView {
    view! {
        <div class="metrics-panel">
            <h3 class="metrics-title">"Pipeline Metrics"</h3>

            // ── Total latency ──────────────────────────────────────────
            <div class="metric-row">
                <span class="metric-label">"Total latency"</span>
                <span class="metric-value">
                    {move || {
                        if let Some(ms) = total_latency_ms.get() {
                            format!("{ms} ms")
                        } else {
                            "—".to_string()
                        }
                    }}
                </span>
            </div>

            // ── Token count ────────────────────────────────────────────
            <div class="metric-row">
                <span class="metric-label">"Tokens"</span>
                <span class="metric-value">
                    {move || {
                        let n = token_count.get();
                        if n > 0 {
                            format!("~{n}")
                        } else {
                            "—".to_string()
                        }
                    }}
                </span>
            </div>

            // ── Per-stage breakdown ────────────────────────────────────
            <div class="metric-stages">
                <h4 class="metrics-subtitle">"Stages"</h4>
                <For
                    each=move || stages.get()
                    key=|s| s.0.clone()
                    children=move |(name, latency, confidence)| {
                        let confidence_class = match confidence {
                            Some(c) if c >= 0.9 => "conf-high",
                            Some(c) if c >= 0.7 => "conf-mid",
                            Some(_) => "conf-low",
                            None => "conf-none",
                        };
                        view! {
                            <div class="stage-row">
                                <span class="stage-name">{name}</span>
                                <span class="stage-latency">{format!("{latency} ms")}</span>
                                <span class={format!("stage-confidence {confidence_class}")}>
                                    {move || {
                                        match confidence {
                                            Some(c) => format!("{:.0}%", c * 100.0),
                                            None => "—".to_string(),
                                        }
                                    }}
                                </span>
                            </div>
                        }
                    }
                />
            </div>

            // ── Throughput (tokens/sec) ─────────────────────────────────
            <div class="metric-row">
                <span class="metric-label">"Throughput"</span>
                <span class="metric-value">
                    {move || {
                        let tokens = token_count.get() as f64;
                        if let Some(ms) = total_latency_ms.get() {
                            if ms > 0 && tokens > 0.0 {
                                let tps = tokens / (ms as f64 / 1000.0);
                                format!("{:.1} tok/s", tps)
                            } else {
                                "—".to_string()
                            }
                        } else {
                            "—".to_string()
                        }
                    }}
                </span>
            </div>
        </div>
    }
}
