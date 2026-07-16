//! Collection helpers for cross-frame capture finalization.

use crate::model::{Element, ListItem, TableCell, TableRow};

use super::types::{RawElement, RawListItem, RawTableCell, RawTableRow};

pub(super) fn finalize_list_items<F>(
    items: &[RawListItem],
    finalize_children: &mut F,
) -> Vec<ListItem>
where
    F: FnMut(&[RawElement]) -> Vec<Element>,
{
    items
        .iter()
        .map(|item| ListItem {
            children: finalize_children(&item.children),
        })
        .collect()
}

pub(super) fn finalize_table_rows<F>(
    rows: &[RawTableRow],
    finalize_children: &mut F,
) -> Vec<TableRow>
where
    F: FnMut(&[RawElement]) -> Vec<Element>,
{
    rows.iter()
        .map(|row| TableRow {
            cells: finalize_table_cells(&row.cells, finalize_children),
        })
        .collect()
}

pub(super) fn finalize_table_cells<F>(
    cells: &[RawTableCell],
    finalize_children: &mut F,
) -> Vec<TableCell>
where
    F: FnMut(&[RawElement]) -> Vec<Element>,
{
    cells
        .iter()
        .map(|cell| TableCell {
            children: finalize_children(&cell.children),
        })
        .collect()
}
