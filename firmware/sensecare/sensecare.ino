#include <Arduino.h>
#include <WiFi.h>
#include <HTTPClient.h>
#include "config.h"
#ifdef USE_TLS
#include <WiFiClientSecure.h>
#endif
#include <Wire.h>
#include <LiquidCrystal_I2C.h>
#include <time.h>
#include "DHT.h"

// Wiring
//   DHT11   VCC 3V3, DATA GPIO 14, GND GND, 10k between VCC and DATA
//   Button  one leg GPIO 27, other leg GND
//   LCD     default I2C pins, SDA 21 and SCL 22
constexpr uint8_t DHT_PIN = 14;
constexpr uint8_t DHT_MODEL = DHT11;
constexpr uint8_t BUTTON_PIN = 27;

constexpr uint8_t LCD_ADDRESS = 0x27;
constexpr uint8_t LCD_COLUMNS = 16;
constexpr uint8_t LCD_ROWS = 2;

constexpr char FIRMWARE_VERSION[] = "1.0.0";

// The DHT11 samples once per second at best. The display updates from this.
constexpr unsigned long READ_INTERVAL_MS = 2000;

// K8: the API must receive a reading at least every 120 s. Send every 30 s for margin.
constexpr unsigned long REPORT_INTERVAL_MS = 30000;

constexpr unsigned long SENSOR_WARMUP_MS = 2000;
constexpr unsigned long DEBOUNCE_MS = 50;
constexpr unsigned long PRESS_COOLDOWN_MS = 3000;
constexpr unsigned long CONFIRMATION_MS = 5000;
constexpr unsigned long WIFI_RETRY_MS = 10000;
constexpr uint16_t HTTP_TIMEOUT_MS = 5000;

DHT dht(DHT_PIN, DHT_MODEL);
LiquidCrystal_I2C lcd(LCD_ADDRESS, LCD_COLUMNS, LCD_ROWS);

unsigned long lastReadAt = 0;
unsigned long lastReportAt = 0;
unsigned long lastWifiAttemptAt = 0;
unsigned long confirmationUntil = 0;

int lastButtonState = HIGH;
unsigned long lastButtonChangeAt = 0;
unsigned long lastPressAt = 0;

float lastTemperature = NAN;
float lastHumidity = NAN;
bool lastReportOk = false;
bool wifiWasConnected = false;

void writeLine(uint8_t row, const String& text) {
  String padded = text;
  while (padded.length() < LCD_COLUMNS) padded += ' ';
  lcd.setCursor(0, row);
  lcd.print(padded);
}

void refreshDisplay() {
  if (millis() < confirmationUntil) {
    writeLine(0, "Tilkald sendt");
    writeLine(1, "Personale kommer");
    return;
  }

  if (isnan(lastTemperature) || isnan(lastHumidity)) {
    writeLine(0, "Sensorfejl");
    writeLine(1, "Tjek ledninger");
    return;
  }

  // A marker in the corner tells the room whether readings are reaching the API.
  const char* link = WiFi.status() == WL_CONNECTED ? (lastReportOk ? "*" : "?") : "!";
  writeLine(0, "Temp:  " + String(lastTemperature, 1) + " C   " + link);
  writeLine(1, "Fugt:  " + String(lastHumidity, 1) + " %");
}

/** Keeps WiFi up. Never blocks longer than one attempt, so the button stays responsive. */
void maintainWifi() {
  const wl_status_t status = WiFi.status();

  if (status == WL_CONNECTED) {
    if (!wifiWasConnected) {
      wifiWasConnected = true;
      Serial.printf("wifi: connected, ip %s\n", WiFi.localIP().toString().c_str());
    }
    return;
  }

  wifiWasConnected = false;

  // An attempt is already running, let it finish before starting another.
  if (status == WL_IDLE_STATUS || status == WL_SCAN_COMPLETED) return;
  if (millis() - lastWifiAttemptAt < WIFI_RETRY_MS) return;
  lastWifiAttemptAt = millis();

  Serial.printf("wifi: connecting to %s\n", WIFI_SSID);
  WiFi.mode(WIFI_STA);
  WiFi.disconnect();
  WiFi.begin(WIFI_SSID, WIFI_PASSWORD);
}

/** Current time in RFC 3339, or an empty string until NTP has synced. */
String nowIso() {
  time_t now = time(nullptr);
  if (now < 1700000000) return "";
  char buf[25];
  strftime(buf, sizeof(buf), "%Y-%m-%dT%H:%M:%SZ", gmtime(&now));
  return String(buf);
}

