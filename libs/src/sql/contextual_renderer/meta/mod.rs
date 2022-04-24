mod columns;
mod neighbours;

use crate::error::PineError;
use crate::query::RenderableMetaOperation;
use crate::sql::structure::Table;
use columns::render_columns;
use neighbours::render_neighbours;

/// Meta operations give info about the table structure, as opposed to the data
pub fn render_meta_operation(
    meta_op: &RenderableMetaOperation,
    table_specs: &[Table],
) -> Result<String, PineError> {
    match meta_op {
        RenderableMetaOperation::ShowNeighbours(table) => render_neighbours(table, table_specs),
        RenderableMetaOperation::ShowColumns(table) => render_columns(table, table_specs),
    }
}
