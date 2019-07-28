use crate::query::{Condition, Filter as SqlFilter, QualifiedColumnIdentifier, Query};

pub struct QueryShorthand(pub Select, pub From, pub &'static [Filter]);

pub struct Select(pub &'static [&'static str]);
pub struct From(pub &'static str);

pub enum Filter {
    Equals(&'static str, &'static str),
}

impl Into<Query> for QueryShorthand {
    fn into(self) -> Query {
        let mut query: Query = Default::default();

        let table = (self.1).0.to_string();

        query.from = table.clone();
        query.selections = self
            .0
             .0
            .iter()
            .map(|str_ref| str_ref.to_string())
            .map(|column| QualifiedColumnIdentifier {
                table: table.clone(),
                column,
            })
            .collect();

        query.filters = self
            .2
            .iter()
            .map(|filter| match filter {
                Filter::Equals(column, value) => {
                    let column = column.to_string();
                    let table = table.clone();
                    let column = QualifiedColumnIdentifier { table, column };
                    let condition = Condition::Equals(value.to_string());

                    SqlFilter { column, condition }
                }
            })
            .collect();

        query
    }
}