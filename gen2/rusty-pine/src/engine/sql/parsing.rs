use crate::analyze::ColumnName;
use crate::engine::sql::querying::TableDescription;
use crate::engine::sql::structure::{Column, ForeignKey, Key, KeyReference, Table, TableName};
use once_cell::sync::Lazy;
use regex::Regex;
use std::iter::Peekable;

impl Column {
    fn from_sql_string(input: &str) -> Result<Self, String> {
        static COLUMN_NAME_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("(?i)^`([a-z0-9_]+)` ").unwrap());
        let matches = COLUMN_NAME_REGEX.captures(input.trim_start());

        if let Some(captures) = matches {
            Ok(Column {
                name: captures
                    .get(1)
                    .expect("The regex makes this always be here")
                    .as_str()
                    .into(),
            })
        } else {
            Err(format!("Invalid column spec: \"{}\"", input))
        }
    }
}

impl ForeignKey {
    fn from_sql_string(from_table: &str, input: &str) -> Result<Self, String> {
        static FK_LINE_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
            // This regex is a bit more relaxed then the actual syntax, but it will work anyway.
            // A strict regex would match keys like (`k1`, `k2`, `k3`) - it would make sure that
            // all k[1..] are followed by a comma except the last one.
            // Our regex accepts all kind of inconsistent use of commas, which would never be found
            // in the output of a SQL query.
            // A strict regex would be much more complex, but would not offer any real benifit.
            r"(?i)FOREIGN KEY \((?<from_keys>((`[a-z0-9_]+`),?\s*)+)\) REFERENCES `(?<to_table>[a-z0-9_]+)` \((?<to_keys>((`[a-z0-9_]+)`,?\s*)+)\)",
        )
        .unwrap()
        });

        let matches = FK_LINE_REGEX.captures(input.trim_start());

        if let Some(captures) = matches {
            let from_keys_source = captures
                .name("from_keys")
                .expect("from_keys capture group not optional");
            let to_table_source = captures
                .name("to_table")
                .expect("to_table capture group not optional");
            let to_keys_source = captures
                .name("to_keys")
                .expect("to_keys capture group not optional");

            Ok(ForeignKey {
                from: KeyReference::from_sql_str(from_table, from_keys_source.as_str()),
                to: KeyReference::from_sql_str(to_table_source.as_str(), to_keys_source.as_str()),
            })
        } else {
            Err(format!("Invalid foreign key spec: \"{}\"", input))
        }
    }
}

impl KeyReference {
    fn from_sql_str(table: &str, input: &str) -> Self {
        static SQL_NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[\w_]+\b").unwrap());
        let matches: Vec<_> = SQL_NAME_REGEX
            .find_iter(input)
            .map(|m| m.as_str())
            .collect();

        if matches.is_empty() {
            panic!("Found key with 0 columns"); // is this even possible?
        }

        let table = table.into();
        let key = matches.as_slice().into();

        Self { table, key }
    }
}

impl Key {
    fn from_sql_str(input: &str) -> Self {
        static SQL_NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[\w_]+\b").unwrap());
        let columns: Vec<_> = SQL_NAME_REGEX
            .find_iter(input)
            .map(|m| m.as_str())
            .map(|str| str.into())
            .collect();

        if columns.is_empty() {
            panic!("Found key with 0 columns"); // is this even possible?
        }

        Self { columns }
    }
}

// TODO

impl<'a, T> From<T> for Key
where
    T: Into<&'a [&'a str]>,
{
    fn from(value: T) -> Self {
        Self {
            columns: value.into().iter().map(|i| (*i).into()).collect(),
        }
    }
}

impl Key {
    fn try_from_sql_string(value: &str) -> Result<Self, String> {
        static SQL_NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[\w_]+\b").unwrap());
        let matches: Vec<_> = SQL_NAME_REGEX
            .find_iter(value)
            .map(|m| m.as_str())
            .collect();

        if matches.is_empty() {
            return Err(format!("Can't accept keys with 0 columns: {}", value));
        }

        Ok(matches.as_slice().into())
    }
}

impl Table {
    pub fn from_sql_string(input: &TableDescription) -> Result<Self, String> {
        let mut lines = input.as_str().trim_start().split('\n').peekable();

        let name: TableName = Self::parse_table_name_line(&mut lines)?.into();
        let columns = Self::parse_columns(&mut lines);
        let primary_key = Self::parse_primary_key(&mut lines)?;
        let foreign_keys = Self::parse_foreign_keys(name.0.as_str(), &mut lines);

        Ok(Table {
            name,
            primary_key,
            columns,
            foreign_keys,
        })
    }

