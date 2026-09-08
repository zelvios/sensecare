CREATE TABLE measurements
(
    id            bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    room_id       uuid          NOT NULL REFERENCES rooms (id) ON DELETE RESTRICT,
    device_id     uuid          NOT NULL REFERENCES devices (id) ON DELETE RESTRICT,
    temperature_c numeric(4, 1) NOT NULL,
    humidity_pct  numeric(4, 1) NOT NULL,
    measured_at   timestamptz   NOT NULL,
    received_at   timestamptz   NOT NULL,
    CONSTRAINT measurements_temperature_plausible CHECK (temperature_c BETWEEN -40 AND 85),
    CONSTRAINT measurements_humidity_range CHECK (humidity_pct BETWEEN 0 AND 100)
);

CREATE INDEX idx_measurements_room_time ON measurements (room_id, measured_at DESC);
CREATE INDEX idx_measurements_device ON measurements (device_id, measured_at DESC);