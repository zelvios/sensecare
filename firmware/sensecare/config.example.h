#pragma once

// Copy to config.h and fill in.

#define WIFI_SSID "your-network"
#define WIFI_PASSWORD "your-password"

// Register a new device in Administration > Enheder. The key is shown once.
#define DEVICE_ID "00000000-0000-0000-0000-000000000000"
#define DEVICE_KEY "0000000000000000000000000000000000000000000000000000000000000000"

// Pick one: development over HTTP, or production over HTTPS.
// Comment out USE_TLS for development.

// #define USE_TLS

#ifdef USE_TLS
#define API_BASE_URL "https://sensecare.jacob-j.com"

#define API_ROOT_CA \
  "-----BEGIN CERTIFICATE-----\n" \
  "...\n" \
  "-----END CERTIFICATE-----\n"
#else
// The PC running docker compose, on the same network as the board.
#define API_BASE_URL "http://192.168.1.10:8080"
#endif