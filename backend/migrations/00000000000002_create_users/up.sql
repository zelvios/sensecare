CREATE TABLE users
(
    id            uuid PRIMARY KEY      DEFAULT gen_random_uuid(),
    username      varchar(64)  NOT NULL UNIQUE,
    display_name  varchar(128) NOT NULL,
    password_hash varchar(255) NOT NULL,
    role_id       smallint     NOT NULL REFERENCES roles (id) ON DELETE RESTRICT,
    is_active     boolean      NOT NULL DEFAULT true,
    last_login_at timestamptz,
    created_at    timestamptz  NOT NULL DEFAULT now(),
    updated_at    timestamptz  NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_role ON users (role_id);
SELECT diesel_manage_updated_at('users');
