use super::{Condition, Filter as SqlFilter, QualifiedColumnIdentifier, Query};

pub struct QueryShorthand(pub Select, pub From, pub &'static [Filter]);

pub struct Select(pub &'static [&'static str]);
pub struct From(pub &'static str);

pub enum Filter {
    Equals(&'static str, &'static str),
}

impl Into<Query<'static>> for QueryShorthand {
    fn into(self) -> Query<'static> {
        let mut query: Query = Default::default();

        let table = (self.1).0;

        query.from = Some(table); // TODO remove Option<>
        query.selections = self
            .0
             .0
            .iter()
            .map(|column| QualifiedColumnIdentifier { table, column })
            .collect();

        query.filters = self
            .2
            .iter()
            .map(|filter| match filter {
                Filter::Equals(column, value) => {
                    let column = QualifiedColumnIdentifier { table, column };
                    let condition = Condition::Equals(value);

                    SqlFilter { column, condition }
                }
            })
            .collect();

        query
    }
}
