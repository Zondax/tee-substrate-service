//! This crate implements the JSON-RPC protocol as described by `zkms-jsonrpc`

use std::net::ToSocketAddrs;

#[macro_use]
extern crate log;

use futures::{
    stream::{Stream, StreamExt},
    Future, FutureExt, SinkExt,
};
use jsonrpc_http_server::jsonrpc_core::{BoxFuture, IoHandler, Result as RpcResult};

use host_common::{
    channel::{self, mpsc::UnboundedReceiver as Receiver, mpsc::UnboundedSender as Sender},
    zkms_common::schnorrkel::{PublicKey, Signature},
    RequestMethod, RequestResponse, ServiceRequest,
};
use zkms_jsonrpc::{
    ErrorWrapper as RpcError, PublicKey as ZKMSPublicKey, Signature as ZKMSSignature, ZKMS,
};

/// prepares the IoHandler with the Rpc impl
fn get_io_handler<E: Into<RpcError> + Send + 'static>() -> (IoHandler, Receiver<ServiceRequest<E>>)
{
    let mut io = IoHandler::new();

    //configure service handler
    let (handler, rx) = RpcHandler::new();
    io.extend_with(handler.to_delegate());

    (io, rx)
}

/// Will start the JSON-RPC service as configured and return a list of incoming service requests
pub async fn start_service<E: Send + Into<RpcError> + 'static>(
    addr: impl ToSocketAddrs,
) -> impl Stream<Item = Result<ServiceRequest<E>, E>> {
    //get iohandler for jsonrcp
    let (io, rx) = get_io_handler();

    let addr = addr
        .to_socket_addrs()
        .expect("unable construct address list")
        .next()
        .expect("no valid address provided");

    let _ = tokio::task::spawn_blocking(move || {
        let server = jsonrpc_http_server::ServerBuilder::new(io)
            .rest_api(jsonrpc_http_server::RestApi::Unsecure)
            .event_loop_executor(tokio::runtime::Handle::current())
            .threads(1)
            .start_http(&addr)
            .expect("unable to start rpc server");

        info!("starting JSONRPC server at : http://{:}", addr);
        server.wait();
    });

    rx.then(|req| async move { Ok(req) })
}

struct RpcHandler<E> {
    request_sender: Sender<ServiceRequest<E>>,
}

impl<E> RpcHandler<E>
where
    E: Send,
{
    pub fn new() -> (RpcHandler<E>, Receiver<ServiceRequest<E>>) {
        let (tx, rx) = channel::mpsc::unbounded();

        let handler = Self { request_sender: tx };
        (handler, rx)
    }
}

impl<E> RpcHandler<E>
where
    E: Send + 'static,
{
    async fn submit(
        mut sender: Sender<ServiceRequest<E>>,
        request: RequestMethod,
    ) -> Result<RequestResponse, E> {
        let (tx, rx) = channel::oneshot::channel();

        let request = ServiceRequest::new(request, tx);
        sender
            .send(request)
            .await
            .expect("did the handler die? TODO: not panic but error gracefully");

        rx.await
            .expect("couldn't receive, did the handler drop the tx? TODO: error gracefully")
    }

    fn generate_new_impl(
        &self,
        seed: Option<String>,
    ) -> impl Future<Output = Result<PublicKey, E>> {
        let sender = self.request_sender.clone();

        async move {
            match Self::submit(sender, RequestMethod::GenerateNew { seed }).await? {
                RequestResponse::GenerateNew { public_key } => Ok(public_key),
                _ => panic!("expected generatenew response"),
            }
        }
    }

    fn get_public_keys_impl(&self) -> impl Future<Output = Result<Vec<PublicKey>, E>> {
        let sender = self.request_sender.clone();

        async move {
            match Self::submit(sender, RequestMethod::GetPublicKeys).await? {
                RequestResponse::GetPublicKeys { keys } => Ok(keys),
                _ => panic!("expected getpublickeys response"),
            }
        }
    }

    fn sign_message_impl(
        &self,
        public_key: PublicKey,
        msg: Vec<u8>,
    ) -> impl Future<Output = Result<Signature, E>> {
        let sender = self.request_sender.clone();

        async move {
            match Self::submit(sender, RequestMethod::SignMessage { public_key, msg }).await? {
                RequestResponse::SignMessage { signature } => Ok(signature),
                _ => panic!("expected signmessage response"),
            }
        }
    }
}

impl<E> ZKMS for RpcHandler<E>
where
    E: Into<RpcError> + Send + 'static,
{
    fn generate_new(&self, seed: Option<String>) -> BoxFuture<RpcResult<ZKMSPublicKey>> {
        info!("generate new requested");
        Box::pin(self.generate_new_impl(seed).map(|r| {
            r.map(|key| key.to_bytes())
                .map_err(Into::into)
                .map_err(Into::into)
        }))
    }

    fn get_public_keys(&self) -> BoxFuture<RpcResult<Vec<ZKMSPublicKey>>> {
        info!("get public keys requested");
        //keys.into_iter().map(|k| k.to_bytes()).collect()
        Box::pin(self.get_public_keys_impl().map(|r| {
            r.map(|ks| ks.into_iter().map(|k| k.to_bytes()).collect())
                .map_err(Into::into)
                .map_err(Into::into)
        }))
    }

    fn sign_message(
        &self,
        public_key: ZKMSPublicKey,
        msg: Vec<u8>,
    ) -> BoxFuture<RpcResult<ZKMSSignature>> {
        info!("sign requested");

        let public_key = PublicKey::from_bytes(&public_key)
            .map_err(|e| host_common::zkms_common::RequestError::InternalError(e.to_string()))
            .map_err(|e| RpcError::from(e))
            .map_err(Into::into);

        match public_key {
            Ok(public_key) => Box::pin(self.sign_message_impl(public_key, msg).map(|r| {
                r.map(|s| s.to_bytes().to_vec())
                    .map_err(Into::into)
                    .map_err(Into::into)
            })),
            Err(e) => Box::pin(async move { return Err(e) }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use host_common::zkms_common::RequestError;

    fn get_test_handler() -> (
        jsonrpc_test::Rpc,
        impl Stream<Item = Result<ServiceRequest<RpcError>, RpcError>>,
    ) {
        let (handler, rx) = RpcHandler::new();
        let rpc = jsonrpc_test::Rpc::new(handler.to_delegate());

        (rpc, rx.then(|req| async move { Ok(req) }))
    }

    async fn handle_requests(
        rx: impl Stream<Item = Result<ServiceRequest<RpcError>, RpcError>> + Send + 'static,
    ) {
        tokio::spawn(async move {
            futures::pin_mut!(rx);
            while let Some(Ok(srv_req)) = rx.next().await {
                info!("got a request: {:?}", srv_req);
                let err = RequestError::from("dummy".to_string());
                srv_req.reply(Err(err.into())).await
            }
        });
    }

    #[tokio::test(core_threads = 2)]
    async fn generate_new() {
        env_logger::init();
        let (rpc, rx) = get_test_handler();

        handle_requests(rx).await;
        // std::mem::drop(rx);
        let result = rpc.request("generateNew", &());

        let result = result.contains("dummy");
        assert_eq!(result, true);
    }
}
