use crate::error::PineError;
use log::info;
use mysql::prelude::Queryable;
use mysql::{OptsBuilder, Pool};

pub trait Connection {
    fn databases(&self) -> Result<Vec<String>, PineError>;
    fn tables(&self, db: &str) -> Result<Vec<String>, PineError>;
    fn show_create(&self, db: &str, table: &str) -> Result<String, PineError>;
}

pub struct LiveConnection {
    pool: Pool,
}

pub struct OfflineConnection;

impl LiveConnection {
    pub fn new(
        user: &str,
        password: &str,
        host: &str,
        port: u16,
    ) -> Result<LiveConnection, PineError> {
        let opts_builder = OptsBuilder::new()
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
        let query_result = self.pool.get_conn().unwrap().exec(r"show databases;", ())?;
        let all_databases: Vec<_> = query_result.into_iter().collect();

        let user_databases = all_databases
            .into_iter()
            .filter(|database| !MYSQL_BUILTIN_DATABASES.contains(&(&*database as &str)))
            .collect();

        info!("Found databases: {:?}", user_databases);
        Ok(user_databases)
    }

    fn tables(&self, db: &str) -> Result<Vec<String>, PineError> {
        let query_result = self
            .pool
            .get_conn()
            .unwrap()
            .exec(format!("show tables from {}", db), ())?;
        let all_tables: Vec<_> = query_result.into_iter().collect();

        info!("Found tables for db '{}': {:?}", db, all_tables);
        Ok(all_tables)
    }

    fn show_create(&self, db: &str, table: &str) -> Result<String, PineError> {
        let query_result = self
            .pool
            .get_conn()
            .unwrap()
            .exec(format!("show create table {}.{}", db, table), ())?;
        let mut all_tables: Vec<_> = query_result
            .into_iter()
            .map(|(_, create_table): (String, String)| create_table)
            .take(1)
            .collect();

        info!("Table create query retrieved for {}.{}", db, table);
        Ok(all_tables.remove(0))
    }
}

impl Connection for OfflineConnection {
    fn databases(&self) -> Result<Vec<String>, PineError> {
        panic!("Cannot call OfflineConnection::database()")
    }

    fn tables(&self, _db: &str) -> Result<Vec<String>, PineError> {
        panic!("Cannot call OfflineConnection::tables()")
    }

    fn show_create(&self, _db: &str, _table: &str) -> Result<String, PineError> {
        panic!("Cannot call OfflineConnection::show_create()")
    }
}
