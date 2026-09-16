# LibreFi — Frontend (React + Vite)

The LibreFi frontend is a React + TypeScript + Vite SPA that serves as both the **user-facing captive portal** and the **admin control panel** for the LibreFi hotspot backend.

---

## Tech Stack

| Tool | Purpose |
|------|---------|
| React 19 | UI framework |
| TypeScript | Type safety |
| Vite 8 | Dev server & bundler |
| Tailwind CSS v4 | Styling |
| shadcn/ui (Base UI) | Component library |
| Wouter | Client-side routing |

---

## Prerequisites

- **Node.js** ≥ 18 ([nodejs.org](https://nodejs.org))
- **npm** ≥ 9 (comes with Node.js)
- The **LibreFi Rust backend** running on port 8000 (for dev proxying)

---

## Development Setup

### 1. Install dependencies

```bash
cd librefi-reactjs
npm install
```

### 2. Run the Rust backend (via Docker for testing)

For local development, the frontend fetch calls point to `http://localhost:8000` — this is a **dev-only** proxy to the Rust backend running in Docker. In production the frontend is embedded inside the binary and served from the same origin, so no cross-origin calls are made.

Start the backend in Docker (from the project root):

```bash
docker run -p 8000:80 -e LIBREFI_KEY=dev-key-123456 librefi
```

Or run the native binary directly:

```bash
LIBREFI_KEY=dev-key-123456 ./target/debug/librefi
```

### 3. Start the dev server

```bash
npm run dev
```

The dev server starts at `http://localhost:5173` with HMR. API calls go to `http://localhost:8000/api/*` (the Rust backend).

---

## Building for Production

### 1. Build the frontend

```bash
npm run build
```

This runs `tsc` (type-check) then `vite build`. The output goes directly to **`dist/` at the project root** (one level up from `librefi-reactjs/`). The compression plugin automatically gzips all JS and CSS, then deletes the uncompressed originals.

Resulting structure:
```
../dist/         ← project root
├── index.html
└── assets/
    ├── index-[hash].js.gz
    └── index-[hash].css.gz
```

### 2. Rebuild the Rust backend

The `dist/` directory is embedded at compile time by `rust-embed`. After every frontend build, rebuild the Rust binary:

```bash
cd ..
cargo build --release            # native
# or
cross build --release --target mips-unknown-linux-musl   # for the router
```

---

## Cross-Compiling for OpenWrt (MIPS)

The target router runs **OpenWrt on a MIPS (musl)** CPU. The Rust backend must be cross-compiled accordingly.

### Requirements

- **Docker** (required by `cross`)
- **cross** — install once:

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

- **Rust nightly** toolchain (configured via `rust-toolchain.toml`):

```bash
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

### Cross-compile

```bash
# From the project root (d:\Rust\librefi)
cross build --release --target mips-unknown-linux-musl
```

The `Cross.toml` handles the Docker image and build flags automatically.

Output binary:
```
target/mips-unknown-linux-musl/release/librefi
```

---

## Deploying to the Router

### 1. Transfer the binary

```bash
scp target/mips-unknown-linux-musl/release/librefi root@<router-ip>:/usr/bin/librefi
```

### 2. Set the admin key

The backend **refuses to start** without a strong `LIBREFI_KEY` environment variable (minimum 16 characters):

```sh
# /etc/init.d/librefi  (OpenWrt init script)
export LIBREFI_KEY="your-strong-secret-key-here"
/usr/bin/librefi
```

Or set it in `/etc/environment`:
```
LIBREFI_KEY=your-strong-secret-key-here
```

### 3. Start the backend

```bash
LIBREFI_KEY=your-key /usr/bin/librefi
```

The backend listens on **port 80** and serves both the frontend and all `/api/*` routes.

---

## API Routes

All API endpoints are prefixed with `/api`. The frontend is served from `/`.

| Method | Route | Description |
|--------|-------|-------------|
| `GET` | `/api/init` | Load user session |
| `GET` | `/api/rates` | List pricing rates |
| `POST` | `/api/redeem` | Redeem a voucher |
| `POST` | `/api/play_pause` | Pause / resume user |
| `GET/POST` | `/api/admin/dashboard` | Dashboard stats |
| `GET/POST/PUT/DELETE` | `/api/admin/rates` | Manage rates |
| `GET/POST` | `/api/admin/users` | Manage users |
| `GET/POST/DELETE` | `/api/admin/vouchers` | Manage vouchers |
| `GET/POST` | `/api/admin/qos` | QoS settings |
| `GET/POST` | `/api/admin/wifi` | WiFi settings |
| `GET/POST` | `/api/admin/coinslot-key` | Coinslot key |

---

## Admin Authentication

All admin routes require an **AES-256-GCM encrypted payload** encoded in Base64, passed as:

- A `payload` query parameter (for `GET` requests)
- A `payload` field in the JSON body (for `POST`/`PUT`/`DELETE` requests)

Use the included `admin-encryptor.html` (in the project root) to manually generate encrypted payloads for testing.

---

## Project Structure

```
librefi-reactjs/
├── src/
│   ├── components/
│   │   ├── admin/          # Admin panel components
│   │   │   ├── Dashboard.tsx
│   │   │   ├── Users.tsx
│   │   │   ├── VouchRates.tsx
│   │   │   ├── Settings.tsx
│   │   │   └── Layout.tsx
│   │   ├── Timer.tsx       # Countdown timer
│   │   ├── Voucher.tsx     # Voucher redemption
│   │   ├── Coin.tsx        # Coin insert UI
│   │   └── More.tsx        # Rates & extras
│   ├── routes/
│   │   ├── user.tsx        # Captive portal (user-facing)
│   │   └── admin.tsx       # Admin login & panel
│   ├── lib/
│   │   ├── crypto.ts       # AES-256-GCM encrypt/decrypt
│   │   └── utils.ts        # cn(), formatSeconds(), etc.
│   └── App.tsx             # Root router
├── vite.config.ts
├── package.json
└── tsconfig.app.json
```

---

## Available Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server with HMR |
| `npm run build` | Type-check and build for production |
| `npm run lint` | Run ESLint |
| `npm run preview` | Preview the production build locally |

---

## License

GPL-3.0 — see [LICENSE](../LICENSE).

© Arnel Lopena · [github.com/nel003/LibreFi](https://github.com/nel003/LibreFi)
