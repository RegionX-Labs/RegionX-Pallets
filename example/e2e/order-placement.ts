import { ApiPromise, WsProvider } from "@polkadot/api";

const RELAY_ENDPOINT = "ws://127.0.0.1:9944";
const PARA_ENDPOINT = "ws://127.0.0.1:8844";

async function orderPlacementWorks() {
    const relayEndpoint = new WsProvider(RELAY_ENDPOINT);
    const relayApi = await ApiPromise.create({provider: relayEndpoint});

    const paraEndpoint = new WsProvider(PARA_ENDPOINT);
    const paraApi = await ApiPromise.create({provider: paraEndpoint});

    // TODO: configure on-demand on relay
    // TODO: assign a core to on-demand

    // TODO: register parachain
    // TODO: check if it is placing orders (SHOULD because the criteria is always returning true)

    // TODO: Once the criteria is updated to actually track something(e.g. number of pending transactions)
    // then ensure it is only placing orders when required.

    console.log('hello');
}

orderPlacementWorks().then(() => console.log("\n✅ Test complete ✅"));
