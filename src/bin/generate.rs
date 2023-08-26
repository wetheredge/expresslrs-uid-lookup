use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::process;

use md5::{Digest, Md5};
use rayon::prelude::*;

fn main() -> io::Result<()> {
    let words = get_raw_words()?;
    println!(
        "Loaded {} of raw words",
        bytesize::to_string(words.len() as u64, true)
    );

    let table = words
        .par_split(|i| *i == b'\n')
        .map_init(Md5::new, |md5, word| {
            let word = word.strip_suffix(b"\r").unwrap_or(word);

            md5.update(elrs_rainbow_table::HASH_PREFIX);
            md5.update(word);
            md5.update(elrs_rainbow_table::HASH_SUFFIX);

            let hash: [u8; 16] = md5.finalize_reset().into();

            let [b0, b1, b2, b3, b4, b5, ..] = hash;
            let uid = [b0, b1, b2, b3, b4, b5];

            (uid, word)
        })
        .collect::<BTreeMap<_, _>>();
    println!("Generated rainbow table with {} entries", table.len());

    write_table(&table)?;
    process::exit(0);
}

fn get_raw_words() -> Result<Vec<u8>, io::Error> {
    let mut words = Vec::new();
    for entry in fs::read_dir(elrs_rainbow_table::LISTS_DIR)? {
        File::open(entry?.path())?.read_to_end(&mut words)?;
    }

    Ok(words)
}

fn write_table(table: &BTreeMap<[u8; 6], &[u8]>) -> io::Result<()> {
    assert!(table.len() <= u32::MAX as usize, "Table is too large");

    let f = File::create(elrs_rainbow_table::TABLE)?;
    let mut f = BufWriter::new(f);

    f.write_all(&(table.len() as u32).to_le_bytes())?;

    for (uid, &word) in table {
        f.write_all(uid)?;
        f.write_all(word)?;
        f.write_all(&[0])?;
    }

    Ok(())
}
