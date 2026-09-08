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
    "mac": "aa:bb:cc:dd:ee:ff",
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

### `PUT /admin/rates`
Update or overwrite an existing rate entry. Key = `id` (u32). Functions identically to `POST`.

**Plaintext payload (before encryption):**
```json
{ "id": 1, "price": 15.0, "time": 3600 }
```

- **200** — Rate saved → `{"ok":true,"id":1,"rate":{...}}`
- **400** — Invalid JSON payload → `{"error":"Invalid payload: ..."}`
- **401** — Decryption failed (wrong key) → `{"error":"Decryption failed: ..."}`

---

### `DELETE /admin/rates`
Remove a rate entry. Key = `id` (u32).

**Plaintext payload (before encryption):**
```json
{ "id": 1 }
```

- **200** — Rate deleted → `{"ok":true,"id":1,"deleted":true}`
- **400** — Invalid JSON payload → `{"error":"Invalid delete payload: ..."}`
- **401** — Decryption failed (wrong key) → `{"error":"Decryption failed: ..."}`

---

### `POST /admin/vouchers`
Insert or overwrite a voucher coupon. Key = `code` (string). Inserting the same code again overwrites the price.

**Plaintext payload (before encryption):**
```json
{ "code": "PROMO2024", "time": 3600 }
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

### `GET /admin/qos`
Returns the current quality-of-service (QoS) speed limits configured on the system.

- **200** — QoS config found → `{"download":5.0,"upload":1.0}` (or `0.0` if not set)
- **403** — Request blocked (Admin access denied from LAN)

---

### `POST /admin/qos`
Apply quality-of-service (QoS) speed limits to all clients using `sqm-scripts` (cake qdisc for fairness). The provided Mbps values are automatically converted to `kbps`.

**Plaintext payload (before encryption):**
```json
{ "download": 5.0, "upload": 1.0 }
```

| Field | Type | Description |
|---|---|---|
| `download` | `f64` | Download speed limit per client in Mbps |
| `upload` | `f64` | Upload speed limit per client in Mbps |

- **200** — QoS applied successfully → `{"ok":true}`
- **400** — Invalid JSON payload → `{"error":"Invalid qos payload: ..."}`
- **401** — Decryption failed (wrong key) → `{"error":"Decryption failed: ..."}`
- **500** — Failed to apply QoS (shell error) → `{"error":"Failed to apply QoS: ..."}`

---

### `GET /admin/users`
Returns all users currently stored in the database.
> **Note**: Even though this is a `GET` request, you must still send an AES-256-GCM encrypted payload in the body to authenticate the request. Any valid payload (e.g., `{"action":"list"}`) will work as long as it decrypts successfully.

- **200** — Returns an array of user objects including their MAC addresses
- **401** — Decryption failed (wrong key or missing payload)

---

### `POST /admin/users`
Insert or overwrite a user record. Key = `mac` (string).

**Plaintext payload (before encryption):**
```json
{
  "mac": "aa:bb:cc:dd:ee:ff",
  "name": "John Doe",
  "ip": "192.168.1.10",
  "paused": false,
  "pause_attempts": 0,
  "pause_day": 0,
  "paused_on": 0,
  "expires_on": 1756547416
}
```

- **200** — User saved → `{"ok":true}`
- **400** — Invalid JSON payload → `{"error":"Invalid user payload: ..."}`
- **401** — Decryption failed (wrong key)

---

### `GET /admin/wifi`
Returns the current 2.4GHz and 5GHz WiFi configuration stored on the system.
> **Note**: Even though this is a `GET` request, you must still send an AES-256-GCM encrypted payload in the body to authenticate the request. Any valid payload (e.g., `{"action":"get"}`) will work as long as it decrypts successfully.

- **200** — WiFi config found → `{"ssid_2g":"...","key_2g":"...","disabled_2g":false,"ssid_5g":"...","key_5g":"...","disabled_5g":false,"supports_2g":true,"supports_5g":false}`
- **401** — Decryption failed (wrong key or missing payload)

---

### `POST /admin/wifi`
Update the WiFi configuration for both 2.4GHz and 5GHz access points. This will instantly apply the changes to OpenWrt's UCI system and restart the wireless interfaces.
> **Note**: If `key_2g` or `key_5g` are passed as empty strings `""`, the system will remove the password requirement for that band and set the encryption to `none`!

**Plaintext payload (before encryption):**
```json
{
  "ssid_2g": "MyNetwork-2.4G",
  "key_2g": "SuperSecretPass",
  "disabled_2g": false,
  "ssid_5g": "MyNetwork-5G",
  "key_5g": "",
  "disabled_5g": false
}
```

- **200** — WiFi updated successfully → `{"ok":true}`
- **400** — Invalid JSON payload → `{"error":"Invalid wifi payload: ..."}`
- **401** — Decryption failed (wrong key)

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
| `POST /admin/rates` | `src/routes/admin/rates.rs` |
| `PUT /admin/rates` | `src/routes/admin/rates.rs` |
| `DELETE /admin/rates` | `src/routes/admin/rates.rs` |
| `POST /admin/vouchers` | `src/routes/admin/vouchers.rs` |
| `GET /admin/qos` | `src/routes/admin/qos.rs` |
| `POST /admin/qos` | `src/routes/admin/qos.rs` |
| `GET /admin/users` | `src/routes/admin/users.rs` |
| `POST /admin/users` | `src/routes/admin/users.rs` |
| `GET /admin/wifi` | `src/routes/admin/wifi.rs` |
| `POST /admin/wifi` | `src/routes/admin/wifi.rs` |
| `GET /debug/db` | `src/routes/debug.rs` |
| `GET /dist/*` | `src/network/server.rs` (static file fallback) |
