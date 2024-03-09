// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Uuid,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 100]
        password -> Varchar,
        time_stamp -> Nullable<Timestamp>,
        account_type -> Nullable<Int4>,
    }
}

diesel::table! {
    currencies (id) {
        id -> Int4,
        #[max_length = 10]
        code -> Varchar,
        #[max_length = 50]
        name -> Varchar,
    }
}

diesel::table! {
    exchange_api_keys (id) {
        id -> Uuid,
        #[max_length = 255]
        api_key -> Varchar,
        #[max_length = 255]
        secret_key -> Varchar,
    }
}

diesel::table! {
    intra_account_transfer_requests (id) {
        id -> Uuid,
        admin_account_id -> Uuid,
        trader_account_id -> Uuid,
        position_id -> Uuid,
        from_portfolio_id -> Uuid,
        to_portfolio_id -> Uuid,
        price -> Int8,
        quantity -> Int8,
        fee -> Int8,
        is_approved -> Bool,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    orders (id) {
        id -> Uuid,
        time_stamp -> Timestamp,
        state -> Int4,
        buyin -> Bool,
        trading_pair_id -> Int4,
        quotation_id -> Uuid,
        price -> Int8,
        qty -> Int8,
    }
}

diesel::table! {
    portfolio_balance (id) {
        id -> Uuid,
        portfolio_id -> Uuid,
        quantity -> Int8,
        currency_id -> Int4,
    }
}

diesel::table! {
    portfolios (id) {
        id -> Uuid,
        time_stamp -> Timestamp,
        trader_account_id -> Uuid,
        portfolio_type -> Int4,
    }
}

diesel::table! {
    positions (id) {
        id -> Uuid,
        trading_pair_id -> Nullable<Int4>,
        portfolio_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    quotations (id) {
        id -> Uuid,
        time_stamp -> Timestamp,
        base_currency_id -> Int4,
        position_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    risk_management (id) {
        id -> Uuid,
        #[max_length = 30]
        risk_type -> Varchar,
        valid -> Bool,
        pnl -> Int8,
        position -> Int4,
        portfolio_id -> Uuid,
    }
}

diesel::table! {
    sessions (session_token) {
        session_token -> Bytea,
        user_id -> Uuid,
    }
}

diesel::table! {
    trading_pairs (id) {
        id -> Int4,
        base_currency_id -> Int4,
        quote_currency_id -> Int4,
    }
}

diesel::joinable!(intra_account_transfer_requests -> positions (position_id));
diesel::joinable!(orders -> quotations (quotation_id));
diesel::joinable!(portfolio_balance -> currencies (currency_id));
diesel::joinable!(portfolio_balance -> portfolios (portfolio_id));
diesel::joinable!(portfolios -> accounts (trader_account_id));
diesel::joinable!(positions -> portfolios (portfolio_id));
diesel::joinable!(positions -> trading_pairs (trading_pair_id));
diesel::joinable!(quotations -> currencies (base_currency_id));
diesel::joinable!(quotations -> positions (position_id));
diesel::joinable!(risk_management -> portfolios (portfolio_id));
diesel::joinable!(risk_management -> trading_pairs (position));
diesel::joinable!(sessions -> accounts (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    currencies,
    exchange_api_keys,
    intra_account_transfer_requests,
    orders,
    portfolio_balance,
    portfolios,
    positions,
    quotations,
    risk_management,
    sessions,
    trading_pairs,
);
