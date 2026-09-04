CREATE TABLE alarms
(
    id              uuid PRIMARY KEY       DEFAULT gen_random_uuid(),
    room_id         uuid          NOT NULL REFERENCES rooms (id) ON DELETE RESTRICT,
    measurement_id  bigint        NOT NULL REFERENCES measurements (id) ON DELETE RESTRICT,
    kind            varchar(32)   NOT NULL,
    measured_value  numeric(4, 1) NOT NULL,
    threshold_value numeric(4, 1) NOT NULL,
    raised_at       timestamptz   NOT NULL DEFAULT now(),
    acknowledged_at timestamptz,
    acknowledged_by uuid          REFERENCES users (id) ON DELETE SET NULL,
    resolved_at     timestamptz,
    CONSTRAINT alarms_kind_valid CHECK (kind IN
                                        ('temperature_low', 'temperature_high', 'humidity_low', 'humidity_high')),
    CONSTRAINT alarms_ack_together CHECK ((acknowledged_at IS NULL) = (acknowledged_by IS NULL)),
    CONSTRAINT alarms_resolved_after_raised CHECK (resolved_at IS NULL OR resolved_at >= raised_at)
);

CREATE INDEX idx_alarms_room_time ON alarms (room_id, raised_at DESC);
CREATE INDEX idx_alarms_open ON alarms (raised_at DESC) WHERE resolved_at IS NULL;
