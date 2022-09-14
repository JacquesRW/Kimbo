use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

#[derive(Default)]
pub struct PawnHashEntry {
    data: AtomicU32,
}
impl Clone for PawnHashEntry {
    fn clone(&self) -> Self {
        Self {
            data: AtomicU32::new(self.data.load(Ordering::Relaxed)),
        }
    }
}
const ENTRY_SIZE: usize = std::mem::size_of::<PawnHashEntry>();

pub struct PawnHashTable {
    table: Vec<PawnHashEntry>,
    num_entries: usize,
    filled: AtomicU64,
}

#[derive(Default)]
pub struct PawnHashResult {
    pub key: u16,
    pub score: i16,
}

impl PawnHashTable {
    pub fn new(size: usize) -> Self {
        let num_entries = size / ENTRY_SIZE;
        Self {
            table: vec![Default::default(); num_entries],
            num_entries,
            filled: AtomicU64::new(0),
        }
    }

    pub fn push(&self, hash: u64, score: i16) {
        let key = (hash >> 48) as u16;
        let idx = (hash as usize) % self.num_entries;
        if self.table[idx].data.load(Ordering::Relaxed) as u16 == 0 {
            self.filled.fetch_add(1, Ordering::Relaxed);
        }
        self.table[idx].store(key, score);
    }

    pub fn get(&self, zobrist: u64) -> Option<PawnHashResult> {
        let key = (zobrist >> 48) as u16;
        let idx = (zobrist as usize) % self.num_entries;
        let entry = &self.table[idx];
        let data = entry.load();
        if data.key == key {
            return Some(data)
        }
        None
    }

    pub fn hashfull(&self) -> u64 {
        self.filled.load(Ordering::Relaxed) * 1000 / self.num_entries as u64
    }
}

impl PawnHashEntry {
    fn store(&self, key: u16, score: i16) {
        let data = (key as u32) | (((score as u16) as u32) << 16);
        self.data.store(data, Ordering::Relaxed);
    }

    fn load(&self) -> PawnHashResult {
        let data = self.data.load(Ordering::Relaxed);
        PawnHashResult {
            key: data as u16,
            score: (data >> 16) as i16,
        }
    }
}