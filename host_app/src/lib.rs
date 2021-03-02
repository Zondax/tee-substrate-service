//! This crate contains the glue to drive a `host-common::REEService` and feed
//! the request into a `zkms-common::HandleRequest``
//!
//! It effectively allows dependency injection, both from the service and the request handler

#![deny(
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications
)]

#[macro_use]
extern crate log;

use futures::stream::StreamExt;
use host_common::REEService;
use zkms_common::HandleRequest;

pub async fn start_service(
    service: impl REEService<ServiceError = String> + 'static,
    handler: impl HandleRequest + 'static,
) {
    futures::pin_mut!(service);
    //retrieve next request from service
    // whilst we could go concurrent it would be useless as the handler is still blocking
    while let Some(item) = service.next().await {
        match item {
            Err(e) => error!("failed to retrieve next item from service: {:?}", e),
            Ok(request) => {
                //pass request to handler
                let response = handler
                    .process_request(request.method.clone())
                    .map_err(|s| format!("Error processing request: {}", s));

                debug!(
                    "processed request={:?}; response={:?}",
                    request.method, response
                );

                //reply to request
                request.reply(response).await
            }
        }
    }
}
