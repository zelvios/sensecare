CREATE TABLE climate_thresholds
(
    id              uuid PRIMARY KEY       DEFAULT gen_random_uuid(),
    room_id         uuid REFERENCES rooms (id) ON DELETE CASCADE,
    temperature_min numeric(4, 1) NOT NULL,
    temperature_max numeric(4, 1) NOT NULL,
    humidity_min    numeric(4, 1) NOT NULL,
    humidity_max    numeric(4, 1) NOT NULL,
    created_at      timestamptz   NOT NULL DEFAULT now(),
    updated_at      timestamptz   NOT NULL DEFAULT now(),
    CONSTRAINT climate_thresholds_temperature_order CHECK (temperature_min < temperature_max),
    CONSTRAINT climate_thresholds_humidity_order CHECK (humidity_min < humidity_max),
    CONSTRAINT climate_thresholds_humidity_range CHECK (humidity_min >= 0 AND humidity_max <= 100)
);

CREATE UNIQUE INDEX uq_climate_thresholds_room ON climate_thresholds (room_id) WHERE room_id IS NOT NULL;
CREATE UNIQUE INDEX uq_climate_thresholds_global ON climate_thresholds ((true))  WHERE room_id IS NULL;
SELECT diesel_manage_updated_at('climate_thresholds');

INSERT INTO climate_thresholds (room_id, temperature_min, temperature_max, humidity_min, humidity_max)
VALUES (NULL, 19.0, 26.0, 30.0, 60.0);