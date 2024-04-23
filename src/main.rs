use std::io::{self, BufRead, Write};
use std::path::Path;
use std::{env, fs, process};

use elrs_uid_lookup::Table;

fn main() -> anyhow::Result<()> {
    let raw_data;
    let table = if Path::new(elrs_uid_lookup::TABLE).exists() {
        raw_data = fs::read(elrs_uid_lookup::TABLE)?;
        Table::parse(&raw_data)
    } else {
        raw_data = elrs_uid_lookup::fetch_words()?;
        Table::from_words(&raw_data)?
    };

    println!("Loaded {} entries", table.len());

    if let Some(uid) = env::args().nth(1) {
        let code = if handle_uid(&table, &uid)? { 0 } else { 1 };
        process::exit(code)
    } else {
        println!("Press ctrl-d to exit");
        let mut stdin = io::stdin().lock();

        loop {
            let mut uid = String::new();
            print!("\nUID? ");
            io::stdout().flush()?;

            // Exit on EOF
            if stdin.read_line(&mut uid)? == 0 {
                return Ok(());
            }

            handle_uid(&table, &uid)?;
        }
    }
}

fn handle_uid(table: &Table, uid: &str) -> io::Result<bool> {
    #[cfg(feature = "time-lookup")]
    let start = std::time::Instant::now();

    let Some(uid) = elrs_uid_lookup::parse_uid(uid.trim()) else {
        println!("Invalid uid");
        return Ok(false);
    };

    let result = if let Some(binding_phrase) = table.find(uid) {
        let mut stdout = io::stdout();
        write!(stdout, "Found binding phrase: '")?;
        stdout.write_all(binding_phrase)?;
        writeln!(stdout, "'")?;

        Ok(true)
    } else {
        println!("Did not find binding phrase");
        Ok(false)
    };

    #[cfg(feature = "time-lookup")]
    {
        let end = std::time::Instant::now();
        eprintln!("Took {:?}", end.duration_since(start));
    }

    result
}
