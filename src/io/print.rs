//! APIs to represent [`Chunk`] as a formatted table.

use crate::{
    array::{get_display, Array},
    chunk::Chunk,
};

use comfy_table::{Cell, Table};

/// Returns a visual representation of [`Chunk`]
pub fn write<A: AsRef<dyn Array>, N: AsRef<str>>(batches: &[Chunk<A>], names: &[N]) -> String {
    let mut table = Table::new();
    table.load_preset("||--+-++|    ++++++");

    if batches.is_empty() {
        return table.to_string();
    }

    let header = names.iter().map(|name| Cell::new(name.as_ref()));
    table.set_header(header);

    for batch in batches {
        let displayes = batch
            .arrays()
            .iter()
            .map(|array| get_display(array.as_ref()))
            .collect::<Vec<_>>();

        for row in 0..batch.len() {
            let mut cells = Vec::new();
            (0..batch.arrays().len()).for_each(|col| {
                let string = displayes[col](row);
                cells.push(Cell::new(&string));
            });
            table.add_row(cells);
        }
    }
    table.to_string()
}
