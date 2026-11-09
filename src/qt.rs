use qtbridge::{QTableModel, qobject};

use crate::dbus::Unit;

#[derive(Default, PartialEq)]
pub struct UnitTableModel(pub Vec<Unit>);

#[qobject(Base = QTableModel)]
impl QTableModel for UnitTableModel {
    type Item = String;

    fn row_count(&self) -> usize {
        self.0.len()
    }

    fn column_count(&self) -> usize {
        if self.0.is_empty() {
            0
        } else {
            3 // name, description, active_state
        }
    }

    fn get(&self, index: (usize, usize)) -> Option<&Self::Item> {
        let (row, col) = index;
        let item = self.0.get(row)?;

        match col {
            0 => Some(&item.name),
            1 => Some(&item.description),
            2 => Some(&item.active_state),

            _ => None
        }
    }
}
