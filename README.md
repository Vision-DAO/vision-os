# Beacon DAO

Implementation of the Vision [Beacon DAO](https://bookstack.visiondaodev.com/books/vision-v2-specification/chapter/beacon-dao)
in Solidity, and core modules in Rust. Uses hardhat for testing.

## Dependencies

* Rust - This repo implements multiple Vision modules in Rust, with [Cargo Make](https://github.com/sagiegurari/cargo-make)
as its compilation harness.
* Hardhat - This repo also implements DAO smart contracts in Solidity, and
requires hardhat for testing

## Usage

For consuming contracts, add this repository as a dependency, and import the
relevant types, which are listed in `./types`. For example, to get the metadata
address of an Idea:

```typescript
import { Idea__factory } from "beacon-dao";

const idea = Idea__factory.connect(address, provider);
console.log(await idea.ipfsAddr());
```
