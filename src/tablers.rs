pub mod proportional;

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use chrono::{DateTime, Utc};

use crate::entries::Entry;

pub trait Table {
    type RowIter<'a>: Iterator<Item = &'a String>
    where
        Self: 'a;
    type ColIter<'a>: Iterator<Item = &'a DateTime<Utc>>
    where
        Self: 'a;
    type Item<'a>
    where
        Self: 'a;

    fn row_headers(&self) -> Self::RowIter<'_>;
    fn col_headers(&self) -> Self::ColIter<'_>;
    fn get(&self, row: String, col: DateTime<Utc>) -> Self::Item<'_>;
}

pub struct MyTable<T> {
    row_headers: HashSet<String>,
    col_headers: HashSet<DateTime<Utc>>,
    content: HashMap<(String, DateTime<Utc>), T>,
}

impl<T: Clone + Default> Table for MyTable<T> {
    type RowIter<'a> = std::collections::hash_set::Iter<'a, String> where T: 'a;
    type ColIter<'a> = std::collections::hash_set::Iter<'a, DateTime<Utc>> where T: 'a;
    type Item<'a> = T where Self: 'a;

    fn row_headers(&self) -> Self::RowIter<'_> {
        self.row_headers.iter()
    }

    fn col_headers(&self) -> Self::ColIter<'_> {
        self.col_headers.iter()
    }

    fn get(&self, row: String, col: DateTime<Utc>) -> Self::Item<'_> {
        match self.content.get(&(row, col)) {
            Some(v) => v.clone(),
            None => Default::default(),
        }
    }
}

impl<T> MyTable<T> {
    pub fn new() -> MyTable<T> {
        MyTable {
            col_headers: HashSet::new(),
            row_headers: HashSet::new(),
            content: HashMap::new(),
        }
    }

    /// Insert a value
    /// Return None if no value was present at (row, col).
    /// If a value was present, it is replaced by the new and the old value is returned
    fn insert(&mut self, row: String, col: DateTime<Utc>, item: T) -> Option<T> {
        self.col_headers.insert(col);
        self.row_headers.insert(row.clone());
        self.content.insert((row, col), item)
    }

    fn get_mut(&mut self, row: String, col: DateTime<Utc>) -> Option<&mut T> {
        self.content.get_mut(&(row, col))
    }
}

pub trait Tabler<'a> {
    type Table: Table<Item<'a>: Display>
    where
        Self: 'a;
    fn process(entries: Vec<Entry>) -> Self::Table;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion() {
        let mut t: MyTable<u8> = MyTable::new();
        let now = Utc::now();
        t.insert("".to_string(), now, 1);
        let old = t.insert("".to_string(), now, 2).unwrap();
        assert_eq!(1, old);
    }

    #[test]
    fn mutate_same() {
        let mut t: MyTable<u8> = MyTable::new();
        let now = Utc::now();
        t.insert("".to_string(), now, 1);
        let v = t.get_mut("".to_string(), now).unwrap();
        *v += 1;
        let v = t.get("".to_string(), now);
        assert_eq!(2, v);
    }

    #[test]
    fn get_default() {
        let t: MyTable<u8> = MyTable::new();
        let now = Utc::now();
        let v = t.get("".to_string(), now);
        assert_eq!(0, v);
    }
}