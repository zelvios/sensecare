CREATE TABLE rooms
(
    id          uuid PRIMARY KEY     DEFAULT gen_random_uuid(),
    room_number varchar(16) NOT NULL UNIQUE,
    name        varchar(64),
    floor       smallint,
    is_active   boolean     NOT NULL DEFAULT true,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);

SELECT diesel_manage_updated_at('rooms');