# openfi API Routes

## Public Routes

### `GET /`
Captive portal entry point. Validates the connecting device via MAC address lookup in `/proc/net/arp`.

- **200** — Serves `dist/index.html` (React app)
- **403** — Device MAC not recognized → `<h1>This system is not for your device.</h1>`
- **500** — Could not read `dist/index.html`

---

### `GET /init`
Returns the stored user record for the requesting device. Called by the frontend on load to hydrate state.

- **200** — User found; returns JSON user object + current timestamp
  ```json
  {
    "name": "",
    "ip": "192.168.1.42",
    "paused": false,
    "pause_attempts": 0,
    "paused_on": 0,
    "expires_on": 0,
    "now": 1756547416
  }
  ```
- **403** — MAC address not found in ARP table → `{"error":"Device not recognized"}`
- **404** — MAC found but user not yet in database → `{"error":"User not found"}`

---

### `GET /rates`
Returns all available internet plans stored in the rates table.

- **200** — List of rate rows
  ```json
  {
    "count": 2,
    "rows": [
      { "id": 1, "rate": { "price": 15.0, "time": 3600, "pause_attempts": 3 } },
      { "id": 2, "rate": { "price": 25.0, "time": 7200, "pause_attempts": 5 } }
    ]
  }
  ```

---

### `GET /status`
Returns current system resource usage. Reads `/proc/meminfo` and `df /overlay`.

- **200** — Resource stats
  ```json
  {
    "ram_used_pct": 42.3,
    "ram_free_pct": 57.7,
    "storage_used_pct": 18,
    "storage_free_pct": 82
  }
  ```
- **500** — Shell command failed

---

### `POST /play_pause`
Toggles the internet pause state for a user. Users are limited to 3 pauses per day.

- **200** — Pause/Play toggled successfully
  ```json
  {
    "action": "pause",
    "paused": true,
    "paused_on": 1756547416,
    "expires_on": 1756551016,
    "pause_attempts": 1,
    "now": 1756547416
  }
  ```
- **429** — Daily pause limit reached
  ```json
  {
    "error": "Daily pause limit reached (3/3)",
    "pause_attempts": 3,
    "resets_in": 12345
  }
  ```
- **403** — Device MAC not recognized
- **404** — User not found

---

## Admin Routes

> All admin `POST` requests require the body to be **AES-256-GCM encrypted**.
> The key is derived from `ADMIN_SECRET_KEY` in `src/routes/admin.rs`.

### `POST /admin/rates`
Insert or overwrite a rate entry. Key = `id` (u32).

**Plaintext payload (before encryption):**
```json
{ "id": 1, "price": 15.0, "time": 3600 }
```

| Field | Type | Description |
|---|---|---|
| `id` | `u32` | Rate ID (used as database key) |
| `price` | `f64` | Price in local currency |
| `time` | `u32` | Session duration in seconds |

- **200** — Rate saved → `{"ok":true,"id":1,"rate":{...}}`
- **400** — Invalid JSON payload → `{"error":"Invalid payload: ..."}`
- **401** — Decryption failed (wrong key) → `{"error":"Decryption failed: ..."}`

---

### `POST /admin/vouchers`
Insert or overwrite a voucher coupon. Key = `code` (string). Inserting the same code again overwrites the price.

**Plaintext payload (before encryption):**
```json
{ "code": "PROMO2024", "price": 15.0 }
```

| Field | Type | Description |
|---|---|---|
| `code` | `String` | Unique voucher code (used as database key) |
| `price` | `f64` | Voucher value in local currency |

> `used` is always set to `false` on insert. It will be flipped to `true` when the voucher is redeemed.

- **200** — Voucher saved → `{"ok":true,"code":"PROMO2024","voucher":{...}}`
- **400** — Invalid JSON payload → `{"error":"Invalid voucher payload: ..."}`
- **401** — Decryption failed (wrong key) → `{"error":"Decryption failed: ..."}`

---

## Debug Routes

### `GET /debug/db`
Dumps all rows from the `users` table. Useful during development.

- **200** — Full table contents
  ```json
  {
    "table": "users",
    "count": 1,
    "rows": [
      {
        "mac": "aa:bb:cc:dd:ee:ff",
        "name": "",
        "ip": "192.168.1.42",
        "paused": false,
        "pause_attempts": 0,
        "paused_on": 0,
        "expires_on": 0
      }
    ]
  }
  ```

---

## Route → File Map

| Route | File |
|---|---|
| `GET /` | `src/routes/index.rs` |
| `GET /init` | `src/routes/init.rs` |
| `GET /rates` | `src/routes/rates.rs` |
| `GET /status` | `src/routes/status.rs` |
| `POST /play_pause` | `src/routes/play_pause.rs` |
| `POST /admin/rates` | `src/routes/admin.rs` |
| `POST /admin/vouchers` | `src/routes/admin.rs` |
| `GET /debug/db` | `src/routes/debug.rs` |
| `GET /dist/*` | `src/network/server.rs` (static file fallback) |
