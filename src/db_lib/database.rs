pub use rocket_db_pools::Database;

#[derive(Database)]
#[database("postgres_db")]
pub struct AccountsDb(rocket_db_pools::diesel::PgPool);

#[derive(Database)]
#[database("postgres_db")]
pub struct RiskManagementDb(rocket_db_pools::diesel::PgPool);

#[derive(Database)]
#[database("postgres_db")]
pub struct PortfolioDb(rocket_db_pools::diesel::PgPool);
