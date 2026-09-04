CREATE TABLE stays
(
    id             uuid PRIMARY KEY     DEFAULT gen_random_uuid(),
    room_id        uuid        NOT NULL REFERENCES rooms (id) ON DELETE RESTRICT,
    user_id        uuid        NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    checked_in_at  timestamptz NOT NULL DEFAULT now(),
    checked_out_at timestamptz,
    CONSTRAINT stays_checkout_after_checkin CHECK (checked_out_at IS NULL OR checked_out_at > checked_in_at)
);

CREATE INDEX idx_stays_room ON stays (room_id, checked_in_at DESC);
CREATE INDEX idx_stays_user ON stays (user_id, checked_in_at DESC);
CREATE UNIQUE INDEX uq_stays_one_open_per_room ON stays (room_id) WHERE checked_out_at IS NULL;
CREATE UNIQUE INDEX uq_stays_one_open_per_user ON stays (user_id) WHERE checked_out_at IS NULL;
