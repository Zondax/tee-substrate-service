//! Handler that implements the HandleRequest trait that is used by the host service
//! to call to optee for the execution of commands using an especific TA instance.
//! This Handler is implemented here because this module has access to private functions that
//! do the weight lifting of performing invocations to OPTEE through the TEEC api, which is unsafe
//! Also by doing this the host client doesnt need to depend on obscure OPTEE bindings
use schnorrkel::{keys::PublicKey, sign::Signature};
use zkms_common::{HandleRequest, RequestError, RequestMethod, RequestResponse};

use optee_common::{CommandId, Deserialize, Serialize, SerializeFixed};

use crate::invoke_command;

use zondee_teec::wrapper::{Operation, ParamNone, ParamTmpRef};

#[derive(Default)]
pub struct Handler {}

impl HandleRequest for Handler {
    fn process_request(&self, request: RequestMethod) -> Result<RequestResponse, RequestError> {
        //convert items from RequestMethod
        // to something that optee_common understands
        // and `invoke_command`
        match request {
            RequestMethod::GenerateNew { seed } => {
                //convert seed to &str, pass the slice in bytes with prepended len (u64)
                // public key (output) is 32 bytes

                let mut out = vec![0u8; PublicKey::len()];

                let vec = match seed {
                    None => "".serialize().unwrap(),
                    Some(seed) => seed.as_str().serialize().unwrap(),
                };
                let p0 = ParamTmpRef::new_input(&vec);

                let p1 = ParamTmpRef::new_output(&mut out[..]);

                let mut op = Operation::new(p0, p1, ParamNone, ParamNone);

                invoke_command(CommandId::GenerateNew.into(), &mut op)
                    .map_err(|e| e.to_string())?;
                let out = PublicKey::deserialize(&out[..]).map_err(|e| e.to_string())?;

                Ok(RequestResponse::GenerateNew { public_key: out })
            }
            RequestMethod::GetPublicKeys => {
                //ok this prepare just the command id basically
                // but for the return we might need something else since we can't pass N keys back
                // because we need to preallocate the out buffer...
                // UNLESS we can share memory then we can _allocate the memory in the TA_ (what am I thinking?)
                // pass the pointer in the out slice
                // reinterpret it and extract the keys from there...
                // ig we can make 2 request? 1 to retrieve number of public keys
                // and the other one to actually get the data
                //
                // turns out the spec agrees with me, a bit. the size paramenter of the memref
                // if bigger should be interpreted as a request for a bigger buffer, so in this case
                // we'd reallocate a bigger buffer of the specified size and pass it again, kinda like asking
                // how many keys we have..

                let mut out = 0u64.to_le_bytes();
                let vec = 0u64.to_le_bytes();

                let p0 = ParamTmpRef::new_input(&vec);
                let p1 = ParamTmpRef::new_output(&mut out[..]);

                let mut op = Operation::new(p0, p1, ParamNone, ParamNone);
                invoke_command(CommandId::GetKeys.into(), &mut op).map_err(|e| e.to_string())?;

                Ok(RequestResponse::GetPublicKeys { keys: Vec::new() })
            }
            RequestMethod::SignMessage { public_key, msg } => {
                //the key is fixed lenght, so just dump it,
                // the msg prepent length before
                // signature (output) is 64 bytes
                let mut out = vec![0u8; Signature::len()];

                let vec = {
                    let mut vec = vec![0u8; PublicKey::len()];
                    public_key.serialize_fixed(&mut vec).unwrap();
                    vec.append(&mut msg.as_slice().serialize().unwrap());
                    vec
                };
                let p0 = ParamTmpRef::new_input(&vec);

                let p1 = ParamTmpRef::new_output(&mut out[..]);

                let mut op = Operation::new(p0, p1, ParamNone, ParamNone);

                invoke_command(CommandId::SignMessage.into(), &mut op)
                    .map_err(|e| e.to_string())?;
                let out = Signature::deserialize(&out[..]).map_err(|e| e.to_string())?;

                Ok(RequestResponse::SignMessage { signature: out })
            }
        }
    }
}