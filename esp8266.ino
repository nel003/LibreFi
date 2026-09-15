#if defined(ESP8266)
#include <ESP8266WiFi.h>
#include <bearssl/bearssl.h>
#elif defined(ESP32)
#include <WiFi.h>
#include <esp_random.h>
#include <mbedtls/base64.h>
#include <mbedtls/gcm.h>
#else
#error "This sketch supports ESP8266 or ESP32 boards."
#endif

#include <ArduinoJson.h>
#include <Ticker.h>
#include <WebSocketsClient.h>

// ==================== CONFIGURATION ====================
const char *WIFI_SSID = "LibreFi2.4G";
const char *WIFI_PASS = "";

// Router WebSocket Server (Default OpenFi gateway)
const char *WS_HOST = "10.0.0.1";
const uint16_t WS_PORT = 80;
const char *WS_PATH = "/ws";

// Secret key matching ADMIN_SECRET_KEY in src/routes/admin.rs
const char *ADMIN_SECRET_KEY = "9707e9a85d07e6e792417d6a3e68b71f";
// =======================================================

// Hardware Pins
#define COIN_PIN 4
#define SLOT_PIN 14
#define LED_PIN 15

// Timing Configurations
const unsigned long DEBOUNCE_TIME_MS = 50;
const unsigned long CREDIT_DELAY_MS = 200;
const unsigned long BLINK_INTERVAL_MS = 100;
const unsigned long LED_BLINK_INTERVAL = 1000;

Ticker creditTicker;
WebSocketsClient webSocket;

volatile unsigned long lastPulseTime = 0;
unsigned long lastBlinkTime = 0;
unsigned long lastLEDToggleTime = 0;
unsigned long lastHeartbeatTime =
    0; // Heartbeat keeps server's ws.read() from blocking
volatile int pulseCount = 0;

bool sessionActive = false;
int sessionTimeLeft = 0;
unsigned long lastTimerTick = 0;

bool hasNewCredit = false;
bool isReady = false;
bool hasPulse = false;
bool isNotified = false;

// NEW: Variable to track the LED state in software
bool ledState = false;

// ==================== BASE64 HELPER ====================
#if defined(ESP8266)
static const char B64_CHARS[] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

String base64Encode(const uint8_t *data, size_t len) {
  String out = "";
  out.reserve(((len + 2) / 3) * 4);
  for (size_t i = 0; i < len; i += 3) {
    uint32_t octet_a = data[i];
    uint32_t octet_b = (i + 1 < len) ? data[i + 1] : 0;
    uint32_t octet_c = (i + 2 < len) ? data[i + 2] : 0;
    uint32_t triple = (octet_a << 16) + (octet_b << 8) + octet_c;

    out += B64_CHARS[(triple >> 18) & 0x3F];
    out += B64_CHARS[(triple >> 12) & 0x3F];
    out += (i + 1 < len) ? B64_CHARS[(triple >> 6) & 0x3F] : '=';
    out += (i + 2 < len) ? B64_CHARS[triple & 0x3F] : '=';
  }
  return out;
}
#endif

// ==================== ENCRYPTION ====================
// Encrypts payload with AES-256-GCM using [12-byte IV] + [Ciphertext] +
// [16-byte Tag]
String encryptPayload(const char *secretKey, const char *jsonPayload) {
  uint8_t key[32] = {0};
  size_t keyLen = strlen(secretKey);
  if (keyLen > 32)
    keyLen = 32;
  memcpy(key, secretKey, keyLen);

  size_t plainLen = strlen(jsonPayload);

#if defined(ESP32)
  uint8_t iv[12];
  for (int i = 0; i < 12; i++) {
    iv[i] = (uint8_t)esp_random();
  }

  uint8_t ciphertext[plainLen];
  uint8_t tag[16];

  mbedtls_gcm_context gcm;
  mbedtls_gcm_init(&gcm);
  if (mbedtls_gcm_setkey(&gcm, MBEDTLS_CIPHER_ID_AES, key, 256) != 0) {
    mbedtls_gcm_free(&gcm);
    return "";
  }

  if (mbedtls_gcm_crypt_and_tag(&gcm, MBEDTLS_GCM_ENCRYPT, plainLen, iv, 12,
                                NULL, 0, (const unsigned char *)jsonPayload,
                                ciphertext, 16, tag) != 0) {
    mbedtls_gcm_free(&gcm);
    return "";
  }
  mbedtls_gcm_free(&gcm);

  size_t combinedLen = 12 + plainLen + 16;
  uint8_t combined[combinedLen];
  memcpy(combined, iv, 12);
  memcpy(combined + 12, ciphertext, plainLen);
  memcpy(combined + 12 + plainLen, tag, 16);

  size_t dlen = 0;
  mbedtls_base64_encode(NULL, 0, &dlen, combined, combinedLen);
  unsigned char b64[dlen + 1];
  if (mbedtls_base64_encode(b64, sizeof(b64), &dlen, combined, combinedLen) !=
      0) {
    return "";
  }
  b64[dlen] = '\0';
  return String((char *)b64);

#elif defined(ESP8266)
  uint8_t iv[12];
  for (int i = 0; i < 12; i++) {
    iv[i] = (uint8_t)os_random();
  }

  uint8_t ciphertext[plainLen];
  memcpy(ciphertext, jsonPayload, plainLen);
  uint8_t tag[16];

  br_gcm_context gc;
  br_aes_ct_ctr_keys bc;
  br_aes_ct_ctr_init(&bc, key, 32);
  br_gcm_init(&gc, &bc.vtable, br_ghash_ctmul32);
  br_gcm_reset(&gc, iv, 12);
  br_gcm_run(&gc, 1, ciphertext, plainLen);
  br_gcm_get_tag(&gc, tag);

  size_t combinedLen = 12 + plainLen + 16;
  uint8_t combined[combinedLen];
  memcpy(combined, iv, 12);
  memcpy(combined + 12, ciphertext, plainLen);
  memcpy(combined + 12 + plainLen, tag, 16);

  return base64Encode(combined, combinedLen);
#else
  return "";
#endif
}

