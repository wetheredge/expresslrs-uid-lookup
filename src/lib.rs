use std::fs::File;
use std::io::{self, BufReader, Read};
use std::{iter, mem};

use memchr::memchr;

pub const RAW_WORDS: &str = "words.txt";
pub const LISTS_DIR: &str = "lists/";
pub const TABLE: &str = "table.bin";

pub const HASH_PREFIX: &[u8] = b"-DMY_BINDING_PHRASE=\"";
pub const HASH_SUFFIX: &[u8] = b"\"";

pub struct Table<'a>(Vec<(u64, &'a [u8])>);

impl<'a> Table<'a> {
    pub fn parse(file: &'a [u8]) -> Self {
        let entry_count_bytes = mem::size_of::<u32>();

        assert!(file.len() > entry_count_bytes, "File is too small");
        let count = u32::from_le_bytes(file[0..entry_count_bytes].try_into().unwrap());

        let mut table = Vec::with_capacity(count as usize);

        let mut i = entry_count_bytes;
        while i < (file.len() - 7) {
            let mut uid = [0; 8];
            (uid[2..]).copy_from_slice(&file[i..(i + 6)]);
            let uid = u64::from_be_bytes(uid);
            i += 6;

            let value_start = i;
            i = memchr(0, &file[i..]).map(|x| i + x).unwrap_or(file.len());
            let value = &file[value_start..i];
            // Skip trailing null
            i += 1;

            table.push((uid, value));
        }

        Self(table)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn find(&self, uid: u64) -> Option<&[u8]> {
        self.0
            .binary_search_by_key(&uid, |&(uid, _)| uid)
            .ok()
            .map(|i| self.0[i].1)
    }
}

pub fn load_table() -> io::Result<Vec<u8>> {
    let f = File::open(TABLE)?;
    let mut f = BufReader::new(f);

    let mut raw_table = Vec::new();
    f.read_to_end(&mut raw_table)?;

    Ok(raw_table)
}

pub fn parse_uid(uid: &str) -> Option<u64> {
    let bytes = iter::repeat(Some(0_u8))
        .take(2)
        .chain(uid.splitn(15, ',').map(|s| s.parse().ok()))
        .collect::<Option<Vec<_>>>()?;

    let bytes = bytes.try_into().ok()?;
    Some(u64::from_be_bytes(bytes))
}
