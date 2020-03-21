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
