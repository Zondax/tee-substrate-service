//! Handler that implements the HandleRequest trait that is used by the host service
//! to call to optee for the execution of commands using an especific TA instance.
//! This Handler is implemented here because this module has access to private functions that
//! do the weight lifting of performing invocations to OPTEE through the TEEC api, which is unsafe
//! Also by doing this the host client doesnt need to depend on obscure OPTEE bindings
use zkms_common::{HandleRequest, RequestMethod, RequestResponse};

use optee_common::CommandId;

use crate::invoke_command;

use zondee_teec::wrapper::{Operation, ParamNone, ParamTmpRef};

#[derive(Default)]
pub struct Handler {}

impl HandleRequest for Handler {
    fn process_request(&self, request: RequestMethod) -> Result<RequestResponse, String> {
        //convert items from RequestMethod
        // to something that optee_common understands
        // and `invoke_command`
        //
        //invoke_command(CommandId::Mul as _, &mut op).map_err(|e| e.to_string())?;
        todo!()
    }
}
