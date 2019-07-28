use crate::error::PineError;
use mysql::{from_row, OptsBuilder, Pool};

pub trait Connection {
    fn databases(&self) -> Result<Vec<String>, PineError>;
    fn tables(&self, db: &str) -> Result<Vec<String>, PineError>;
    fn show_create(&self, db: &str, table: &str) -> Result<String, PineError>;
}

pub struct LiveConnection {
    pool: Pool,
}

impl LiveConnection {
    pub fn new(
        user: &str,
        password: &str,
        host: &str,
        port: u16,
    ) -> Result<LiveConnection, PineError> {
        let mut opts_builder = OptsBuilder::new();
        opts_builder
            .user(Some(user))
            .pass(Some(password))
            .ip_or_hostname(Some(host))
            .tcp_port(port);

        let pool = Pool::new(opts_builder)?;

        Ok(LiveConnection { pool })
    }
}

const MYSQL_BUILTIN_DATABASES: [&str; 3] = ["mysql", "information_schema", "performance_schema"];
impl Connection for LiveConnection {
    fn databases(&self) -> Result<Vec<String>, PineError> {
        let query_result = self.pool.prep_exec(r"show databases;", ())?;
        let all_databases: Vec<_> = query_result.map(|row| from_row(row.unwrap())).collect();

        let user_databases = all_databases
            .into_iter()
            .filter(|database| !MYSQL_BUILTIN_DATABASES.contains(&(&*database as &str)))
            .collect();

        Ok(user_databases)
    }

    fn tables(&self, db: &str) -> Result<Vec<String>, PineError> {
        let query_result = self
            .pool
            .prep_exec(format!("show tables from {}", db), ())?;
        let all_tables: Vec<_> = query_result.map(|row| from_row(row.unwrap())).collect();

        Ok(all_tables)
    }

    fn show_create(&self, db: &str, table: &str) -> Result<String, PineError> {
        let query_result = self
            .pool
            .prep_exec(format!("show create table {}.{}", db, table), ())?;
        let mut all_tables: Vec<String> = query_result
            .map(|row| {
                let row: (String, String) = from_row(row.unwrap());
                row.1
            })
            .take(1)
            .collect();

        Ok(all_tables.remove(0))
    }
}
