use crate::analyze::Column;
use crate::engine::syntax::TableInput;

pub fn render_columns(table: TableInput, columns: &[Column]) -> String {
    let mut buffer = format!("/*\nColumns for `{}`:\n", table.table.it.name);

    for column in columns {
        buffer.push_str("  ");
        buffer.push_str(column.name.0.as_str());
        buffer.push('\n');
    }

    buffer.push_str("*/--");

    buffer
}
