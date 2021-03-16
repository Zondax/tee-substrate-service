use futures::future::Future;

use jsonrpc_core_client::transports::http::connect;
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
        "generateNew",
        "generate new keypair and return a public key; no seed",
        async {
            match client.generate_new(None).await {
                Err(e) => {
                    error!("test failed with: {:?}", e);
                    false
                }
                Ok(_) => true,
            }
        },
    )
    .exec_fut()
    .await;

    info!("TESTS FINISHED");
}

async fn get_client(addr: std::net::SocketAddr) -> ZKMSClient {
    connect::<ZKMSClient>(&addr.to_string())
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

impl<'s, F: Future<Output = bool>> Test<'s, F> {
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
        info!(
            "[REMOTEE TEST {}]: {}",
            name,
            if result { "SUCCESS" } else { "FAILURE" }
        );

        result
    }
}

impl<'s, F: FnOnce() -> bool> Test<'s, F> {
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
        info!(
            "[REMOTEE TEST {}]: {}",
            name,
            if result { "SUCCESS" } else { "FAILURE" }
        );

        result
    }
}
