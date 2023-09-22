//! Structures used to represent the structure of the database. Used for using foreign keys to
//! augment our Pines.
#[derive(Debug, Clone)]
pub struct Database<'a> {
    pub name: TableName<'a>,
    pub tables: Vec<Table<'a>>,
}

#[derive(Debug, Clone)]
pub struct Column<'a> {
    pub name: ColumnName<'a>,
}

#[derive(Debug, Clone)]
pub struct Table<'a> {
    pub name: TableName<'a>,
    pub primary_key: Key<'a>,
    pub columns: Vec<Column<'a>>,
    pub foreign_keys: Vec<ForeignKey<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key<'a> {
    pub columns: Vec<ColumnName<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKey<'a> {
    pub from: KeyReference<'a>,
    pub to: KeyReference<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyReference<'a> {
    pub table: TableName<'a>,
    pub key: Key<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnName<'a>(pub &'a str);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableName<'a>(pub &'a str);

impl<'a> Table<'a> {
    pub fn get_foreign_key(&'a self, to_table: &str) -> Option<&ForeignKey<'a>> {
        self.foreign_keys
            .iter()
            .find(|foreign_key| foreign_key.to.table == to_table)
    }
}

impl PartialEq<&str> for ColumnName<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<&str> for TableName<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl<'a> From<&'a str> for ColumnName<'a> {
    fn from(name: &'a str) -> ColumnName<'a> {
        ColumnName(name)
    }
}

impl<'a> From<&'a str> for TableName<'a> {
    fn from(name: &'a str) -> TableName<'a> {
        TableName(name)
    }
}

impl<'a> From<&'a str> for Column<'a> {
    fn from(name: &'a str) -> Column<'a> {
        Column { name: name.into() }
    }
}

impl<'a> From<&'_ TableName<'a>> for &'a str {
    fn from(name: &'_ TableName<'a>) -> &'a str {
        name.0
    }
}