void registerCoinslot() {
  String b64 = encryptPayload(ADMIN_SECRET_KEY, "{\"type\":\"COINSLOT\"}");
  if (b64.length() == 0) {
    // Verified fallback for default ADMIN_SECRET_KEY ("arns_super_pogi_4ever")
    b64 =
        "MjNT6dyGSMyDQ4OCK/AthH23luO5w+dUach2X6v63AT1VQDOa0azGJLt/GnbP5nc6vc=";
    Serial.println("[WS] Using pre-computed registration token");
  }
  String regMsg = "{\"payload\":\"" + b64 + "\"}";
  webSocket.sendTXT(regMsg);
  Serial.println("[WS] Sent registration: " + regMsg);
}

// Encrypt json and send as {"payload":"<base64>"} so the server
// can verify every message originates from an authenticated device.
void sendEncrypted(const String &json) {
  String b64 = encryptPayload(ADMIN_SECRET_KEY, json.c_str());
  if (b64.length() == 0) {
    Serial.println("[ENC] Encryption failed — message dropped: " + json);
    return;
  }
  String msg = "{\"payload\":\"" + b64 + "\"}";
  webSocket.sendTXT(msg);
  Serial.println("[WS] Sent (enc): " + json);
}

// ==================== INTERRUPT & TIMERS ====================
void IRAM_ATTR onCoinInterrupt() {
  static unsigned long lastInterruptTime = 0;
  unsigned long currentTime = millis();

  hasPulse = true;

  if ((currentTime - lastInterruptTime) >= DEBOUNCE_TIME_MS) {
    pulseCount++;
    creditTicker.detach();
    creditTicker.once_ms(CREDIT_DELAY_MS, sendCredit);
    lastInterruptTime = currentTime;
  }
}

void sendCredit() { hasNewCredit = true; }

// ==================== LED & BUZZER ====================
void handleLED() {
  unsigned long currentMillis = millis();

  if (!isReady) {
    // Blinking while waiting for connection/registration
    if (currentMillis - lastLEDToggleTime >= LED_BLINK_INTERVAL) {
      ledState = !ledState;
      digitalWrite(LED_PIN, ledState);
      lastLEDToggleTime = currentMillis;
    }
  } else {
    if (sessionActive) {
      if (millis() - lastBlinkTime >= BLINK_INTERVAL_MS) {
        ledState = !ledState;
        digitalWrite(LED_PIN, ledState);
        lastBlinkTime = currentMillis;
      }
    } else {
      ledState = true;
      digitalWrite(LED_PIN, HIGH);
    }
  }
}

void processData(const char *jsonStr) {
  JsonDocument doc;
  DeserializationError error = deserializeJson(doc, jsonStr);

  if (error) {
    Serial.print("[JSON] Parse error: ");
    Serial.println(error.c_str());
    return;
  }

  if (doc["type"] == "status" && doc["value"] == "ok") {
    isReady = true;
    Serial.println("[STATUS] Registration confirmed — coinslot ready!");
  }

  if (doc["type"] == "ACK") {
    sessionActive = true;
    sessionTimeLeft = 30;
    lastTimerTick = millis();
    digitalWrite(SLOT_PIN, HIGH);
    sendEncrypted("{\"type\":\"ACK_SUCCESS\"}");
    Serial.println("[ACK] Ping from server — started 30s session, slot opened "
                   "(encrypted)");
  }

  if (doc["type"] == "cmd" && doc["value"] == "open") {
    sendEncrypted("{\"type\":\"res\",\"value\":\"open\"}");
    sessionActive = true; // Added this so handleLED() knows the slot is open
    digitalWrite(SLOT_PIN, HIGH);
    Serial.println("[CMD] Coin slot opened");
  }

  if (doc["type"] == "cmd" && doc["value"] == "close") {
    sendEncrypted("{\"type\":\"res\",\"value\":\"close\"}");
    digitalWrite(SLOT_PIN, LOW);
    sessionActive = false;
    sessionTimeLeft = 0;
    if (pulseCount > 0) {
      hasNewCredit = true;
    }

    Serial.println("[CMD] Coin slot closed from user DONE command");
  }
}

