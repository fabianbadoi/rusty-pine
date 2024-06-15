pub use neighbors::render_neighbors;
pub use query_rendering::render_query;
use std::fmt::{Display, Formatter};

mod neighbors;
mod query_rendering;

struct OptionalClause<'a, T> {
    intro: &'a str,
    ligature: &'a str,
    items: &'a [T],
}

impl<'a, T> OptionalClause<'a, T> {
    fn group_by(items: &'a [T]) -> Self {
        OptionalClause {
            intro: "GROUP BY",
            ligature: ",",
            items,
        }
    }

    fn order_by(items: &'a [T]) -> Self {
        OptionalClause {
            intro: "ORDER BY",
            ligature: ",",
            items,
        }
    }

    fn filter(items: &'a [T]) -> Self {
        OptionalClause {
            intro: "WHERE",
            ligature: " AND",
            items,
        }
    }
}

/// Displays things like "WHERE x AND Y AND Z", "GROUP BY 1, 2, 3", and "ORDER BY 1, 2, 3".
/// These are all optional fields that have a ligature between each element.
impl<'a, T> Display for OptionalClause<'a, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            intro,
            ligature,
            items,
        } = self;

        if let Some((first, rest)) = items.split_first() {
            write!(f, "{intro} {first}")?;

            for condition in rest {
                write!(f, "{ligature} {condition}")?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
