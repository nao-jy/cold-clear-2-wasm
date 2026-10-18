# Cold Clear 2

Cold Clear 2 is a modern Tetris versus bot and a complete rewrite and evolution
of [Cold Clear](https://github.com/MinusKelvin/cold-clear). It implements the
[Tetris Bot Protocol](https://github.com/tetris-bot-protocol/tbp-spec) for
interaction with a frontend, such as [Quadspace](https://github.com/SoRA-X7/Quadspace).

## Technical Features

- Column-major bitboards
- WebAssembly interface using the Tetris Bot Protocol message types
- Transposition-aware game tree
- MCTS-inspired tree expansion

## WebAssembly

Build the library for the `wasm32-unknown-unknown` target and use
`wasm-bindgen` to generate JavaScript bindings. `initialize()` returns the
Tetris Bot Protocol `info` message. Pass protocol messages to
`process_message(message)`; it returns the corresponding bot message, or
`null` when there is no response.

`configure(config)` accepts the bot configuration object before starting a
game. Search runs synchronously in WebAssembly: call `think(iterations)` to
spend a chosen number of search iterations, then send a `suggest` message to
get the current suggestion. A suggestion request performs one search iteration
automatically if no search has been done since the last start or play message.

## License

Cold Clear 2 is licensed under either [Apache License Version 2.0](LICENSE-APACHE)
or [MIT License](LICENSE-MIT), at your option.
