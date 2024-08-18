use crate::analyze::ForeignKey;
use crate::engine::rendering::OptionalClause;

pub fn render_neighbors(neighbors: Vec<ForeignKey>) -> String {
    // We wrap the response in a comment. This makes sure we probably won't actually
    // run a query while just displaying things.
    let mut rendering = "/*\nForeign keys to:\n".to_string();

    for fk in neighbors {
        let intro = format!(
            "{}.{} using",
            fk.to.table.name.as_str(),
            fk.to
                .key
                .columns
                .iter()
                .map(|c| c.0.as_str())
                .collect::<Vec<_>>()
                .join("+")
        );

        let columns: Vec<_> = fk
            .from
            .key
            .columns
            .iter()
            .map(|c| format!(".{c}"))
            .collect();

        let key = OptionalClause {
            intro: intro.as_str(),
            ligature: "+",
            items: columns.as_slice(),
        };

        rendering.push_str(format!("  {key}").as_str());
    }

    rendering.push_str("*/--");

    rendering
}
