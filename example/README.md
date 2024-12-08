# On-Demand Example Parachain

### Testing process

Start the relay chain:
```
./polkadot \
--alice \
--validator \
--base-path /tmp/relay/alice \
--chain rococo-dev \
--port 30333 \
--rpc-port 9944 \
--unsafe-force-node-key-generation
```

Generate genesis wasm:
```
./parachain-example-node export-genesis-wasm --chain dev artifacts/example-para.wasm
```

Generate genesis state:
```
./parachain-example-node export-genesis-state --chain dev artifacts/example-state
```

Start the collator node:
```
./parachain-example-node \
--alice \
--collator \
--force-authoring \
--chain dev \
--base-path ./tmp/para/alice \
--port 40333 \
--rpc-port 8844 \
--rpc-cors all \
--unsafe-rpc-external \
--unsafe-force-node-key-generation \
-- \
--execution wasm \
--chain rococo-dev \
--port 30343 \
--rpc-port 9977
```
