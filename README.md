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

The first run will download the lists and build the rainbow table. Future runs
will reuse the cached table.

### CLI

```shell
$ cargo run --release 65,245,33,230,58,226
Loaded 14897972 entries
Found binding phrase: 'expresslrs'

$ cargo run --release
Loaded 14897972 entries
Press ctrl-d to exit

UID? 65,245,33,230,58,226
Found binding phrase: 'expresslrs'

UID? 67,127,47,177,211,57
Found binding phrase: 'ExpressLRS'
 ```

### Server

The server is set up to be deployed on [shuttle.rs](https://shuttle.rs/). It can
run locally with [`cargo-shuttle`](https://crates.io/crates/cargo-shuttle):

```shell
$ cd server && cargo shuttle run --release

$ curl localhost:8000/65,245,33,230,58,226
{
    "bindingPhrase": "expresslrs",
    "found": true
}
```

Or, try the deployed version: <https://elrs-uid.shuttleapp.rs/65,245,33,230,58,226>.
