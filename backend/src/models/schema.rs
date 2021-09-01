table! {
    assets (id) {
        id -> Integer,
        title -> Text,
        description -> Text,
    }
}

table! {
    comments (id) {
        id -> Integer,
        ticket_id -> Integer,
        time -> Timestamp,
        content -> Text,
    }
}

table! {
    tickets (id) {
        id -> Integer,
        asset_id -> Integer,
        title -> Text,
        description -> Text,
        time -> Timestamp,
        is_closed -> Bool,
    }
}

joinable!(comments -> tickets (ticket_id));
joinable!(tickets -> assets (asset_id));

allow_tables_to_appear_in_same_query!(
    assets,
    comments,
    tickets,
);
