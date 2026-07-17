// Step 9-5: Exercise table storage and primary index together.
//
// Run:
// rustc --edition=2021 --test 05_table_index_smoke.rs && ./05_table_index_smoke

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

#[derive(Debug)]
struct MiniTable {
    table: Table,
    p_index: PrimaryIndex,
}

impl MiniTable {
    fn new() -> Self {
        Self {
            table: Table::new(),
            p_index: PrimaryIndex::new(),
        }
    }

    fn insert(&mut self, row: Row) -> Result<(), String> {
        let record_id = self.table.insert(row.clone());
        self.p_index.insert(row.id, record_id);
        Ok(())
    }

    fn get(&self, id: i64) -> Result<Option<Row>, String> {
        if let Some(record_id) = self.p_index.get(id) {
            return Ok(self.table.get(record_id).cloned());
        }
        Ok(None)
    }
}

fn main() {
    println!("table index smoke");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_and_gets_rows_by_primary_key() {
        let mut table = MiniTable::new();

        table
            .insert(Row {
                id: 1,
                name: "alice".to_string(),
            })
            .unwrap();
        table
            .insert(Row {
                id: 2,
                name: "bob".to_string(),
            })
            .unwrap();

        assert_eq!(
            table.get(1).unwrap(),
            Some(Row {
                id: 1,
                name: "alice".to_string(),
            })
        );
        assert_eq!(
            table.get(2).unwrap(),
            Some(Row {
                id: 2,
                name: "bob".to_string(),
            })
        );
        assert_eq!(table.get(3).unwrap(), None);
    }
}
