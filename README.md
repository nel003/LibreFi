# LibreFi

An open-source hotspot management system for OpenWrt routers. It provides a captive portal, voucher-based access, coinslot integration, QoS per-user bandwidth control, and a full admin control panel — compiled into a single self-contained binary.

> **Author:** Arnel Lopena · [@nel003](https://github.com/nel003)
> **License:** GPL-3.0

---

## How It Works

```
Router (OpenWrt, MIPS)
└── /usr/bin/librefi   ← single binary
    ├── Serves the React SPA (embedded via rust-embed)
    ├── Handles /api/* routes (tiny_http)
    ├── Manages users in a redb database (/etc/librefi/data.redb)
    └── Controls nftables firewall rules for access control
```

The frontend is compiled separately, then embedded into the Rust binary at build time. You only deploy one file to the router.

---

## Prerequisites

### Rust (backend + binary)

1. Install Rust via [rustup](https://rustup.rs/):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install the **nightly** toolchain (required — configured in `rust-toolchain.toml`):
   ```bash
   rustup toolchain install nightly
   rustup component add rust-src --toolchain nightly
   ```

3. Verify:
   ```bash
   rustc --version   # should say nightly
   cargo --version
   ```

### Node.js (frontend)

Install **Node.js ≥ 18** from [nodejs.org](https://nodejs.org). npm comes bundled with it.

```bash
node --version   # ≥ 18
npm --version    # ≥ 9
```

---

## Step 1 — Build the Frontend

```bash
cd librefi-reactjs
npm install
npm run build
cd ..
```

Vite outputs directly to `dist/` at the **project root** (not inside `librefi-reactjs/`). The compression plugin automatically:
- Gzips all JS and CSS assets in `dist/assets/`
- Deletes the uncompressed originals (only `.js.gz` and `.css.gz` are kept)
- Leaves `index.html` and fonts untouched

Resulting `dist/` structure:
```
dist/
├── index.html
└── assets/
    ├── index-[hash].js.gz
    └── index-[hash].css.gz
```

The backend (`rust-embed`) embeds this directory at compile time and serves `.gz` files with a `Content-Encoding: gzip` header automatically.

---

## Step 2 — Compile the Rust Backend

### A. For your own machine (development / testing)

This builds a native binary for your current OS (Linux, macOS, or Windows):

```bash
# Debug build (faster compile, bigger binary, debug output enabled)
cargo build

# Release build (optimized, smaller binary)
cargo build --release
```

Binary output:
```
target/debug/librefi         ← debug
target/release/librefi       ← release
```

Run it locally (the backend listens on port 80 by default, or set `PORT` if needed):
```bash
LIBREFI_KEY="my-dev-key-123456" ./target/debug/librefi
```

> **Dev note:** The frontend source has API calls hardcoded to `http://localhost:8000` — this is a **Docker proxy** used during development so the Vite dev server (`localhost:5173`) can reach the Rust backend (`localhost:8000`). In production the frontend is embedded in the binary and served from the same origin, so no cross-origin requests are made.

---


### B. For the OpenWrt Router (cross-compile)

OpenWrt routers run on many different CPU architectures. You need to cross-compile for the correct target.

#### 1. Find your router's architecture

SSH into the router and run:

```bash
uname -m
cat /proc/cpuinfo | grep "cpu model\|system type\|machine"
```

Then map the output to the correct Rust target:

| `uname -m` output | CPU family | Rust target |
|--------------------|------------|-------------|
| `mips` | MIPS 32-bit big-endian (e.g. TP-Link, Netgear WNR) | `mips-unknown-linux-musl` |
| `mipsel` | MIPS 32-bit little-endian (e.g. TP-Link Archer, MT7621) | `mipsel-unknown-linux-musl` |
| `mips64` | MIPS 64-bit big-endian | `mips64-unknown-linux-muslabi64` |
| `armv7l` | ARM 32-bit (e.g. RPi 2, Linksys WRT) | `armv7-unknown-linux-musleabihf` |
| `aarch64` | ARM 64-bit (e.g. RPi 4, GL.iNet AXT1800) | `aarch64-unknown-linux-musl` |
| `x86_64` | x86 64-bit (e.g. PC Engines APU, x86 routers) | `x86_64-unknown-linux-musl` |

> You can also check the [OpenWrt Table of Hardware](https://openwrt.org/toh/start) — search for your device and look at the **Architecture** column.

#### 2. Install Docker

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) (Windows/macOS)
- Or Docker Engine on Linux

Make sure Docker is running before the next step.

#### 3. Install `cross`

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

#### 4. Cross-compile

Replace `<TARGET>` with the Rust target for your router from the table above:

```bash
cross build --release --target <TARGET>
```

**Example for a MIPS router (TP-Link, Netgear):**
```bash
cross build --release --target mips-unknown-linux-musl
```

**Example for a MIPSEL router (MT7621, TP-Link Archer):**
```bash
cross build --release --target mipsel-unknown-linux-musl
```

**Example for an ARM64 router (GL.iNet AXT1800, RPi 4):**
```bash
cross build --release --target aarch64-unknown-linux-musl
```

> **Note:** `Cross.toml` in the project root is pre-configured for `mips-unknown-linux-musl`. For other targets, `cross` will pull the appropriate Docker image automatically on first run.

Binary output:
```
target/<TARGET>/release/librefi
```

Check the binary size (should be ~2–4 MB after stripping):
```bash
ls -lh target/<TARGET>/release/librefi
```

---

> [!CAUTION]
> **Before running the binary — read this first.**
>
> LibreFi is designed to run as a **secondary router/access point** behind your main router.
> The router running LibreFi has two sides:
>
> | Side | Interface | Who is here |
> |------|-----------|-------------|
> | **WAN** | `wan` (upstream) | Your main router — where SSH and the admin panel are reachable |
> | **LAN** | `br-lan` | Hotspot clients — captive portal users only |
>
> - **SSH** into the LibreFi router is only reachable from the **WAN side** (from your main router's network).
> - The **admin panel** (`/admin`) is blocked from the LAN side by design — clients can only see the captive portal.
> - **Client isolation** on your main router **must be disabled**. If client isolation is on, devices on the main router's WiFi cannot reach the LibreFi router's WAN IP, which means you will lose SSH access and won't be able to reach the admin panel.
>
> To disable client isolation: go to your main router's wireless settings and turn off **"AP Isolation"**, **"Client Isolation"**, or **"Wireless Isolation"** (the name varies by router brand).

## Step 3 — Deploy to the Router

### 1. Copy the binary

```bash
scp target/<TARGET>/release/librefi root@<router-ip>:/usr/bin/librefi
chmod +x /usr/bin/librefi

```

### 2. Set the admin encryption key

The binary **will not start** without `LIBREFI_KEY` set to a strong value (≥ 16 characters). This key encrypts all admin API communication.

Create `/etc/librefi.env`:
```sh
LIBREFI_KEY=change-this-to-a-strong-secret-key
```

Or export it directly in your init script.

### 3. Create an OpenWrt init script

`/etc/init.d/librefi`:
```sh
#!/bin/sh /etc/rc.common
START=99
STOP=10

start() {
    export LIBREFI_DEBUG="false" #true
    export LIBREFI_KEY="change-this-to-a-strong-secret-key"
    procd_open_instance
    procd_set_param command /usr/bin/librefi
    procd_set_param respawn
    procd_close_instance
}

stop() {
    killall librefi
}
```

Enable and start:
```bash
chmod +x /etc/init.d/librefi
/etc/init.d/librefi enable
/etc/init.d/librefi start
```

### 4. Verify

Open a browser from a device connected to the router's LAN and navigate to `http://10.0.0.1`. You should see the LibreFi captive portal.

---

## Step 4 — Admin Access

The admin panel is at `http://10.0.0.1/admin` (or your router's LAN IP).

- The admin password is your `LIBREFI_KEY` value.
- All admin API calls are encrypted with AES-256-GCM using that key.
- Use `admin-encryptor.html` (in the project root) to manually encrypt payloads for debugging.

---

## Running Integration Tests

Tests require a running backend. Start the backend first, then:

```bash
cargo test -- --test-threads=1
```

> Tests are in `tests/api_tests.rs` and cover all admin endpoints.

---

## Rebuilding After Changes

| Changed | Run |
|---------|-----|
| Frontend only | `cd librefi-reactjs && npm run build && cd .. && cargo build --release` |
| Backend only | `cargo build --release` |
| Both | Build frontend first, then Rust |
| For router | Replace `cargo build` with `cross build --release --target mips-unknown-linux-musl` |

---

## Project Structure

```
librefi/
├── src/                        # Rust backend source
│   ├── main.rs                 # Entry point, route registration
│   ├── network/server.rs       # HTTP server (tiny_http wrapper)
│   ├── routes/
│   │   ├── admin/              # Admin API routes (auth, users, rates, qos, wifi…)
│   │   ├── redeem.rs           # Voucher redemption
│   │   ├── init.rs             # User session init
│   │   ├── play_pause.rs       # Pause/resume user
│   │   └── coin.rs             # Coinslot webhook
│   └── utils/
│       ├── db.rs               # redb database + models
│       ├── crypto.rs           # AES-256-GCM
│       └── setup_captive_portal.rs  # nftables helpers
├── librefi-reactjs/            # React frontend (Vite + Tailwind)
├── tests/api_tests.rs          # Integration tests
├── Cross.toml                  # cross-rs config for MIPS
├── rust-toolchain.toml         # Pins Rust nightly
├── Cargo.toml
└── ROUTES.md                   # Full API reference
```

---

## API Reference

See [`ROUTES.md`](./ROUTES.md) for a full description of every endpoint, request format, and response schema.

---

## License

GPL-3.0 — see [LICENSE](./LICENSE).
