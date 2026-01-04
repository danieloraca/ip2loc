# IP Geolocation Service (Axum + IP2Location.io)

A tiny Rust backend service that geolocates an IP address using the
[IP2Location.io](https://www.ip2location.io/) API.

- Minimal dependencies
- No SDKs
- Returns raw JSON from the provider
- Proper `Content-Type: application/json`

---

## Requirements

- Rust (stable)
- An IP2Location.io API key

---

## Setup

Clone the repo and set your API key as an environment variable:

```bash
export IP2LOCATIONIO_KEY=your_api_key_here

---

## Then run:

```bash
cargo run --release

The server will start on: `http://localhost:3000`

---

## Usage

Send a GET request to `/ip` with the `ip` query parameter:

```bash
curl -i http://localhost:3000/ip?ip=8.8.8.8
```
