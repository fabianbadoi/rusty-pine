use crate::error::PineError;
use crate::sql::structure::Table;

pub fn render_neighbours(table: &str, table_specs: &[Table]) -> Result<String, PineError> {
    let table_spec = table_specs.iter().find(|spec| spec.name == table);

    if table_spec.is_none() {
        return Err(PineError::from(format!("Unknown table: {}", table)));
    }

    let table_spec = table_spec.unwrap();

    let outgoing_neighbours = get_outgoing_neighbours(table_spec);
    let incoming_neighbours = get_incoming_neighbours(table, table_specs);

    let all_neighbours = outgoing_neighbours
        .into_iter()
        .chain(incoming_neighbours.into_iter())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!("/*\nForeign keys to:\n{}\n*/", all_neighbours))
}

fn get_incoming_neighbours(to_table: &str, table_specs: &[Table]) -> Vec<String> {
    table_specs
        .iter()
        .flat_map(|spec| {
            spec.foreign_keys
                .iter()
                .filter(|fk| fk.to_table == to_table)
                .map(move |fk| (&spec.name, fk))
        })
        .map(|(table, fk)| format!("  {}.{} using .{}", table, fk.from_column.0, fk.to_column.0))
        .collect()
}

fn get_outgoing_neighbours(table_spec: &Table) -> Vec<String> {
    table_spec
        .foreign_keys
        .iter()
        .map(|fk| {
            format!(
                "  {}.{} using .{}",
                fk.to_table.0, fk.to_column.0, fk.from_column.0
            )
        })
        .collect()
}
