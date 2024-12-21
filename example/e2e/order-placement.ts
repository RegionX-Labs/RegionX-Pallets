import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types";
import { KeyringPair } from "@polkadot/keyring/types";
import { EventRecord } from "@polkadot/types/interfaces";
import assert from "assert";

const RELAY_ENDPOINT = "ws://127.0.0.1:9944";
const PARA_ENDPOINT = "ws://127.0.0.1:9988";

const PARA_ID = 2000;

const CHARLIE = "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y";
const EVE = "5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw";
const FERDIE = "5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL";

const COLLATORS = [CHARLIE, EVE, FERDIE];

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
    log(`Para height before stopping: ${paraHeight}`);

    // Wait some time to prove that the parachain is not producing blocks.
    await sleep(24 * 1000);

    var newParaHeight = (await paraApi.query.system.number()).toJSON() as number;
    assert(paraHeight === newParaHeight, "Para should stop with block production");

    await force(relayApi, relayApi.tx.parasSudoWrapper.sudoScheduleParachainDowngrade(PARA_ID));
    // Wait for new sesion for the parachain to downgrade:
    await sleep(120 * 1000);

    var newParaHeight = (await paraApi.query.system.number()).toJSON() as number;
    log(`Para height after switching to on-demand: ${newParaHeight}`);
    assert(newParaHeight > paraHeight, "Para should continue block production");

    let previousPlacer = '';
    await relayApi.query.system.events((events: any) => {
      events.forEach((record: EventRecord) => {
        const { event } = record;

        if(event.method === 'OnDemandOrderPlaced') {
          console.log(`${event.method} : ${event.data}`);
          const orderPlacer = event.data[2].toString();

          // Ensure orders are not always placed by the same collator:
          assert(orderPlacer !== previousPlacer);
          assert(COLLATORS.includes(orderPlacer));

          previousPlacer = orderPlacer;
        }
      });
    });

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

const log = (message: string) => {
  // Green log.
  console.log("\x1b[32m%s\x1b[0m", message);
}

/*
2024-12-20 18:42:11 [Parachain] 💤 Idle (0 peers), best: #5 (0xd386…54f5), finalized #5 (0xd386…54f5), ⬇ 0.1kiB/s ⬆ 0.1kiB/s    
2024-12-20 18:42:12 [Relaychain] 🏆 Imported #43 (0xfd8f…2e22 → 0x8f6f…7dce)    
2024-12-20 18:42:12 [Parachain] New best head: 0x8f6f…7dce    

====================

Version: 0.1.0-fc942848e22

   0: sp_panic_handler::set::{{closure}}
   1: std::panicking::rust_panic_with_hook
   2: std::panicking::begin_panic_handler::{{closure}}
   3: std::sys::backtrace::__rust_end_short_backtrace
   4: rust_begin_unwind
   5: core::panicking::panic_fmt
   6: core::option::expect_failed
   7: tracing::span::Span::in_scope
   8: sp_io::storage::get_version_1
   9: sp_io::storage::get
  10: frame_support::storage::unhashed::get
  11: <parachain_example_node::service::OrderPlacementCriteria as on_demand_service::config::OrderCriteria>::should_place_order
  12: on_demand_service::follow_relay_chain::{{closure}}
  13: on_demand_service::run_on_demand_task::{{closure}}::{{closure}}::{{closure}}
  14: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::future::future::Future>::poll
  15: <futures_util::future::future::Map<Fut,F> as core::future::future::Future>::poll
  16: <sc_service::task_manager::prometheus_future::PrometheusFuture<T> as core::future::future::Future>::poll
  17: <futures_util::future::select::Select<A,B> as core::future::future::Future>::poll
  18: <tracing_futures::Instrumented<T> as core::future::future::Future>::poll
  19: tokio::runtime::park::CachedParkThread::block_on
  20: sc_service::task_manager::SpawnTaskHandle::spawn_inner::{{closure}}
  21: tokio::runtime::task::core::Core<T,S>::poll
  22: tokio::runtime::task::harness::Harness<T,S>::poll
  23: tokio::runtime::blocking::pool::Inner::run
  24: std::sys::backtrace::__rust_begin_short_backtrace
  25: core::ops::function::FnOnce::call_once{{vtable.shim}}
  26: std::sys::pal::unix::thread::Thread::new::thread_start
  27: start_thread
             at ./nptl/pthread_create.c:447:8
  28: __GI___clone3
             at ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78:0


Thread 'tokio-runtime-worker' panicked at '`get_version_1` called outside of an Externalities-provided environment.', /home/sergej/.cargo/registry/src/index.crates.io-6f17d22bba15001f/sp-io-38.0.0/src/lib.rs:185

This is a bug. Please report it at:

	https://github.com/paritytech/polkadot-sdk/issues/new
*/