void webSocketEvent(WStype_t type, uint8_t *payload, size_t length) {
  switch (type) {
  case WStype_DISCONNECTED:
    Serial.println("[WS] Disconnected from router WebSocket server.");
    isReady = false;
    break;

  case WStype_CONNECTED:
    Serial.printf("[WS] Connected to: %s\n", payload);
    isReady = false;
    registerCoinslot();
    break;

  case WStype_TEXT:
    Serial.printf("[WS] Received: %s\n", payload);
    processData((char *)payload);
    break;

  case WStype_BIN:
    Serial.printf("[WS] Received binary length: %u\n", length);
    break;

  case WStype_ERROR:
    Serial.println("[WS] Error occurred");
    break;

  default:
    break;
  }
}

// ==================== SETUP & LOOP ====================
void setup() {
  Serial.begin(115200);
  delay(100);
  Serial.println("\n--- LibreFi Coinslot Starting ---");

  pinMode(COIN_PIN, INPUT_PULLUP);
  pinMode(LED_PIN, OUTPUT);
  pinMode(SLOT_PIN, OUTPUT);

  digitalWrite(SLOT_PIN, LOW);
  digitalWrite(LED_PIN, LOW);

  // Connect to WiFi
  WiFi.mode(WIFI_STA);
#if defined(ESP8266)
  WiFi.setOutputPower(20.5);
#elif defined(ESP32)
  WiFi.setTxPower(WIFI_POWER_8_5dBm);
#else
#endif

  WiFi.begin(WIFI_SSID, WIFI_PASS);
  Serial.print("[WiFi] Connecting to ");
  Serial.print(WIFI_SSID);

  // FIXED: Using ledState boolean here as well
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
    ledState = !ledState;
    digitalWrite(LED_PIN, ledState);
  }

  Serial.println("\n[WiFi] Connected! IP: " + WiFi.localIP().toString());

  attachInterrupt(digitalPinToInterrupt(COIN_PIN), onCoinInterrupt, FALLING);

  // Connect WebSocket to router
  Serial.printf("[WS] Connecting to ws://%s:%d%s\n", WS_HOST, WS_PORT, WS_PATH);
  webSocket.begin(WS_HOST, WS_PORT, WS_PATH);
  webSocket.onEvent(webSocketEvent);
  webSocket.setReconnectInterval(3000);
}

void loop() {
  webSocket.loop();

  if (webSocket.isConnected() && isReady) {
    // Heartbeat ping every 2000ms — keeps the server's blocking ws.read()
    // cycling so it can check the relay channel between reads.
    if (millis() - lastHeartbeatTime >= 2000) {
      sendEncrypted("{\"type\":\"ping\"}");
      lastHeartbeatTime = millis();
    }

    // Hardware Session Timer
    if (sessionActive) {
      if (millis() - lastTimerTick >= 1000) {
        lastTimerTick = millis();
        sessionTimeLeft--;
        String timerMsg =
            "{\"type\":\"timer\",\"value\":" + String(sessionTimeLeft) + "}";
        sendEncrypted(timerMsg);

        if (sessionTimeLeft <= 0) {
          sessionActive = false;
          digitalWrite(SLOT_PIN, LOW); // Timeout, safely close the slot
          Serial.println("[TIMER] Session timeout, slot closed.");
        }
      }
    }

    // Notify on initial pulse detection
    if (hasPulse && !isNotified) {
      sendEncrypted("{\"type\":\"notify\",\"value\":\"true\"}");
      isNotified = true;
      if (sessionActive) {
        sessionTimeLeft = 30; // Reset timer on coin drop
      }
    }

    // Send accumulated coin pulses (encrypted — prevents spoofing)
    if (hasNewCredit) {
      String coinMsg =
          "{\"type\":\"coin\",\"value\":" + String(pulseCount) + "}";
      sendEncrypted(coinMsg);
      pulseCount = 0;
      hasPulse = false;
      isNotified = false;
      hasNewCredit = false;
    }
  }

  handleLED();
}