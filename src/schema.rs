table! {
    refresh_tokens (id) {
        id -> Int4,
        expiry -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
        email -> Text,
        password -> Text,
        is_deleted -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    refresh_tokens,
    users,
);
