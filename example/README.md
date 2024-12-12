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
./parachain-example-node export-genesis-wasm --chain local artifacts/para.wasm
```

Generate parachain genesis state:
```
./parachain-example-node export-genesis-state --chain local artifacts/para-genesis
```

Start the collator node:
```
./parachain-example-node \
--alice \
--collator \
--force-authoring \
--chain local \
--base-path /tmp/para/alice \
--port 40333 \
--rpc-port 8844 \
--rpc-cors all \
--unsafe-rpc-external \
--unsafe-force-node-key-generation \
-- \
--execution wasm \
--chain /tmp/zombie-5ee6a2f6c05b95cbb2b51bd07112b2b7_-63416-R5iP07csFccq/alice/cfg/rococo-local.json
--port 30343 \
--rpc-port 9977 \
--bootnodes /ip4/127.0.0.1/tcp/30333/ws/p2p/12D3KooWQCkBm1BYtkHpocxCwMgR8yjitEeHGx8spzcDLGt2gkBm
```
