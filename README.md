# ERC-4626 in ink!

This is an implementation of ERC-4626 that I wrote to learn the basics of writing ink! smart contracts. ERC4626 is a standardization of minting, burning, and redeeming a token
for other assets.  

The implementation was closely ported from OpenZeppelin's ERC-4626 Solidity implementation in version 5.0.0 of its smart contracts.  

Unfortunately, Solidity depends a lot on inheritance wheras Rust does not. This means that developers that want to use this **unaudited code** should be making a copy of it and making manual changes. There are `@dev` tags indicating points of interest for developers to tweak code. In the future this may be changed to generative macros+traits similar to OpenBrush.  

## base
This folder contains the baes ERC-4626 smart contract that can be used as a boiler plate template.  

Since this is an ink! smart contract, the complete implementation isn't available for your
specific parachain, as you may require specific pallets to work with specific assets. Please look for @dev tags to see where manual implementation is necessary.  

This smart contract was written and based off of the ERC20 smart contract provided by the
ink-examples repository.  

## erc_4626_zeit
This folder contains an example variant of the ERC-4626 smart contract base that holds native Zeitgeist (ZTG) on the Zeitgeist battery chain.  