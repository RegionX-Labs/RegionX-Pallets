import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types";
import { compactAddLength } from '@polkadot/util';
import { KeyringPair } from "@polkadot/keyring/types";
import { promises as fs } from 'fs';

const RELAY_ENDPOINT = "ws://127.0.0.1:9944";
const PARA_ENDPOINT = "ws://127.0.0.1:8844";

const keyring = new Keyring({ type: "sr25519" });

async function orderPlacementWorks() {
    const relayEndpoint = new WsProvider(RELAY_ENDPOINT);
    const relayApi = await ApiPromise.create({provider: relayEndpoint});

    // const paraEndpoint = new WsProvider(PARA_ENDPOINT);
    // const paraApi = await ApiPromise.create({provider: paraEndpoint});

    // Configure on-demand on the relay chain
    const configureTxs = [
        relayApi.tx.configuration.setOnDemandBaseFee(1_000_000),
        relayApi.tx.configuration.setOnDemandQueueMaxSize(100),
        relayApi.tx.configuration.setCoretimeCores(3),
        relayApi.tx.configuration.setSchedulingLookahead(2),
    ];
    await force(relayApi, relayApi.tx.utility.batchAll(configureTxs));

    // Assigning a core to the instantaneous coretime pool:
    await force(relayApi, relayApi.tx.coretime.assignCore(1, 0, [['Pool', 57600]], null));
    // ^^^^^^^^^^^
    // NOTE: The scheduler updates CoreDescriptors only during the para inherent process 
    // (specifically when backing a candidate). This means that if we only have an 
    // on-demand chain without any other chains, assigning a core to the insta pool 
    // will remain stuck in CoreSchedules.
    //
    // Because of this, we will always run a system parachain in our test cases.   

    const alice = keyring.addFromUri("//Alice");

    // await submitExtrinsic(alice, relayApi.tx.registrar.reserve(), {});
    const paraId = (await relayApi.query.registrar.nextFreeParaId()).toJSON() as number - 1;

    const genesisHead = await readFileAsUint8Array("../artifacts/para-genesis");
    const wasm = await readFileAsUint8Array("../artifacts/para.wasm");
    await submitExtrinsic(
      alice, 
      relayApi.tx.registrar.register(
        paraId,
        compactAddLength(genesisHead),
        compactAddLength(wasm)
      ),
      {}
    );

    // TODO: check if it is placing orders (SHOULD because the criteria is always returning true)

    // TODO: Once the criteria is updated to actually track something(e.g. number of pending transactions)
    // then ensure it is only placing orders when required.
}

orderPlacementWorks().then(() => console.log("\n✅ Test complete ✅"));

async function force(api: ApiPromise, call: SubmittableExtrinsic<"promise">): Promise<void> {
  const sudoCall = api.tx.sudo.sudo(call);

  const alice = keyring.addFromUri("//Alice");

  await submitExtrinsic(alice, sudoCall, {});
}

async function submitExtrinsic(
  signer: KeyringPair,
  call: SubmittableExtrinsic<"promise">,
  options: Partial<SignerOptions>
): Promise<void> {
  try {
    return new Promise((resolve, _reject) => {
      const unsub = call.signAndSend(signer, options, (result) => {
        console.log(`Current status is ${result.status}`);
        if (result.status.isInBlock) {
          console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
        } else if (result.status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
          unsub.then();
          return resolve();
        } else if (result.isError) {
          console.log("Transaction error");
          unsub.then();
          return resolve();
        }
      });
    });
  } catch (e) {
    console.log(e);
  }
}

async function readFileAsUint8Array(filePath: string): Promise<Uint8Array> {
    const data = await fs.readFile(filePath);
    return new Uint8Array(data);
}
