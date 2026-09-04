CREATE TABLE audit_log
(
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    actor_id    uuid        REFERENCES users (id) ON DELETE SET NULL,
    action      varchar(64) NOT NULL, -- 'user.deactivated', 'room.created', ...
    entity_type varchar(32) NOT NULL, -- 'user', 'room', 'device', ...
    entity_id   varchar(64) NOT NULL, -- uuid or bigint as text
    details     jsonb,                -- before/after or extra context
    created_at  timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_log_entity ON audit_log (entity_type, entity_id, created_at DESC);
CREATE INDEX idx_audit_log_actor ON audit_log (actor_id, created_at DESC);
CREATE INDEX idx_audit_log_time ON audit_log (created_at DESC);