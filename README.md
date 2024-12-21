
# NAME

Blockchain Exercise demonstrating the _Proof of Work_ concept

# DESCRIPTION

This service is an exercise to demonstrate the functionality of a cryptocurrency node
as _HTTP API_.

This combines blockchain concepts with web development technology producing a
extremely fast crypto-currency implementation which shows off how _Actix_ can add
elegance and speed to a crypto-currency.

# MOTIVATION

Translating the concepts seen in the blockchain course
[Blockchain and Smart Contract Introduction](https://www.udemy.com/certificate/UC-75023750-2591-42ce-aeed-c5519c9d7cbd/)
in a _Rust_ implementation as a small auto-sufficient Web Application.

# REQUIREMENTS

To rebuild this web site the **Minimum Rust Compiler Version** is _Rust_ `1.49`.
The site uses the libraries `Actix`, `Serde` and `json-rust`.
The _Actix_ Web Server requires the `Tokio` library.
The Server Responses are provided as `JSON` documents.

# INSTALLATION

- cargo

The `cargo` Command will install the dependencies on local user level as they
are found in the `Cargo.toml` file.

# EXECUTION

- `cargo run`

The Site can be launched using the `cargo run` Command.
To launch the Site call the `cargo run` Command within the project directory:

    cargo run

# IMPLEMENTATION

- Actor Model

To not block the server main thread too long and to enable asynchronous request processing
the `Actor` trait of _Actix_ and `Future`s are used.

- Miner Actor

The _Proof of Work_ block mining runs in a dedicated miner thread which keeps the
API operative while a new block is mined.

- Mutex

The central data needs to be shared between threads since transaction are added through
the HTTP API but also new blocks need to be mined in dedicated threads.
