use futures::future::Future;
use futures::future::TryFutureExt;

mod client;
use client::Client;

use zkms_common::CryptoAlgo;
use zkms_ductile::{
    crypto::{self, Pair as _},
    ecdsa, ed25519, sr25519,
};

pub async fn execute_tests(addr: impl std::net::ToSocketAddrs) {
    info!("Connecting testing client...");
    let client = Client::connect(addr).expect("server not running!");

    info!("TESTS STARTING");

    Test::new(
        "generateNew 00",
        "generate new sr25519 keypair and return a public key; no seed",
        || {
            client
                .sr25519_generate_new()
                .map_err(|e| format!("failed to issue request: {:?}", e))?;
            Ok::<_, String>(())
        },
    )
    .exec();

    Test::new(
        "signMessage 00",
        "sign a message with sr25519 and verify signature",
        || {
            let key = client
                .sr25519_generate_new()
                .map_err(|e| format!("failed to issue request: {:?}", e))?;

            const MSG: &[u8] = "support@zondax.ch".as_bytes();

            let sign = client
                .sign_with(CryptoAlgo::Sr25519, key.to_vec(), MSG)
                .map_err(|e| format!("failed to issue request: {:?}", e))?;

            let sign = sr25519::Signature::from_slice(&sign[..]);

            if sr25519::Pair::verify(&sign, MSG, &key) {
                Ok(())
            } else {
                Err("Signature was not valid".to_string())
            }
        },
    )
    .exec();

    Test::new(
        "getPublicKeys 00",
        "attempt to retrieve sr25519 public keys",
        || {
            let keys = client.sr25519_public_keys();

            debug!("sr25519 keys={:x?}", keys);

            Ok::<_, String>(())
        },
    )
    .exec();

    Test::new(
        "getPublicKeys 01",
        "attempt to retrieve sr25519 public keys, min 1",
        || {
            let _ = client
                .sr25519_generate_new()
                .map_err(|e| format!("unable to make request: {:?}", e))?;

            let keys = client.sr25519_public_keys();

            debug!("sr25519 keys={:x?}", keys);

            if keys.len() < 1 {
                Err("at least 1 key should be present".to_string())
            } else {
                Ok(())
            }
        },
    )
    .exec();

    Test::new(
        "hasKeys 00",
        "attempt to check presence of non existing keys",
        || {
            let query = vec![vec![0; 32]];

            let search = client.has_keys(query.clone());

            debug!("haskeys; query={:x?}, search={}", query, search);

            Ok::<_, String>(())
        },
    )
    .exec();

    Test::new(
        "hasKeys 01",
        "attempt to check presence of freshly generated key",
        || {
            let pk = client
                .sr25519_generate_new()
                .map_err(|e| format!("failed to issue request: {:?}", e))?;

            let query = vec![pk.0.to_vec()];

            let search = client.has_keys(query.clone());

            debug!("haskeys; query={:x?}, search={}", query, search);

            if !search {
                Err("search not ok".to_string())
            } else {
                Ok(())
            }
        },
    )
    .exec();

    Test::new(
        "vrfSign 00",
        "attempt to sign vrf with freshly generated key",
        || {
            let pk = client
                .sr25519_generate_new()
                .map_err(|e| format!("failed to issue request: {:?}", e))?;

            let vrf = client
                .sr25519_vrf_sign(&pk)
                .map_err(|e| format!("failed to issue reqiest: {:?}", e))?;

            debug!("vrf={:x?}", vrf);

            Ok::<_, String>(())
        },
    )
    .exec();

    info!("TESTS FINISHED");
}

struct Test<'s, F> {
    name: &'s str,
    description: &'s str,
    logic: F,
}

impl<'s, F> Test<'s, F> {
    fn new(name: &'s str, description: &'s str, logic: F) -> Self {
        Self {
            name,
            description,
            logic,
        }
    }
}

impl<'s, E, F> Test<'s, F>
where
    E: std::fmt::Debug,
    F: Future<Output = Result<(), E>>,
{
    #[allow(dead_code)]
    async fn exec_fut(self) -> bool {
        let Self {
            name,
            description,
            logic,
        } = self;

        info!("[REMOTEE TEST {}]: START", name);
        debug!("[REMOTEE TEST {}]: {}", name, description);
        let result = logic.await;
        match result {
            Ok(_) => {
                info!("[REMOTEE TEST {}]: SUCCESS", name);
                true
            }
            Err(e) => {
                info!("[REMOTEE TEST {}]: FAILURE", name);
                error!("[REMOTEE TEST {}]: REASON: {:?}", name, e);
                false
            }
        }
    }
}

impl<'s, F, E> Test<'s, F>
where
    E: std::fmt::Debug,
    F: FnOnce() -> Result<(), E>,
{
    #[allow(dead_code)]
    fn exec(self) -> bool {
        let Self {
            name,
            description,
            logic,
        } = self;

        info!("[REMOTEE TEST {}]: START", name);
        debug!("[REMOTEE TEST {}]: {}", name, description);
        let result = tokio::task::block_in_place(|| (logic)());
        match result {
            Ok(_) => {
                info!("[REMOTEE TEST {}]: SUCCESS", name);
                true
            }
            Err(e) => {
                info!("[REMOTEE TEST {}]: FAILURE", name);
                error!("[REMOTEE TEST {}]: REASON: {:?}", name, e);
                false
            }
        }
    }
}
