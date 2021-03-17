use futures::future::Future;

use futures::future::TryFutureExt;
use jsonrpc_core_client::transports::http::connect;
use schnorrkel::{PublicKey, Signature};
use zkms_jsonrpc::ZKMSClient;

pub async fn execute_tests(addr: impl std::net::ToSocketAddrs) {
    info!("Connecting testing client...");
    let client = get_client(
        addr.to_socket_addrs()
            .expect("unable to construct address list")
            .next()
            .expect("no valid address provided"),
    )
    .await;

    info!("TESTS STARTING");

    Test::new(
        "generateNew 00",
        "generate new keypair and return a public key; no seed",
        async {
            let key = client
                .generate_new(None)
                .await
                .map_err(|e| format!("failed to issue request: {:?}", e))?;
            PublicKey::from_bytes(&key[..])
                .map_err(|e| format!("public key was not valid: {:?}", e))?; //verify that key is valid
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
                .generate_new(None)
                .await
                .map_err(|e| format!("failed to issue request: {:?}", e))?;
            let public_key = PublicKey::from_bytes(&key[..])
                .map_err(|e| format!("public key was not valid: {:?}", e))?; //verify that key is valid

            const MSG: &[u8] = "francesco@zondax.ch".as_bytes();

            let sign = client
                .sign_message(key, MSG.to_vec())
                .await
                .map_err(|e| format!("failed to issue request: {:?}", e))?;

            let sign = Signature::from_bytes(&sign[..])
                .map_err(|e| format!("signature could not be deserialized: {:?}", e))?;

            public_key
                .verify_simple(b"zondax", MSG, &sign)
                .map_err(|e| format!("signature was not valid: {:?}", e))
        },
    )
    .exec_fut()
    .await;

    info!("TESTS FINISHED");
}

async fn get_client(addr: std::net::SocketAddr) -> ZKMSClient {
    let addr = format!("http://{}", addr);

    connect::<ZKMSClient>(&addr)
        .await
        .expect("unable to connect to jsonrpc server")
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
