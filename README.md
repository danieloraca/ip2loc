# IP Geolocation Service (Axum + IP2Location.io)

A tiny Rust backend service that geolocates an IP address using the
[IP2Location.io](https://www.ip2location.io/) API.

- Minimal dependencies
- No SDKs
- Returns raw JSON from the provider
- Proper `Content-Type: application/json`
- Optional in‑memory caching for repeated IP lookups
- Rejects private/loopback/non‑routable IPs up front

---

## Requirements

- Rust (stable)
- An IP2Location.io API key

---

## Setup

Clone the repo and set your API key as an environment variable:

```bash
export IP2LOCATIONIO_KEY=your_api_key_here
```

---

## Running the server

```bash
cargo run --release
```

By default, the server will start on: `http://localhost:3000`.

### Caching

The server supports a simple in‑memory cache for IP lookups:

- When enabled, responses from IP2Location.io are cached per IP address.
- Subsequent requests for the same IP within the cache TTL are served from memory
  (no external API call), which reduces latency and API usage.
- The cache is per‑process and in‑memory only (not shared across multiple instances).

Caching is configured via the application state in `src/main.rs`, which constructs
an `AppState` value and passes it into the Axum router:

```rust
use iploc::{AppState, app_with_state};
use std::time::Duration;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("server error: {e}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Build application state from environment variables and construct the app
    // with a 1-hour in-memory cache for IP lookups.
    let state = AppState::from_env_with_cache(Duration::from_secs(3600));
    let app = app_with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

To disable caching entirely, either:

- Use `AppState::from_env()` instead of `from_env_with_cache`, or
- Use `AppState::new(api_key, Duration::from_secs(0))` in tests or custom wiring.

---

## Usage

Send a GET request to `/geo` with the `ip` query parameter:

```bash
curl -i "http://localhost:3000/geo?ip=8.8.8.8"
```

Example successful response (status `200 OK`):

```http
HTTP/1.1 200 OK
content-type: application/json
...

{
  ...raw JSON from IP2Location.io...
}
```

### Error cases

- **Missing API key** (`IP2LOCATIONIO_KEY` not set or unreadable):

  ```http
  HTTP/1.1 500 Internal Server Error

  missing IP2LOCATIONIO_KEY
  ```

- **Invalid IP address** (e.g. `not-an-ip`):

  ```http
  HTTP/1.1 400 Bad Request

  invalid ip
  ```

- **Non‑routable IP address** (loopback, private, link‑local, broadcast, etc.,
  e.g. `127.0.0.1` or `192.168.1.1`):

  ```http
  HTTP/1.1 400 Bad Request

  non-routable ip not allowed
  ```

  These are rejected before calling the upstream provider to avoid pointless
  requests for addresses that cannot be meaningfully geolocated on the public
  Internet.

- **Upstream provider errors** (network issues, non‑2xx status, body read failures)
  return `502 Bad Gateway` with a short text message indicating what went wrong.

---

## Implementation notes

The core server is implemented using:

- **Axum 0.7** with `State<AppState>` for dependency injection
- A shared `reqwest::Client` stored in `AppState` and reused across requests
- An optional in‑memory cache keyed by `IpAddr` with a per‑entry TTL

The main types live in `src/lib.rs`:

- `AppState` – holds:
  - `api_key: Option<Arc<str>>`
  - `cache_ttl: Duration`
  - an internal cache map
  - a shared `reqwest::Client`
- `app_with_state(state: AppState) -> Router` – builds the Axum router with state
- `app() -> Router` – convenience helper using `AppState::from_env()`

---

## Testing

This project includes integration tests using `tokio` and `tower::ServiceExt`
to hit the Axum router directly (no actual TCP socket needed).

Run all tests with:

```bash
cargo test
```

The tests:

- Construct the application with a custom `AppState`, injecting the API key directly
  and controlling the cache TTL for deterministic behavior.
- Verify:
  - Invalid IPs return `400 invalid ip`
  - Missing API key returns `500 missing IP2LOCATIONIO_KEY`
  - Valid IP + key does not return local validation errors
  - Private/loopback IPs (e.g. `127.0.0.1`, `192.168.1.1`) are rejected with
    `400 non-routable ip not allowed`
  - With caching enabled, repeated requests for the same IP return consistent
    HTTP status codes.