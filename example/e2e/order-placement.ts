import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types";
import { KeyringPair } from "@polkadot/keyring/types";
import assert from "assert";

const RELAY_ENDPOINT = "ws://127.0.0.1:9944";
const PARA_ENDPOINT = "ws://127.0.0.1:9988";

const PARA_ID = 2000;

const keyring = new Keyring({ type: "sr25519" });

async function orderPlacementWorks() {
    const relayEndpoint = new WsProvider(RELAY_ENDPOINT);
    const relayApi = await ApiPromise.create({provider: relayEndpoint});

    const paraEndpoint = new WsProvider(PARA_ENDPOINT);
    const paraApi = await ApiPromise.create({provider: paraEndpoint});

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

    const paraHeight = (await paraApi.query.system.number()).toJSON() as number;
    console.log(`Para height before stopping: ${paraHeight}`);

    // Wait some time to prove that the parachain is not producing blocks.
    await sleep(24 * 1000);

    var newParaHeight = (await paraApi.query.system.number()).toJSON() as number;
    assert(paraHeight === newParaHeight, "Para should stop with block production");

    await force(relayApi, relayApi.tx.parasSudoWrapper.sudoScheduleParachainDowngrade(PARA_ID));
    // Wait for new sesion for the parachain to downgrade:
    await sleep(60 * 1000);

    var newParaHeight = (await paraApi.query.system.number()).toJSON() as number;
    console.log(`Para height after switching to on-demand: ${newParaHeight}`);
    assert(newParaHeight > paraHeight, "Para should continue block production");

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

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));
