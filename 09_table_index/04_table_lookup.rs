// Step 9-4: Look up records through an index and table storage.
//
// Run:
// rustc --edition=2021 --test 04_table_lookup.rs && ./04_table_lookup

#![allow(unused)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotId(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RecordId {
    page_id: PageId,
    slot_id: SlotId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    id: i64,
    name: String,
}

#[derive(Debug)]
struct Table {
    rows: Vec<Option<Row>>,
}

#[derive(Debug)]
struct PrimaryIndex {
    entries: Vec<(i64, RecordId)>,
}

impl Table {
    fn new() -> Self {
        Self { rows: Vec::new() }
    }

    fn insert(&mut self, row: Row) -> RecordId {
        let slot_id = SlotId(self.rows.len() as u16);
        self.rows.push(Some(row));
        RecordId {
            page_id: PageId(0),
            slot_id,
        }
    }

    fn get(&self, record_id: RecordId) -> Option<&Row> {
        let idx = record_id.slot_id.0 as usize;
        if idx >= self.rows.len() {
            return None;
        }
        self.rows[idx].as_ref()
    }

    fn delete(&mut self, record_id: RecordId) {
        let idx = record_id.slot_id.0 as usize;
        if idx >= self.rows.len() {
            return;
        }
        if let Some(row) = self.rows.get_mut(idx) {
            // get_mut returns reference of the item.
            *row = None;
        }
    }
}

impl PrimaryIndex {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn insert(&mut self, key: i64, record_id: RecordId) {
        let position = self.leftmost_position(key);
        if position < self.entries.len() && self.entries[position].0 == key {
            self.entries[position].1 = record_id;
            return;
        }
        self.entries.push((key, record_id));
        let tail = self.entries.len() - 1;
        if position < tail {
            let mut current = tail;
            while current > position {
                self.entries.swap(current, current - 1);
                current -= 1;
            }
        }
    }

    fn get(&self, key: i64) -> Option<RecordId> {
        let position = self.leftmost_position(key);
        if position < self.entries.len() && self.entries[position].0 == key {
            return Some(self.entries[position].1);
        }
        None
    }
    fn leftmost_position(&self, key: i64) -> usize {
        let count = self.entries.len();

        let mut lo = 0;
        let mut hi = count;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.entries[mid].0 < key {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }

        lo
    }
}

fn get_by_primary_key<'a>(index: &PrimaryIndex, table: &'a Table, key: i64) -> Option<&'a Row> {
    if let Some(record_id) = index.get(key) {
        return table.get(record_id);
    }
    None
}

fn main() {
    println!("table lookup");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_row_by_primary_key() {
        let mut table = Table::new();
        let mut index = PrimaryIndex::new();
        let row = Row {
            id: 1,
            name: "alice".to_string(),
        };
        let record_id = table.insert(row.clone());
        index.insert(row.id, record_id);

        assert_eq!(get_by_primary_key(&index, &table, 1), Some(&row));
        assert_eq!(get_by_primary_key(&index, &table, 2), None);
    }

    #[test]
    fn deleted_record_is_not_visible() {
        let mut table = Table::new();
        let mut index = PrimaryIndex::new();
        let record_id = table.insert(Row {
            id: 1,
            name: "alice".to_string(),
        });
        index.insert(1, record_id);
        table.delete(record_id);

        assert_eq!(get_by_primary_key(&index, &table, 1), None);
    }
}
