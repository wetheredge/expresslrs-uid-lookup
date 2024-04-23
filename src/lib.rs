use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::{iter, mem};

use md5::{Digest, Md5};
use memchr::memchr;
use rayon::prelude::*;

pub const RAW_WORDS: &str = "words.txt";
pub const LISTS_DIR: &str = "lists/";
pub const TABLE: &str = "table.bin";

pub const HASH_PREFIX: &[u8] = b"-DMY_BINDING_PHRASE=\"";
pub const HASH_SUFFIX: &[u8] = b"\"";

pub struct Table<'a>(Vec<(u64, &'a [u8])>);

impl<'a> Table<'a> {
    fn from_map(map: &BTreeMap<[u8; 6], &'a [u8]>) -> Self {
        let mut table = Vec::with_capacity(map.len());

        for (uid, &word) in map {
            table.push((uid_from_bytes(uid), word));
        }

        Self(table)
    }

    pub fn parse(file: &'a [u8]) -> Self {
        let entry_count_bytes = mem::size_of::<u32>();

        assert!(file.len() > entry_count_bytes, "File is too small");
        let count = u32::from_le_bytes(file[0..entry_count_bytes].try_into().unwrap());

        let mut table = Vec::with_capacity(count as usize);

        let mut i = entry_count_bytes;
        while i < (file.len() - 7) {
            let uid = uid_from_bytes(&file[i..(i + 6)]);
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

    pub fn from_words(words: &[u8]) -> io::Result<Table> {
        let table = words
            .par_split(|i| *i == b'\n')
            .map_init(Md5::new, |md5, word| {
                let word = word.strip_suffix(b"\r").unwrap_or(word);

                md5.update(HASH_PREFIX);
                md5.update(word);
                md5.update(HASH_SUFFIX);

                let hash: [u8; 16] = md5.finalize_reset().into();

                let [b0, b1, b2, b3, b4, b5, ..] = hash;
                let uid = [b0, b1, b2, b3, b4, b5];

                (uid, word)
            })
            .collect::<BTreeMap<_, _>>();
        println!("Generated lookup table with {} entries", table.len());

        write_table(&table, TABLE)?;
        Ok(Table::from_map(&table))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn find(&self, uid: u64) -> Option<&[u8]> {
        self.0
            .binary_search_by_key(&uid, |&(uid, _)| uid)
            .ok()
            .map(|i| self.0[i].1)
    }
}

fn uid_from_bytes(bytes: &[u8]) -> u64 {
    let mut uid = [0; 8];
    (uid[2..]).copy_from_slice(bytes);
    u64::from_be_bytes(uid)
}

pub fn parse_uid(uid: &str) -> Option<u64> {
    let bytes = iter::repeat(Some(0_u8))
        .take(2)
        .chain(uid.splitn(15, ',').map(|s| s.parse().ok()))
        .collect::<Option<Vec<_>>>()?;

    let bytes = bytes.try_into().ok()?;
    Some(u64::from_be_bytes(bytes))
}

pub fn fetch_words() -> anyhow::Result<Vec<u8>> {
    let base = b"ExpressLRS\nexpresslrs\nELRS\nelrs";

    let plain_lists = [
        "https://github.com/dwyl/english-words/raw/master/words.txt",
        "https://archive.org/download/mobywordlists03201gut/SINGLE.TXT",
        "https://archive.org/download/mobywordlists03201gut/ACRONYMS.TXT",
        "https://archive.org/download/mobywordlists03201gut/COMPOUND.TXT",
        "https://archive.org/download/mobywordlists03201gut/NAMES.TXT",
        "https://github.com/brannondorsey/naive-hashcat/releases/download/data/rockyou.txt",
    ];

    let diceware_list = "https://www.eff.org/files/2016/07/18/eff_large_wordlist.txt";

    let mut output = Vec::from(base.as_slice());

    for word in base {
        writeln!(output, "{word}")?;
    }

    for url in plain_lists {
        println!("Fetching {url}");
        let mut words = ureq::get(url).call()?.into_reader();
        io::copy(&mut words, &mut output)?;
    }

    let diceware_words = ureq::get(diceware_list).call()?.into_reader();
    let diceware_words = BufReader::new(diceware_words);
    for line in diceware_words.lines() {
        let line = &line?;

        let (_, word) = line.split_once('\t').unwrap();
        writeln!(output, "{word}")?;
    }

    Ok(output)
}

fn write_table<P: AsRef<Path>>(table: &BTreeMap<[u8; 6], &[u8]>, path: P) -> io::Result<()> {
    assert!(table.len() <= u32::MAX as usize, "Table is too large");

    let f = File::create(&path)?;
    let mut f = BufWriter::new(f);

    f.write_all(&(table.len() as u32).to_le_bytes())?;

    for (uid, word) in table {
        f.write_all(uid)?;
        f.write_all(word)?;
        f.write_all(&[0])?;
    }

    Ok(())
}
