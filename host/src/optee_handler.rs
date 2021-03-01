//! Handler that implements the HandleRequest trait that is used by the host service
//! to call to optee for the execution of commands using an especific TA instance.
//! This Handler is implemented here because this module has access to private functions that
//! do the weight lifting of performing invocations to OPTEE through the TEEC api, which is unsafe
//! Also by doing this the host client doesnt need to depend on obscure OPTEE bindings
use schnorrkel::{
    keys::{PublicKey, PUBLIC_KEY_LENGTH},
    sign::Signature,
};
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
        match request {
            RequestMethod::GenerateNew { seed } => {
                //convert seed to &str, pass the slice in bytes with prepended len (u64)
                // public key (output) is 32 bytes

                let mut out = [0u8; PUBLIC_KEY_LENGTH];

                let vec = match seed {
                    None => 0u64.to_le_bytes().to_vec(),
                    Some(seed) => {
                        let len = seed.len();

                        let mut vec = vec![0; 8 + len];
                        vec[..8].copy_from_slice(&len.to_le_bytes()[..]);
                        vec[8..].copy_from_slice(seed.as_bytes());

                        vec
                    }
                };
                let p0 = ParamTmpRef::new_input(&vec);

                let p1 = ParamTmpRef::new_output(&mut out[..]);

                let mut op = Operation::new(p0, p1, ParamNone, ParamNone);

                invoke_command(CommandId::GenerateNew.into(), &mut op)
                    .map_err(|e| e.to_string())?;
                let out = PublicKey::from_bytes(&out[..]).map_err(|e| e.to_string())?;

                Ok(RequestResponse::GenerateNew { public_key: out })
            }
            RequestMethod::GetPublicKeys => {
                //ok this prepare just the command id basically
                // but for the return we might need something else since we can't pass N keys back
                // because we need to preallocate the out buffer...
                // UNLESS we can share memory then we can allocate the memory in the TA
                // pass the pointer in the out slice
                // reinterpret it and extract the keys from there...
                todo!()
            }
            RequestMethod::SignMessage { public_key, msg } => {
                //the key is fixed lenght, so just dump it,
                // the msg prepent length before
                // signature (output) is 64 bytes
                todo!()
            }
        }
    }
}
