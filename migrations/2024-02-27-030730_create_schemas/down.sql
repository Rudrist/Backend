-- This file should undo anything in `up.sql`
-- Drop foreign key constraints
-- ALTER TABLE inter_account_transfer_requests DROP CONSTRAINT IF EXISTS fk_admin_account_id;
-- ALTER TABLE inter_account_transfer_requests DROP CONSTRAINT IF EXISTS fk_from_account_id;
-- ALTER TABLE inter_account_transfer_requests DROP CONSTRAINT IF EXISTS fk_to_account_id;

-- ALTER TABLE bbgo_start_info DROP CONSTRAINT IF EXISTS fk_bbgo_api_keys_id;
-- ALTER TABLE bbgo_start_info DROP CONSTRAINT IF EXISTS fk_bbgo_pair_id;
-- ALTER TABLE bbgo_start_info DROP CONSTRAINT IF EXISTS fk_bbgo_strategy_id;
-- ALTER TABLE bbgo_start_info DROP CONSTRAINT IF EXISTS fk_bbgo_risk_id;
-- ALTER TABLE bbgo_start_info DROP CONSTRAINT IF EXISTS fk_bbgo_logs_id;
-- ALTER TABLE bbgo_start_info DROP CONSTRAINT IF EXISTS fk_bbgo_ma_id;
-- ALTER TABLE bbgo_start_info DROP CONSTRAINT IF EXISTS fk_bbgo_rs_id;

ALTER TABLE sessions DROP CONSTRAINT IF EXISTS fk_sessions_user_id;

-- Drop tables in reverse order of creation to respect foreign key relationships
-- DROP TABLE IF EXISTS inter_account_transfer_requests;
DROP TABLE IF EXISTS risk_management;
DROP TABLE IF EXISTS intra_account_transfer_requests;
DROP TABLE IF EXISTS portfolio_balance;
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS quotations;
DROP TABLE IF EXISTS positions;
DROP TABLE IF EXISTS portfolios;
DROP TABLE IF EXISTS accounts;
DROP TABLE IF EXISTS trading_pairs;
DROP TABLE IF EXISTS currencies;
-- DROP TABLE IF EXISTS bbgo_start_info;
-- DROP TABLE IF EXISTS rs_indicator;
-- DROP TABLE IF EXISTS moving_average_indicator;
-- DROP TABLE IF EXISTS logging_monitoring_settings;
DROP TABLE IF EXISTS trading_strategy;
DROP TABLE IF EXISTS exchange_api_keys;
DROP TABLE IF EXISTS sessions;