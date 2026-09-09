CREATE TABLE service_calls
(
    id              uuid PRIMARY KEY     DEFAULT gen_random_uuid(),
    room_id         uuid        NOT NULL REFERENCES rooms (id) ON DELETE RESTRICT,
    device_id       uuid        NOT NULL REFERENCES devices (id) ON DELETE RESTRICT,
    status          varchar(16) NOT NULL DEFAULT 'open',
    created_at      timestamptz NOT NULL DEFAULT now(),
    pressed_at      timestamptz, -- device clock, if it has one
    acknowledged_at timestamptz,
    acknowledged_by uuid        REFERENCES users (id) ON DELETE SET NULL,
    closed_at       timestamptz,
    closed_by       uuid        REFERENCES users (id) ON DELETE SET NULL,
    note            varchar(500),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT service_calls_status_valid CHECK (status IN ('open', 'in_progress', 'closed')),
    CONSTRAINT service_calls_ack_together CHECK ((acknowledged_at IS NULL) = (acknowledged_by IS NULL)),
    CONSTRAINT service_calls_close_together CHECK ((closed_at IS NULL) = (closed_by IS NULL)),
    CONSTRAINT service_calls_status_matches_timestamps CHECK (
        (status = 'open' AND acknowledged_at IS NULL AND closed_at IS NULL) OR
        (status = 'in_progress' AND acknowledged_at IS NOT NULL AND closed_at IS NULL) OR
        (status = 'closed' AND closed_at IS NOT NULL) )
);

CREATE INDEX idx_service_calls_room_time ON service_calls (room_id, created_at DESC);
CREATE INDEX idx_service_calls_open ON service_calls (created_at DESC) WHERE status <> 'closed';
SELECT diesel_manage_updated_at('service_calls');