use host_common::{channel, RequestMethod, RequestResponse, ServiceRequest};
use std::net::ToSocketAddrs;

use zkms_ductile::{RemoteKeystore, RemoteKeystoreResponse};

#[macro_use]
extern crate tracing;

/// Will start the ductile service as configured and return a list of incoming service requests
pub async fn start_service<E: Send + 'static>(
    addr: impl ToSocketAddrs,
) -> impl Stream<Item = Result<ServiceRequest<E>, E>> {
    let (tx, rx) = channel::mpsc::unbounded();

    let _ = tokio::task::spawn_blocking(move || {
        let listener = ductile::ChannelServer::bind(addr).expect("unable to bind server");

        //accept connections
        for (duct_tx, duct_rx, peer) in listener {
            let tx = tx.clone();
            info!(?peer, "NEW connection");

            //for every message from the peer
            while let Ok(req) = duct_rx.recv() {
                let duct_tx = duct_tx.clone();
                let tx = tx.clone();

                //prepare request for service
                debug!(request = ?req);
                let req = match translate_request(req) {
                    Err(resp) => {
                        //if there's an unsupported request we can reply early
                        // and move on to the next request
                        duct_rx.send(resp);
                        continue;
                    }
                    Ok(req) => req,
                };

                let (resp_tx, resp_rx) = channel::oneshot::channel();
                let service_request = ServiceRequest::new(req, Some(resp_tx));

                tokio::task::spawn(async move {
                    //send request to service
                    tx.send(service_request).await;

                    //wait for reply from service
                    let resp = resp_rx.await.expect("channel will not be canceled");
                    let resp = translate_response(resp);

                    //send response back to peer
                    debug!(response = ?resp);
                    duct_rx.send(resp);
                });
            }
        }
    });

    //receive requests for service (and stream them)
    rx
}

fn translate_request(request: RemoteKeystore) -> Result<RequestMethod, RemoteKeystoreResponse> {
    match request {
        RemoteKeystore::Sr25519PublicKeys(_) => {
            unimplemented!()
        }
        RemoteKeystore::Sr25519GenerateNew { id, seed } => {
            unimplemented!()
        }
        RemoteKeystore::Ed25519GenerateNew { id, seed } => {
            unimplemented!()
        }
        RemoteKeystore::EcdsaGenerateNew { id, seed } => {
            unimplemented!()
        }
        RemoteKeystore::HasKeys(_) => {
            unimplemented!()
        }
        RemoteKeystore::SignWith { id, key, msg } => {
            unimplemented!()
        }
        RemoteKeystore::Sr25519VrfSign {
            key_type,
            public,
            transcript_data,
        } => {
            unimplemented!()
        }
        RemoteKeystore::Ed25519PublicKeys(_) => {
            Err(RemoteKeystoreResponse::Ed25519PublicKeys(vec![]))
        }
        RemoteKeystore::EcdsaPublicKeys(_) => Err(RemoteKeystoreResponse::EcdsaGenerateNew(vec![])),
        RemoteKeystore::InsertUnknown { .. } => Err(RemoteKeystoreResponse::InsertUnknown(Err(()))),
        RemoteKeystore::SupportedKeys { .. } => Err(RemoteKeystoreResponse::SupportedKeys(Err(
            sp_keystore::Error::Unavailable,
        ))),
        RemoteKeystore::Keys(_) => Err(RemoteKeystoreResponse::Keys(Err(
            sp_keystore::Error::Unavailable,
        ))),
        RemoteKeystore::SignWithAny { .. } => Err(RemoteKeystoreResponse::SignWithAny(Err(
            sp_keystore::Error::Unavailable,
        ))),
        RemoteKeystore::SignWithAll { .. } => Err(RemoteKeystoreResponse::SignWithAll(Err(()))),
    }
}

fn translate_response<E: Send + 'static>(
    response: Result<RequestResponse, E>,
) -> RemoteKeystoreResponse {
    match response {
        Ok(response) => match response {
            RequestResponse::GenerateNew { public_key } => {}
            RequestResponse::GetPublicKeys { keys } => {}
            RequestResponse::SignMessage { signature } => {}
        },
        Err(_) => {
            todo!("handle error outside response")
        }
    }

    todo!()
}
