# ExpressLRS Rainbow Table Generator

This is a Rust port of https://github.com/MUSTARDTIGERFPV/ELRS-Rainbow-Tables
focused on optimizing lookup speed. It attempts to find an ExpressLRS binding
phrase that matches the given uid.

This implementation uses a binary format for the rainbow table rather than CSV
to save on disk space and improve loading time. The lookup code also loads the
table into a preallocated byte array and swaps the hash map lookup for a binary
search. With these changes, data loading takes ~300 milliseconds on an i7 laptop
with an NVME drive. Each lookup then takes less than 25 microseconds.

## Usage

### Setup

```shell
# Download the word lists
$ ./fetch.sh

# Build the rainbow table (may take up to a few minutes)
$ cargo run --release --bin generate
```

### CLI

```shell
$ cargo run --release --bin cli 65,245,33,230,58,226
Loaded 14897972 entries
Found binding phrase: 'expresslrs'

$ cargo run --release --bin cli
Loaded 14897972 entries
Press ctrl-d to exit

UID? 65,245,33,230,58,226
Found binding phrase: 'expresslrs'

UID? 67,127,47,177,211,57
Found binding phrase: 'ExpressLRS'
 ```

### Server

```shell
$ cargo run --release --bin server
Loaded 14897972 entries
Running on 0.0.0.0:3000

$ curl localhost:3000/65,245,33,230,58,226
{
    "bindingPhrase": "expresslrs",
    "found": true
}
```