/** POSTs a JSON body with the device headers. Returns the HTTP status, or 0 on a transport error. */
int postJson(const char* path, const String& body) {
  if (WiFi.status() != WL_CONNECTED) return 0;

  const String url = String(API_BASE_URL) + "/api/v1" + path;
  HTTPClient http;
  http.setTimeout(HTTP_TIMEOUT_MS);

#ifdef USE_TLS
  WiFiClientSecure client;
  client.setCACert(API_ROOT_CA);
  http.begin(client, url);
#else
  http.begin(url);
#endif

  http.addHeader("Content-Type", "application/json");
  http.addHeader("X-Device-Id", DEVICE_ID);
  http.addHeader("X-Device-Key", DEVICE_KEY);

  const int status = http.POST(body);
  if (status <= 0) {
    Serial.printf("http: %s failed, %s\n", path, http.errorToString(status).c_str());
  } else if (status >= 400) {
    Serial.printf("http: %s -> %d %s\n", path, status, http.getString().c_str());
  } else {
    Serial.printf("http: %s -> %d\n", path, status);
  }
  http.end();
  return status;
}

void reportMeasurement() {
  if (isnan(lastTemperature) || isnan(lastHumidity)) return;

  String body = "{\"temperature_c\":" + String(lastTemperature, 1) +
                ",\"humidity_pct\":" + String(lastHumidity, 1);
  const String at = nowIso();
  if (at.length()) body += ",\"measured_at\":\"" + at + "\"";
  body += "}";

  const int status = postJson("/devices/measurements", body);
  lastReportOk = status == 201;
}

void reportPress() {
  const String at = nowIso();
  const String body = at.length() ? "{\"pressed_at\":\"" + at + "\"}" : "{}";
  const int status = postJson("/devices/service-calls", body);
  lastReportOk = status == 200 || status == 201;
}

bool buttonPressed() {
  const int state = digitalRead(BUTTON_PIN);
  if (state != lastButtonState) {
    lastButtonChangeAt = millis();
    lastButtonState = state;
  }
  const bool settled = millis() - lastButtonChangeAt > DEBOUNCE_MS;
  const bool down = state == LOW;
  const bool ready = millis() - lastPressAt > PRESS_COOLDOWN_MS;
  if (settled && down && ready) {
    lastPressAt = millis();
    return true;
  }
  return false;
}

void readSensor() {
  lastHumidity = dht.readHumidity();
  lastTemperature = dht.readTemperature();
  if (isnan(lastHumidity) || isnan(lastTemperature)) {
    Serial.println("dht: read failed");
    return;
  }
  Serial.printf("dht: %.1f C, %.1f %%\n", lastTemperature, lastHumidity);
}

void setup() {
  Serial.begin(115200);
  Serial.printf("sensecare firmware %s\n", FIRMWARE_VERSION);
  pinMode(BUTTON_PIN, INPUT_PULLUP);

  lcd.init();
  lcd.backlight();
  writeLine(0, "SenseCare");
  writeLine(1, "Starter...");
  WiFi.onEvent(
    [](WiFiEvent_t, WiFiEventInfo_t info) {
      Serial.printf("wifi: disconnected, reason %d\n", info.wifi_sta_disconnected.reason);
    },
    ARDUINO_EVENT_WIFI_STA_DISCONNECTED);

  maintainWifi();
  // UTC from NTP, so measured_at and pressed_at are comparable with the server clock.
  configTime(0, 0, "pool.ntp.org", "time.google.com");

#ifdef USE_TLS
  Serial.println("api: https");
#else
  Serial.println("api: http, development only");
#endif

  dht.begin();
  delay(SENSOR_WARMUP_MS);
}

void loop() {
  maintainWifi();

  if (buttonPressed()) {
    Serial.println("button: pressed");
    confirmationUntil = millis() + CONFIRMATION_MS;
    refreshDisplay();
    reportPress();
  }

  if (millis() - lastReadAt >= READ_INTERVAL_MS) {
    lastReadAt = millis();
    readSensor();
    refreshDisplay();
  }

  if (millis() - lastReportAt >= REPORT_INTERVAL_MS) {
    lastReportAt = millis();
    reportMeasurement();
    refreshDisplay();
  }
}