// @generated automatically by Diesel CLI.

diesel::table! {
    alarms (id) {
        id -> Uuid,
        room_id -> Uuid,
        measurement_id -> Int8,
        #[max_length = 32]
        kind -> Varchar,
        measured_value -> Numeric,
        threshold_value -> Numeric,
        raised_at -> Timestamptz,
        acknowledged_at -> Nullable<Timestamptz>,
        acknowledged_by -> Nullable<Uuid>,
        resolved_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    audit_log (id) {
        id -> Int8,
        actor_id -> Nullable<Uuid>,
        #[max_length = 64]
        action -> Varchar,
        #[max_length = 32]
        entity_type -> Varchar,
        #[max_length = 64]
        entity_id -> Varchar,
        details -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    climate_thresholds (id) {
        id -> Uuid,
        room_id -> Nullable<Uuid>,
        temperature_min -> Numeric,
        temperature_max -> Numeric,
        humidity_min -> Numeric,
        humidity_max -> Numeric,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

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
    service_calls (id) {
        id -> Uuid,
        room_id -> Uuid,
        device_id -> Uuid,
        #[max_length = 16]
        status -> Varchar,
        created_at -> Timestamptz,
        pressed_at -> Nullable<Timestamptz>,
        acknowledged_at -> Nullable<Timestamptz>,
        acknowledged_by -> Nullable<Uuid>,
        closed_at -> Nullable<Timestamptz>,
        closed_by -> Nullable<Uuid>,
        #[max_length = 500]
        note -> Nullable<Varchar>,
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

diesel::joinable!(alarms -> measurements (measurement_id));
diesel::joinable!(alarms -> rooms (room_id));
diesel::joinable!(alarms -> users (acknowledged_by));
diesel::joinable!(audit_log -> users (actor_id));
diesel::joinable!(climate_thresholds -> rooms (room_id));
diesel::joinable!(devices -> rooms (room_id));
diesel::joinable!(measurements -> devices (device_id));
diesel::joinable!(measurements -> rooms (room_id));
diesel::joinable!(service_calls -> devices (device_id));
diesel::joinable!(service_calls -> rooms (room_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(stays -> rooms (room_id));
diesel::joinable!(stays -> users (user_id));
diesel::joinable!(users -> roles (role_id));

diesel::allow_tables_to_appear_in_same_query!(
    alarms,
    audit_log,
    climate_thresholds,
    devices,
    measurements,
    roles,
    rooms,
    service_calls,
    sessions,
    stays,
    users,
);
