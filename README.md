# Crusty n3xB

Crusty n3xB is a library written in Rust, implementing the [n3xB Bitcoin exchange protocol](https://github.com/nobu-maeda/n3xb/). Since n3xB is a protocol specification standardizing order formats, order publishing, and order taking, it still requires a trade engine along with a UI to be implemented on top for a full trading experience. To better understand this, please refer to the [architectural description](https://github.com/nobu-maeda/n3xb/blob/master/specs/architecture/architecture.md) of the n3xB protocol for details.

## Demo Example Implementation

An example exchange application and an example trade engine have been written for demonstration purposes. See the macOS/iOS application [OceanSea](https://github.com/nobu-maeda/oceansea) and the [FatCrab Trade Engine](https://github.com/nobu-maeda/fatcrab-trading) for details.

## Usage

For now, no Rust crate have yet been created for the Crusty-n3xB library. To use the library, one have to reference the Github location of this project in their Cargo.toml. An example of how this can be done can be seen in the [Cargo.toml of the Fatcrab Trading project](https://github.com/nobu-maeda/fatcrab-trading/blob/ff9af0479b2b8ace4bdd3aff65e5968cfe4970d9/Cargo.toml#L20)

## Discrepancies & Other Questions

There will inevitably be discrepancies between the [n3xB protocol specification](https://github.com/nobu-maeda/n3xb/) and what is actually implemented in this Crusty n3xB library. If you find one, or have any other questions or feedback, feel free to raise an issue here in Github, or visit the [n3xB Discord](https://discord.com/invite/5CFBMF38Nh).
