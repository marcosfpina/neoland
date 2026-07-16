// Neoland Service Worker — v0.3.0
//
// Strategy: Cache-first for static assets (WASM, CSS, JS), network-first for API.
// On install, pre-caches the app shell so it loads offline.

const CACHE_NAME = "neoland-v0.3.0";
const APP_SHELL = [
  "/",
  "/index.html",
  "/neoland.css",
  "/manifest.json",
];

// ── Install: pre-cache static shell ────────────────────────────────────
self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(APP_SHELL);
    }).then(() => self.skipWaiting())
  );
});

// ── Activate: clean old caches ─────────────────────────────────────────
self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((keys) => {
      return Promise.all(
        keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k))
      );
    }).then(() => self.clients.claim())
  );
});

// ── Fetch: cache-first for static, network-first for API ───────────────
self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);

  // API calls — bypass cache, go straight to network
  if (url.pathname.startsWith("/v1/") || url.pathname === "/health" || url.pathname === "/metrics") {
    return;
  }

  // Static assets — cache-first
  event.respondWith(
    caches.match(event.request).then((cached) => {
      const fetchPromise = fetch(event.request).then((response) => {
        if (response && response.status === 200) {
          const clone = response.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(event.request, clone);
          });
        }
        return response;
      });

      return cached || fetchPromise;
    })
  );
});
