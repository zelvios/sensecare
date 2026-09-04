CREATE TABLE devices
(
    id               uuid PRIMARY KEY      DEFAULT gen_random_uuid(),
    room_id          uuid REFERENCES rooms (id) ON DELETE RESTRICT,
    key_hash         varchar(255) NOT NULL,
    label            varchar(64),
    firmware_version varchar(32),
    is_active        boolean      NOT NULL DEFAULT true,
    last_seen_at     timestamptz,
    created_at       timestamptz  NOT NULL DEFAULT now(),
    updated_at       timestamptz  NOT NULL DEFAULT now()
);

CREATE INDEX idx_devices_room ON devices (room_id);
CREATE UNIQUE INDEX uq_devices_one_active_per_room ON devices (room_id) WHERE is_active AND room_id IS NOT NULL;
SELECT diesel_manage_updated_at('devices');
