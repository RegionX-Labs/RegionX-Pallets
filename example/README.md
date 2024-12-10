# On-Demand Example Parachain

### Testing process

Setup: https://paritytech.github.io/zombienet/cli/setup.html
> NOTE: Get both polkadot and polkadot-parachain binaries.

Start the zombienet network:
```
zombienet-linux -p native spawn zombienet.toml
```

Generate parachain genesis wasm:
```
./parachain-example-node export-genesis-wasm --chain dev artifacts/example-para.wasm
```

Generate parachain genesis state:
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
--base-path /tmp/para/alice \
--port 40333 \
--rpc-port 8844 \
--rpc-cors all \
--unsafe-rpc-external \
--unsafe-force-node-key-generation \
-- \
--execution wasm \
--chain <COPY FROM ZOMBIENET ALICE COMMAND>
--port 30343 \
--rpc-port 9977 \
--bootnodes /ip4/127.0.0.1/tcp/30333/ws/p2p/<COPY THE LOCAL IDENTITY FROM ALICE'S ZOMBIENET LOGS>
```