    fn parse_table_name_line<'a>(
        lines: &mut dyn Iterator<Item = &'a str>,
    ) -> Result<&'a str, String> {
        if let Some(table_name_line) = lines.next() {
            static CREATE_TABLE_SQL_FIRST_LINE_REGEX: Lazy<Regex> =
                Lazy::new(|| Regex::new("(?i)^CREATE TABLE `([a-z0-9_]+)`").unwrap());
            let matches = CREATE_TABLE_SQL_FIRST_LINE_REGEX.captures(table_name_line);

            if let Some(captures) = matches {
                let table_name = captures.get(1).unwrap();

                Ok(table_name.as_str())
            } else {
                Err(format!(
                    "Column name line not as expected:\n{}",
                    table_name_line
                ))
            }
        } else {
            Err("Column name line not found".to_string())
        }
    }

    fn parse_columns(lines: &mut Peekable<std::str::Split<'_, char>>) -> Vec<Column> {
        let mut columns: Vec<Column> = Vec::new();

        while let Some(next_line) = lines.peek() {
            if let Ok(column) = Column::from_sql_string(next_line) {
                columns.push(column);
                lines.next();
            } else {
                // MySQL puts all of the columns at the beginning of 'show create table'
                // statements. Therefore, the first line that fails to parse as a column
                // is the start of the indexes section
                break;
            }
        }

        columns
    }

    fn parse_primary_key(lines: &mut dyn Iterator<Item = &str>) -> Result<Key, String> {
        if let Some(table_name_line) = lines.next() {
            static PRIMARY_KEY_SQL_LINE_REGEX: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"(?i)^\s*PRIMARY KEY \((?<key>((`[a-z0-9_]+`),?\s?)+)\)").unwrap()
            });
            let matches = PRIMARY_KEY_SQL_LINE_REGEX.captures(table_name_line);

            if let Some(captures) = matches {
                let table_names = captures.get(1).expect("Key group is not optional");

                Key::try_from_sql_string(table_names.as_str())
            } else {
                Err(format!(
                    "Unsupported primary key spec:\n{}",
                    table_name_line
                ))
            }
        } else {
            Err("Primary Key line not found".to_string())
        }
    }

    /// Consumes the rest of the iterator
    fn parse_foreign_keys(table: &str, lines: &mut dyn Iterator<Item = &str>) -> Vec<ForeignKey> {
        lines
            .map(|fk| ForeignKey::from_sql_string(table, fk))
            .filter_map(Result::ok)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_column() {
        let input = "`id` int(11) NOT NULL AUTO_INCREMENT,";
        let column = Column::from_sql_string(input).unwrap();

        assert_eq!(column.name, "id");
    }

    #[test]
    fn parse_foreign_key() {
        let input = "CONSTRAINT `FK_96C2225810EE4CEE` FOREIGN KEY (`parentId`, `fk2`) REFERENCES `teams` (`id`, `id2`) ON DELETE CASCADE,";
        let foreign_key = ForeignKey::from_sql_string("table", input).unwrap();

        assert_eq!(foreign_key.from.key.columns[0], "parentId");
        assert_eq!(foreign_key.from.key.columns[1], "fk2");
        assert_eq!(foreign_key.to.table, "teams");
        assert_eq!(foreign_key.to.key.columns[0], "id");
        assert_eq!(foreign_key.to.key.columns[1], "id2");
    }

    #[test]
    fn test_parse_table() {
        let input = "
CREATE TABLE `teams` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `name` varchar(50) COLLATE utf8_unicode_ci NOT NULL,
  `description` varchar(255) COLLATE utf8_unicode_ci DEFAULT NULL,
  `parentId` int(11) DEFAULT NULL,
  PRIMARY KEY (`id`, `id2`),
  KEY `IDX_96C22258F17FD7A5` (`customerId`),
  KEY `IDX_96C2225810EE4CEE` (`parentId`),
  CONSTRAINT `FK_96C2225810EE4CEE` FOREIGN KEY (`parentId`) REFERENCES `teams` (`id`) ON DELETE CASCADE,
  CONSTRAINT `FK_96C22258F17FD7A5` FOREIGN KEY (`customerId`) REFERENCES `customers` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci
";
        let input = TableDescription::new_for_tests(input);
        let table = Table::from_sql_string(&input).unwrap();

        assert_eq!(table.name, "teams");
        assert_eq!(table.primary_key.columns.len(), 2);
        assert_eq!(table.columns.len(), 4);
        assert_eq!(table.foreign_keys.len(), 2);
    }
}
