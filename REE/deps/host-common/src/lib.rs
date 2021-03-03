//! This crate is for common types between `host` crates, such as
//! `host`
//! `host_jsonrpc`
//!
//! It contains the definition that all services need to adhere to be valid for the host application

#[macro_use]
extern crate log;

use flume::Sender;
use futures::stream::Stream;

pub use flume;
pub use zkms_common::{self, RequestMethod, RequestResponse};

/// Type alias for the channel to send the result of the request to
pub type ResponseSender<E> = Sender<Result<RequestResponse, E>>;

/// Type returned by the service
///
/// Contains the requested method and a way to reply back (if a response is expected)
pub struct ServiceRequest<E> {
    pub method: RequestMethod,
    channel: Option<ResponseSender<E>>,
}

/// Interface used to describe a service
///
/// The service shall produce a stream of `ServiceRequest`
pub trait REEService
where
    Self: Stream<Item = Result<ServiceRequest<Self::ServiceError>, Self::ServiceError>>,
{
    type ServiceError;
}

impl<S, E> REEService for S
where
    S: Stream<Item = Result<ServiceRequest<E>, E>>,
{
    type ServiceError = E;
}

impl<E> ServiceRequest<E> {
    /// Create a new `ServiceRequest` with optional `ResponseSender`
    pub fn new(method: RequestMethod, channel: impl Into<Option<ResponseSender<E>>>) -> Self {
        Self {
            method,
            channel: channel.into(),
        }
    }

    /// Consume the request and reply with the given response if a response was expected
    ///
    /// If no response is needed then this method shouldn't be called
    pub async fn reply(self, response: Result<RequestResponse, E>) {
        if let Some(chan) = self.channel {
            trace!("reply was expected, sending to service...");
            if let Err(e) = chan.send_async(response).await {
                warn!("unable to send response to service! err={:?}", e);
            }
        }
    }
}
