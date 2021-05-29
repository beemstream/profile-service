table! {
    refresh_tokens (id) {
        id -> Int4,
        token -> Varchar,
        expiry -> Timestamp,
        user_id -> Int4,
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

joinable!(refresh_tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(refresh_tokens, users,);
