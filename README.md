# ERC-4626 in ink!

This is an implementation of ERC-4626 that I wrote to learn the basics of writing ink! smart contracts.  

The implementation was closely ported from OpenZeppelin's ERC-4626 Solidity implementation in version 5.0.0 of its smart contracts.  

Unfortunately, Solidity depends a lot on inheritance wheras Rust does not. This means that developers that want to use this **unaudited code** should be making a copy of it and making manual changes. There are `@dev` tags indicating points of interest for developers to tweak code.