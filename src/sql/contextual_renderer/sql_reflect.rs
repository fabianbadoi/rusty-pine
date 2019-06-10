use super::structure::{Column, Table};
use crate::error::ParseError;
use regex::Regex;

// TODO creating regex instances on every function call is not optimal.
impl Column {
    pub fn from_sql_string(input: &str) -> Result<Column, ParseError> {
        let regex = Regex::new("(?i)^`([a-z0-9_]+)` ").unwrap();
        let matches = regex.captures(input.trim_start());

        if let Some(captures) = matches {
            Ok(Column {
                name: captures[1].to_string(),
            })
        } else {
            Err(ParseError::from_message(
                format!("Invalid column spec: \"{}\"", input),
            ))
        }
    }
}

impl Table {
    pub fn from_sql_string(input: &str) -> Result<Table, ParseError> {
        let mut lines = input.trim_start().split('\n');

        let name = Self::parse_table_name_line(&mut lines)?;
        let columns = Self::parse_columns(&mut lines);       

        Ok(Table { name, columns })
    }

    fn parse_table_name_line(lines: &mut Iterator<Item = &str>) -> Result<String, ParseError> {
        if let Some(table_name_line) = lines.next() {
            let regex = Regex::new("(?i)^CREATE TABLE `([a-z0-9_]+)`").unwrap();
            let matches = regex.captures(table_name_line);

            if let Some(captures) = matches {
                let table_name = captures.get(1).unwrap();

                Ok(table_name.as_str().to_string())
            } else {
                Err(ParseError::from_message(format!("Column name line not as expected:\n{}", table_name_line)))
            }
        } else {
            Err(ParseError::from_str("Column name line not found"))
        }
    }

    fn parse_columns(lines: &mut Iterator<Item = &str>) -> Vec<Column> {
        let mut columns: Vec<Column> = Vec::new();
        
        for line in lines {
            if let Ok(column) = Column::from_sql_string(line) {
                columns.push(column);
            } else {
                // MySQL puts all of the columns at the beginning of 'show create table'
                // statements. Therefore, the first line that fails to parse as a column
                // is the start of the indexes section.
                break;
            }
        }

        columns
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
    fn test_parse_table() {
        let input = "
CREATE TABLE `teams` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `name` varchar(50) COLLATE utf8_unicode_ci NOT NULL,
  `description` varchar(255) COLLATE utf8_unicode_ci DEFAULT NULL,
  `parentId` int(11) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `IDX_96C22258F17FD7A5` (`customerId`),
  KEY `IDX_96C2225810EE4CEE` (`parentId`),
  CONSTRAINT `FK_96C2225810EE4CEE` FOREIGN KEY (`parentId`) REFERENCES `teams` (`id`) ON DELETE CASCADE,
  CONSTRAINT `FK_96C22258F17FD7A5` FOREIGN KEY (`customerId`) REFERENCES `customers` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci
";
        let table = Table::from_sql_string(input).unwrap();

        assert_eq!(table.name, "teams");
        assert_eq!(table.columns.len(), 4);
    }
}

