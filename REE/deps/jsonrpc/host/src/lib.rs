//! This crate implements the JSON-RPC protocol as described by `zkms-jsonrpc`

use std::net::ToSocketAddrs;

use futures::stream::{Stream, StreamExt};
use jsonrpc_http_server::jsonrpc_core::{BoxFuture, IoHandler, Result};

use host_common::{
    flume::{self, Receiver, Sender},
    zkms_common::schnorrkel::{PublicKey, Signature},
    RequestMethod, RequestResponse, ServiceRequest,
};
use zkms_jsonrpc::ZKMS;

/// Will start the JSON-RPC service as configured and return a list of incoming service requests
pub async fn start_service<E: Send + std::fmt::Debug + 'static>(
    addr: impl ToSocketAddrs,
) -> impl Stream<Item = std::result::Result<ServiceRequest<E>, E>> {
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

    tokio::task::spawn_blocking(move || async move {
        let server = jsonrpc_http_server::ServerBuilder::new(io)
            .start_http(&addr)
            .expect("unable to start rpc server");
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
    ) -> std::result::Result<RequestResponse, E> {
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

impl<E> ZKMS for RpcHandler<E>
where
    E: std::fmt::Debug + Send + 'static,
{
    fn generate_new(&self, seed: Option<String>) -> BoxFuture<Result<PublicKey>> {
        let sender = self.request_sender.clone();

        let fut = async move {
            match Self::submit(sender, RequestMethod::GenerateNew { seed })
                .await
                .expect("this call isn't supposed to error")
            {
                RequestResponse::GenerateNew { public_key } => Ok(public_key),
                _ => panic!("expected generatenew response"),
            }
        };

        Box::pin(fut)
    }

    fn get_public_keys(&self) -> BoxFuture<Result<Vec<PublicKey>>> {
        let sender = self.request_sender.clone();

        let fut = async move {
            match Self::submit(sender, RequestMethod::GetPublicKeys)
                .await
                .expect("this call isn't supposed to error")
            {
                RequestResponse::GetPublicKeys { keys } => Ok(keys),
                _ => panic!("expected getpublickeys response"),
            }
        };

        Box::pin(fut)
    }

    fn sign_message(&self, public_key: PublicKey, msg: Vec<u8>) -> BoxFuture<Result<Signature>> {
        let sender = self.request_sender.clone();

        let fut = async move {
            match Self::submit(sender, RequestMethod::SignMessage { public_key, msg })
                .await
                .expect("TODO: handle error")
            {
                RequestResponse::SignMessage { signature } => Ok(signature),
                _ => panic!("expected signmessage response"),
            }
        };

        Box::pin(fut)
    }
}
