use std::net::ToSocketAddrs;

use host_common::{channel, CryptoAlgo, RequestMethod, RequestResponse, ServiceRequest};
use zkms_ductile::{RemoteKeystore, RemoteKeystoreResponse};

#[macro_use]
extern crate tracing;

/// Will start the ductile service as configured and return a list of incoming service requests
pub async fn start_service<E: Send + 'static>(
    addr: impl ToSocketAddrs + Send + 'static,
) -> impl futures::Stream<Item = Result<ServiceRequest<E>, E>> {
    use futures::SinkExt;

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
                let mut tx = tx.clone();

                //prepare request for service
                debug!(request = ?req);
                let req = match translate_request(req) {
                    Err(resp) => {
                        //if there's an unsupported request we can reply early
                        // and move on to the next request
                        duct_tx.send(resp);
                        continue;
                    }
                    Ok(req) => req,
                };

                let (resp_tx, resp_rx) = channel::oneshot::channel();
                let service_request = ServiceRequest::new(req.clone(), Some(resp_tx));

                tokio::task::spawn(async move {
                    //send request to service
                    tx.send(service_request).await;

                    //wait for reply from service
                    let resp = resp_rx.await.expect("channel will not be canceled");
                    let resp = translate_response(&req, resp);

                    //send response back to peer
                    debug!(response = ?resp);
                    duct_tx.send(resp);
                });
            }
        }
    });

    use futures::StreamExt;
    //receive requests for service (and stream them)
    rx.then(|s| async { Ok(s) })
}

fn translate_request(request: RemoteKeystore) -> Result<RequestMethod, RemoteKeystoreResponse> {
    use host_common::HasKeysPair;
    use zkms_ductile::KeystoreError as Error;

    match request {
        RemoteKeystore::Sr25519PublicKeys(id) => Ok(RequestMethod::GetPublicKeys {
            algo: CryptoAlgo::Sr25519,
            key_type: id.0,
        }),
        RemoteKeystore::Sr25519GenerateNew { id, seed: _ } => Ok(RequestMethod::GenerateNew {
            algo: CryptoAlgo::Sr25519,
            key_type: id.0,
        }),
        RemoteKeystore::Ed25519GenerateNew { id, seed: _ } => Ok(RequestMethod::GenerateNew {
            algo: CryptoAlgo::Ed25519,
            key_type: id.0,
        }),
        RemoteKeystore::EcdsaGenerateNew { id, seed: _ } => Ok(RequestMethod::GenerateNew {
            algo: CryptoAlgo::Ecdsa,
            key_type: id.0,
        }),
        RemoteKeystore::HasKeys(v) => {
            let pairs = v
                .into_iter()
                .map(|(public, id)| HasKeysPair {
                    key_type: id.0,
                    public_key: public,
                })
                .collect();
            Ok(RequestMethod::HasKeys { pairs })
        }
        RemoteKeystore::SignWith { id, key, msg } => {
            //not clear which id is the public key's...
            // the passed id or the one in the CryptoTypePublicPair??
            //
            // maybe the one in the pair represents the curve of the key? (ie ed25, sr25, ecds ?)
            // NOTE: DAMN I did find it! I was right!! so each module in core has a `CRYPTO_ID` which is the curve/algo :)
            warn!(
                req = "SignWith",
                ?id,
                ?key,
                ?msg,
                "temporarily unimplemented!"
            );
            Err(RemoteKeystoreResponse::SignWith(Err(Error::Other(
                "temporarily unimplemented".to_string(),
            ))))
        }
        RemoteKeystore::Sr25519VrfSign {
            key_type,
            public,
            transcript_data,
        } => Ok(RequestMethod::VrfSign {
            key_type: key_type.0,
            public_key: public,
            transcript_data,
        }),
        RemoteKeystore::Ed25519PublicKeys(_) => {
            Err(RemoteKeystoreResponse::Ed25519PublicKeys(vec![]))
        }
        RemoteKeystore::EcdsaPublicKeys(_) => Err(RemoteKeystoreResponse::EcdsaPublicKeys(vec![])),
        RemoteKeystore::InsertUnknown { .. } => Err(RemoteKeystoreResponse::InsertUnknown(Err(()))),
        RemoteKeystore::SupportedKeys { .. } => Err(RemoteKeystoreResponse::SupportedKeys(Err(
            Error::Unavailable,
        ))),
        RemoteKeystore::Keys(_) => Err(RemoteKeystoreResponse::Keys(Err(Error::Unavailable))),
        RemoteKeystore::SignWithAny { .. } => {
            Err(RemoteKeystoreResponse::SignWithAny(Err(Error::Unavailable)))
        }
        RemoteKeystore::SignWithAll { .. } => Err(RemoteKeystoreResponse::SignWithAll(Err(()))),
    }
}

fn translate_response<E: Send + 'static>(
    original_request: &RequestMethod,
    response: Result<RequestResponse, E>,
) -> RemoteKeystoreResponse {
    use zkms_ductile::KeystoreError as Error;
    use zkms_ductile::{crypto::Public, ecdsa, ed25519, sr25519};

    match response {
        Ok(response) => match response {
            RequestResponse::GenerateNew { public_key } => {
                let algo = match original_request {
                    RequestMethod::GenerateNew { algo, .. } => algo,
                    _ => todo!("handle non matching request-response...???"),
                };

                match algo {
                    CryptoAlgo::Sr25519 => {
                        let public = sr25519::Public::from_slice(&public_key);

                        RemoteKeystoreResponse::Sr25519GenerateNew(Ok(public))
                    }
                    CryptoAlgo::Ed25519 => {
                        let public = ed25519::Public::from_slice(&public_key);

                        RemoteKeystoreResponse::Ed25519GenerateNew(Ok(public))
                    }
                    CryptoAlgo::Ecdsa => {
                        let public = ecdsa::Public::from_slice(&public_key);

                        RemoteKeystoreResponse::EcdsaGenerateNew(Ok(public))
                    }
                }
            }
            RequestResponse::GetPublicKeys { keys } => {
                let algo = match original_request {
                    RequestMethod::GetPublicKeys { algo, .. } => algo,
                    _ => todo!("handle non matching request-response...???"),
                };

                match algo {
                    CryptoAlgo::Sr25519 => {
                        let keys = keys
                            .into_iter()
                            .map(|key| sr25519::Public::from_slice(&key))
                            .collect();

                        RemoteKeystoreResponse::Sr25519PublicKeys(keys)
                    }
                    CryptoAlgo::Ed25519 => {
                        let keys = keys
                            .into_iter()
                            .map(|key| ed25519::Public::from_slice(&key))
                            .collect();

                        RemoteKeystoreResponse::Ed25519PublicKeys(keys)
                    }
                    CryptoAlgo::Ecdsa => {
                        let keys = keys
                            .into_iter()
                            .map(|key| ecdsa::Public::from_slice(&key))
                            .collect();

                        RemoteKeystoreResponse::EcdsaPublicKeys(keys)
                    }
                }
            }
            RequestResponse::HasKeys { all } => RemoteKeystoreResponse::HasKeys(all),
            RequestResponse::SignMessage { signature } => {
                RemoteKeystoreResponse::SignWith(Ok(signature))
            }
            RequestResponse::VrfSign { signature } => {
                RemoteKeystoreResponse::Sr25519VrfSign(Ok(signature))
            }
        },
        Err(_) => {
            //probably need to wrap error so that RequestError -> sp_keystore::Error...
            // also might need to pass the request to properly construct response
            todo!("handle error outside response")
        }
    }
}
