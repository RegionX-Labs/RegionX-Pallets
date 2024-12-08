# On-Demand Example Parachain

### Testing process


Generate relay chain spec:
```
./polkadot build-spec --disable-default-bootnode --chain rococo-dev > artifacts/rococo.json
```

Start the relay chain:
```
./polkadot \
--alice \
--validator \
--base-path /tmp/relay/alice \
--chain artifacts/rococo.json \
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
--chain artifacts/rococo.json \
--port 30343 \
--rpc-port 9977 \
--bootnodes /ip4/127.0.0.1/tcp/30333/ws/p2p/12D3KooWSoY6HPbvLbiL6HXg8JLH57p2tfxnbzYStV3Rk7X5VrK3
```
