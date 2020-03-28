use crate::error::PineError;
use crate::sql::structure::Table;

pub fn render_neighbours(table: &str, table_specs: &[Table]) -> Result<String, PineError> {
    let table_spec = table_specs.iter().find(|spec| spec.name == table);

    if table_spec.is_none() {
        return Err(PineError::from(format!("Unknown table: {}", table)));
    }

    let table_spec = table_spec.unwrap();
    let neighbours = table_spec
        .foreign_keys
        .iter()
        .map(|fk| {
            format!(
                "  {}.{} using .{}",
                fk.to_table.0, fk.to_column.0, fk.from_column.0
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!("/*\nForeign keys to:\n{}\n*/", neighbours))
}
