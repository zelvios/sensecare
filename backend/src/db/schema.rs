// @generated automatically by Diesel CLI.

diesel::table! {
    devices (id) {
        id -> Uuid,
        room_id -> Nullable<Uuid>,
        #[max_length = 255]
        key_hash -> Varchar,
        #[max_length = 64]
        label -> Nullable<Varchar>,
        #[max_length = 32]
        firmware_version -> Nullable<Varchar>,
        is_active -> Bool,
        last_seen_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    measurements (id) {
        id -> Int8,
        room_id -> Uuid,
        device_id -> Uuid,
        temperature_c -> Numeric,
        humidity_pct -> Numeric,
        measured_at -> Timestamptz,
        received_at -> Timestamptz,
    }
}

diesel::table! {
    roles (id) {
        id -> Int2,
        #[max_length = 32]
        name -> Varchar,
        #[max_length = 255]
        description -> Varchar,
    }
}

diesel::table! {
    rooms (id) {
        id -> Uuid,
        #[max_length = 16]
        room_number -> Varchar,
        #[max_length = 64]
        name -> Nullable<Varchar>,
        floor -> Nullable<Int2>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        user_agent -> Nullable<Varchar>,
        created_at -> Timestamptz,
        last_used_at -> Timestamptz,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    stays (id) {
        id -> Uuid,
        room_id -> Uuid,
        user_id -> Uuid,
        checked_in_at -> Timestamptz,
        checked_out_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 64]
        username -> Varchar,
        #[max_length = 128]
        display_name -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        role_id -> Int2,
        is_active -> Bool,
        last_login_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(devices -> rooms (room_id));
diesel::joinable!(measurements -> devices (device_id));
diesel::joinable!(measurements -> rooms (room_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(stays -> rooms (room_id));
diesel::joinable!(stays -> users (user_id));
diesel::joinable!(users -> roles (role_id));

diesel::allow_tables_to_appear_in_same_query!(
    devices,
    measurements,
    roles,
    rooms,
    sessions,
    stays,
    users,
);
