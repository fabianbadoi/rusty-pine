use crate::error::PineError;
use crate::sql::structure::Table;

pub fn render_columns(table: &str, table_specs: &[Table]) -> Result<String, PineError> {
    let table_spec = table_specs.iter().find(|spec| spec.name == table);

    if table_spec.is_none() {
        return Err(PineError::from(format!("Unknown table: {}", table)));
    }

    let table_spec = table_spec.unwrap();

    let column_list = table_spec.columns
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join("\n  ");


    Ok(format!("/*\nColumns for `{}`:\n  {}\n*/--", table, column_list))
}
