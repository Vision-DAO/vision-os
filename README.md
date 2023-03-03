# Vision Operating System

Implementation of a WebAssembly runtime for [actor-based](https://en.wikipedia.org/wiki/Actor_model) web3 applications.

## Dependencies

1. Rust - This repo implements multiple Vision modules in Rust, with [Cargo Make](https://github.com/sagiegurari/cargo-make)
as its compilation harness.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install --force cargo-make
```

2. Hardhat - This repo also implements DAO smart contracts in Solidity, and requires hardhat for testing

```sh
yarn && yarn run build
```

## Usage

### Local Development

After acquiring all necessary dependencies, simply run `cargo make run` in the `fixtures/modules` directory to launch the Vision Operating System.

### Remote Development

Development of Vision apps is possible inside the Vision Operating System. To do so, simply open https://os.vision.eco in a web browser.
