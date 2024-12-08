import { ApiPromise, WsProvider } from "@polkadot/api";

const PARA_ENDPOINT = "ws://127.0.0.1:9988";

async function orderPlacementWorks() {
    const paraEndpoint = new WsProvider(PARA_ENDPOINT);
    const paraApi = await ApiPromise.create({provider: paraEndpoint});

    

    console.log('hello');
}

orderPlacementWorks().then(() => console.log("\n✅ Test complete ✅"));
