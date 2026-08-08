//! Shared test harness for the integration test binaries.
//!
//! Cargo compiles `tests/common/mod.rs` into every test target that declares
//! `mod common;` — it is not itself a test binary. Each test binary runs in
//! its own process, so the env vars set here never leak across binaries.
#![allow(dead_code)]

use std::sync::OnceLock;
use std::time::{Duration, Instant};

#[allow(unused_imports)]
pub use neoland::auth::dev_keys;

/// In-process Neoland server bound to ephemeral ports (`127.0.0.1:0`).
///
/// One instance per test binary, started on first use and shared by every
/// test in the binary. It lives in a dedicated OS thread with its own tokio
/// runtime so it survives the per-test runtimes that `#[tokio::test]`
/// creates and destroys.
pub struct TestServer {
    pub rest_url: String,
    pub grpc_url: String,
    shutdown: tokio::sync::broadcast::Sender<()>,
}

static SHARED: OnceLock<TestServer> = OnceLock::new();

impl TestServer {
    /// Returns the shared server for this test binary, booting it on first
    /// call and waiting until `/health` responds 200.
    pub async fn shared() -> &'static TestServer {
        let server = SHARED.get_or_init(Self::start);
        server.wait_healthy().await;
        server
    }

    fn start() -> TestServer {
        setup_test_env();

        // Bind on the caller side so the addresses are known before the
        // server thread takes over; port 0 → the OS picks free ports and
        // parallel test binaries can no longer collide.
        let grpc_std = std::net::TcpListener::bind("127.0.0.1:0").expect("bind grpc");
        let rest_std = std::net::TcpListener::bind("127.0.0.1:0").expect("bind rest");
        grpc_std.set_nonblocking(true).expect("nonblocking grpc");
        rest_std.set_nonblocking(true).expect("nonblocking rest");
        let grpc_addr = grpc_std.local_addr().expect("grpc addr");
        let rest_addr = rest_std.local_addr().expect("rest addr");

        let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);
        let external_shutdown = shutdown_tx.clone();

        std::thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build server runtime")
                .block_on(async move {
                    let grpc = tokio::net::TcpListener::from_std(grpc_std).expect("grpc listener");
                    let rest = tokio::net::TcpListener::from_std(rest_std).expect("rest listener");
                    if let Err(e) =
                        neoland::server::run_server_with(grpc, rest, "", Some(external_shutdown))
                            .await
                    {
                        eprintln!("[TEST SERVER] run_server_with failed: {e}");
                    }
                });
        });

        TestServer {
            rest_url: format!("http://{rest_addr}"),
            grpc_url: format!("http://{grpc_addr}"),
            shutdown: shutdown_tx,
        }
    }

    /// Polls `/health` until 200 or panics after 30s. With embeddings stubbed
    /// (`NEOLAND_SKIP_EMBEDDINGS`) a healthy boot takes well under a second.
    async fn wait_healthy(&self) {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("http client");
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if let Ok(resp) = client.get(format!("{}/health", self.rest_url)).send().await {
                if resp.status() == reqwest::StatusCode::OK {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("test server did not become healthy within 30s at {}", self.rest_url);
    }

    /// Requests a graceful shutdown. Rarely needed — the shared server lives
    /// for the test binary's lifetime.
    pub fn stop(&self) {
        let _ = self.shutdown.send(());
    }
}

/// Hermetic environment for the in-process server, applied once per binary.
fn setup_test_env() {
    static ENV_READY: OnceLock<()> = OnceLock::new();
    ENV_READY.get_or_init(|| {
        // SAFETY: called before the server thread spawns, from the first test
        // that touches TestServer; test binaries are single-process.
        unsafe {
            // Fail fast instead of calling a live DSPy pipeline.
            std::env::set_var("NEOLAND_DSPY_URL", "http://127.0.0.1:9");
            std::env::set_var("NEOLAND_PIPELINE_TIMEOUT_SECS", "2");
            std::env::set_var("NEOLAND_NATS_ENABLED", "false");
            std::env::set_var(
                "AUDIT_LOG_PATH",
                format!("/tmp/neoland-test-audit-{}.log", std::process::id()),
            );
            // Deterministic 384-dim stub instead of a HuggingFace download.
            std::env::set_var("NEOLAND_SKIP_EMBEDDINGS", "true");
            // Exercise database-degraded behavior unless the test opts in.
            if std::env::var("NEOLAND_TEST_KEEP_DATABASE").is_err() {
                std::env::remove_var("DATABASE_URL");
                std::env::remove_var("NEOLAND_DATABASE_URL");
            }
        }
    });
}

/// True when the CI e2e job demands that external dependencies be present
/// (`NEOLAND_TEST_REQUIRE_DEPS=1`) — missing deps must FAIL, not skip.
pub fn deps_required() -> bool {
    std::env::var("NEOLAND_TEST_REQUIRE_DEPS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Skip-or-fail policy for tests that need external dependencies.
///
/// Call when a dependency (Postgres, DSPy, NATS) is unavailable, then
/// `return` from the test. Locally this prints a visible skip; in CI with
/// `NEOLAND_TEST_REQUIRE_DEPS=1` it panics so a missing service turns the
/// job red instead of silently passing 0 assertions.
pub fn skip_or_fail(dep: &str, hint: &str) {
    if deps_required() {
        panic!("{dep} unavailable but NEOLAND_TEST_REQUIRE_DEPS=1 — {hint}");
    }
    eprintln!("[SKIP] {dep} unavailable — {hint}");
}
