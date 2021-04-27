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
        async {
            client
                .sr25519_generate_new()
                .map_err(|e| format!("failed to issue request: {:?}", e))?;
            Ok::<_, String>(())
        },
    )
    .exec_fut()
    .await;

    Test::new(
        "signMessage 00",
        "sign a message and verify signature",
        async {
            let key = client
                .sr25519_generate_new()
                .map_err(|e| format!("failed to issue request: {:?}", e))?;

            const MSG: &[u8] = "francesco@zondax.ch".as_bytes();

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
    .exec_fut()
    .await;

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
        let result = (logic)();
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
