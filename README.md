# IP Geolocation Service (Axum + IP2Location.io)

A tiny Rust backend service that geolocates an IP address using the
[IP2Location.io](https://www.ip2location.io/) API.

- Minimal dependencies
- No SDKs
- Returns raw JSON from the provider
- Proper `Content-Type: application/json`
- Optional in‑memory caching for repeated IP lookups

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

The cache behavior and TTL are configured in code:

- The `AppConfig` struct (in `src/lib.rs`) has a `cache_ttl: Duration` field.
- In `src/main.rs`, the app is currently built with a 3600‑second cache:

  ```rust
  let config = AppConfig::from_env_with_cache(Duration::from_secs(60));
  ```

- Setting `cache_ttl` to `0` disables caching.

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

- **Upstream provider errors** (network issues, non‑2xx status, body read failures)
  return `502 Bad Gateway` with a short text message indicating what went wrong.

---

## Testing

This project includes basic integration tests using `tokio` and `tower::ServiceExt`
to hit the Axum router directly (no actual TCP socket needed).

Run all tests with:

```bash
cargo test
```

The tests construct the application with a custom `AppConfig`, injecting the API key
directly and disabling caching for deterministic behavior.
