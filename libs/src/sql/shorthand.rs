use crate::common::BinaryFilterType;
use crate::query::{Filter as SqlFilter, Operand, QualifiedColumnIdentifier, Query, ResultColumn};

pub struct QueryShorthand(pub Select, pub From, pub &'static [Filter]);

pub struct Select(pub &'static [&'static str]);
pub struct From(pub &'static str);

pub enum Filter {
    Binary(&'static str, &'static str, BinaryFilterType),
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
            .map(|column| ResultColumn::Column(column))
            .collect();

        query.filters = self
            .2
            .iter()
            .map(|filter| match filter {
                Filter::Binary(rhs, lhs, filter_type) => {
                    let rhs = parse_operand(rhs);
                    let lhs = parse_operand(lhs);

                    SqlFilter::Binary(rhs, lhs, *filter_type)
                }
            })
            .collect();

        query
    }
}

fn parse_operand(operand: &str) -> Operand {
    if operand.contains('.') {
        let parts: Vec<&str> = operand.split('.').collect();

        Operand::Column(QualifiedColumnIdentifier {
            table: parts[0].to_string(),
            column: parts[1].to_string(),
        })
    } else {
        Operand::Value(operand.to_string())
    }
}
