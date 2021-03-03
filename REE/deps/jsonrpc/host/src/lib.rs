//! This crate implements the JSON-RPC protocol as described by `zkms-jsonrpc`

use std::net::ToSocketAddrs;

#[macro_use]
extern crate log;

use futures::{
    stream::{Stream, StreamExt},
    Future, FutureExt,
};
use jsonrpc_http_server::jsonrpc_core::{
    BoxFuture, Error as RpcError, IoHandler, Result as RpcResult,
};

use host_common::{
    flume::{self, Receiver, Sender},
    zkms_common::schnorrkel::{PublicKey, Signature},
    RequestMethod, RequestResponse, ServiceRequest,
};
use zkms_jsonrpc::ZKMS;

/// Will start the JSON-RPC service as configured and return a list of incoming service requests
pub fn start_service<E: Send + std::fmt::Debug + 'static>(
    addr: impl ToSocketAddrs,
) -> impl Stream<Item = Result<ServiceRequest<E>, E>> {
    //get iohandler for jsonrcp
    let mut io = IoHandler::new();

    //configure service handler
    let (handler, rx) = RpcHandler::new();
    io.extend_with(handler.to_delegate());

    let addr = addr
        .to_socket_addrs()
        .expect("unable construct address list")
        .next()
        .expect("no valid address provided");

    let _ = std::thread::spawn(move || {
        let server = jsonrpc_http_server::ServerBuilder::new(io)
            .rest_api(jsonrpc_http_server::RestApi::Unsecure)
            .start_http(&addr)
            .expect("unable to start rpc server");

        info!("starting JSONRPC server at : http://{:}", addr);
        server.wait();
    });

    rx.into_stream().then(|req| async move { Ok(req) })
}

struct RpcHandler<E> {
    request_sender: Sender<ServiceRequest<E>>,
}

impl<E> RpcHandler<E>
where
    E: Send + std::fmt::Debug,
{
    pub fn new() -> (RpcHandler<E>, Receiver<ServiceRequest<E>>) {
        let (tx, rx) = flume::unbounded();

        let handler = Self { request_sender: tx };
        (handler, rx)
    }
}

impl<E: 'static> RpcHandler<E> {
    async fn submit(
        sender: Sender<ServiceRequest<E>>,
        request: RequestMethod,
    ) -> Result<RequestResponse, E> {
        let (tx, rx) = flume::bounded(1);

        let request = ServiceRequest::new(request, tx);
        sender
            .send_async(request)
            .await
            .expect("did the handler die? TODO: not panic but error gracefully");

        rx.into_recv_async()
            .await
            .expect("couldn't receive, did the handler drop the tx? TODO: error gracefully")
    }
}

impl<E: std::fmt::Debug + 'static> RpcHandler<E> {
    fn generate_new_impl(&self, seed: Option<String>) -> impl Future<Output = PublicKey> {
        let sender = self.request_sender.clone();

        async move {
            match Self::submit(sender, RequestMethod::GenerateNew { seed })
                .await
                .expect("this call isn't supposed to error")
            {
                RequestResponse::GenerateNew { public_key } => public_key,
                _ => panic!("expected generatenew response"),
            }
        }
    }

    fn get_public_keys_impl(&self) -> impl Future<Output = Vec<PublicKey>> {
        let sender = self.request_sender.clone();

        async move {
            match Self::submit(sender, RequestMethod::GetPublicKeys)
                .await
                .expect("this call isn't supposed to error")
            {
                RequestResponse::GetPublicKeys { keys } => keys,
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
    E: std::fmt::Debug + Send + 'static,
{
    fn generate_new(&self, seed: Option<String>) -> BoxFuture<RpcResult<PublicKey>> {
        info!("generate new requested");
        Box::pin(self.generate_new_impl(seed).map(|k| Ok(k)))
    }

    fn get_public_keys(&self) -> BoxFuture<RpcResult<Vec<PublicKey>>> {
        info!("get public keys requested");
        Box::pin(self.get_public_keys_impl().map(|k| Ok(k)))
    }

    fn sign_message(&self, public_key: PublicKey, msg: Vec<u8>) -> BoxFuture<RpcResult<Signature>> {
        info!("sign requested");
        Box::pin(
            self.sign_message_impl(public_key, msg)
                .map(|result| result.map_err(|e| RpcError::invalid_params(format!("{:?}", e)))),
        )
    }
}
