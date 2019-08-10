use super::super::structure::{Database, Table};
use super::connection::Connection;
use crate::error::PineError;

pub trait Reflector {
    fn analyze(&self) -> Result<Vec<Database>, PineError>;
}

pub trait TableParser {
    fn parse(&self, create_statement: &str) -> Result<Table, PineError>;
}

pub struct MySqlReflector<T, U> {
    connection: T,
    table_parser: U,
}

impl<T> MySqlReflector<T, MySqlTableParser>
where
    T: Connection,
{
    pub fn for_connection(connection: T) -> MySqlReflector<T, MySqlTableParser> {
        MySqlReflector {
            connection,
            table_parser: MySqlTableParser {},
        }
    }
}

impl<T, U> MySqlReflector<T, U>
where
    T: Connection,
    U: TableParser,
{
    fn analyze_database(&self, db_name: &str) -> Result<Database, PineError> {
        let tables = self.connection.tables(db_name)?;
        let tables = tables
            .iter()
            .map(|t| self.analyze_table(db_name, t))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Database {
            name: db_name.to_string(),
            tables: tables,
        })
    }

    fn analyze_table(&self, db_name: &str, table_name: &str) -> Result<Table, PineError> {
        let create_statement = self.connection.show_create(db_name, table_name)?;

        self.table_parser.parse(&create_statement)
    }
}

impl<T, U> Reflector for MySqlReflector<T, U>
where
    T: Connection,
    U: TableParser,
{
    fn analyze(&self) -> Result<Vec<Database>, PineError> {
        self.connection
            .databases()?
            .iter()
            .map(|s| self.analyze_database(s))
            .collect()
    }
}

pub struct MySqlTableParser;
impl TableParser for MySqlTableParser {
    fn parse(&self, create_statement: &str) -> Result<Table, PineError> {
        Ok(Table::from_sql_string(create_statement)?)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::structure::Column;
    use super::*;

    #[test]
    fn all_databases_are_covered() {
        let connection = MockConnection::new(&[("database 1", &[]), ("database 2", &[])]);
        let reflector = MySqlReflector::new(connection, MockTableParser {});

        let databases = reflector.analyze().unwrap();
        assert_eq!(2, databases.len());
    }

    #[test]
    fn databases_are_parsed() {
        let connection = MockConnection::new(&[("database 1", &["table 1"])]);
        let reflector = MySqlReflector::new(connection, MockTableParser {});

        let database1 = &reflector.analyze().unwrap()[0];
        assert_eq!("database 1", database1.name);
        assert_eq!("table 1", &database1.tables[0].name);
    }

    type TableSpec = (&'static str, &'static [&'static str]);

    struct MockConnection {
        databases: &'static [TableSpec],
    }

    impl MockConnection {
        fn new(databases: &'static [TableSpec]) -> Self {
            MockConnection { databases }
        }
    }

    impl Connection for MockConnection {
        fn databases(&self) -> Result<Vec<String>, PineError> {
            Ok(self.databases.iter().map(|s| s.0.to_string()).collect())
        }

        fn tables(&self, db: &str) -> Result<Vec<String>, PineError> {
            let db_find = self.databases.iter().find(|some_db| some_db.0 == db);

            match db_find {
                Some(&db) => Ok(db.1.iter().map(|s| s.to_string()).collect()),
                None => Err(format!("Can't find db: {}", db).into()),
            }
        }

        fn show_create(&self, db: &str, table: &str) -> Result<String, PineError> {
            let db_find = self.databases.iter().find(|some_db| some_db.0 == db);
            if db_find.is_none() {
                return Err("database not found".into());
            }

            let db_find = db_find.unwrap();

            let table_find = db_find.1.iter().find(|some_table| **some_table == table);

            match table_find {
                Some(table) => Ok(table.to_string().to_string()),
                None => Err("Table not found".into()),
            }
        }
    }

    struct MockTableParser;
    impl TableParser for MockTableParser {
        fn parse(&self, create_statement: &str) -> Result<Table, PineError> {
            Ok(Table {
                name: create_statement.to_string(),
                columns: vec![Column {
                    name: "column name".to_string(),
                }],
                foreign_keys: Vec::new(),
            })
        }
    }

    impl<T, U> MySqlReflector<T, U> {
        fn new(connection: T, table_parser: U) -> Self {
            MySqlReflector {
                connection,
                table_parser,
            }
        }
    }
}
