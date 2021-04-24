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
            RequestMethod::GenerateNew { algo, key_type } => {
                //pass the algo and key type
                // prepare output for public key

                let algo = crate::utils::convert_crypto_algo_to_optee(algo);
                let mut out = vec![0u8; algo.pubkey_len()];

                let p0 = {
                    let mut v = vec![0; 1 + 4];
                    algo.serialize_fixed(&mut v[..1]).unwrap();
                    key_type.serialize_fixed(&mut v[1..]).unwrap();
                    v
                };
                let p0 = ParamTmpRef::new_input(p0.as_slice());

                let p1 = ParamTmpRef::new_output(&mut out[..]);

                let mut op = Operation::new(p0, p1, ParamNone, ParamNone);

                invoke_command(CommandId::GenerateNew.into(), &mut op)
                    .map_err(|e| e.to_string())?;

                Ok(RequestResponse::GenerateNew { public_key: out })
            }
            RequestMethod::GetPublicKeys { algo, key_type } => {
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

                // Yeah this for now does nothing, until I implement the bigger size required

                let mut out = 0u64.to_le_bytes();
                let vec = 0u64.to_le_bytes();

                let p0 = ParamTmpRef::new_input(&vec);
                let p1 = ParamTmpRef::new_output(&mut out[..]);

                let mut op = Operation::new(p0, p1, ParamNone, ParamNone);
                invoke_command(CommandId::GetKeys.into(), &mut op).map_err(|e| e.to_string())?;

                Ok(RequestResponse::GetPublicKeys { keys: Vec::new() })
            }
            RequestMethod::SignMessage {
                algo,
                key_type,
                public_key,
                msg,
            } => {
                //send algo and key type
                // key might have different size so prepend length
                // and also for msg

                let algo = crate::utils::convert_crypto_algo_to_optee(algo);
                let mut out = vec![0u8; algo.signature_len()];

                let vec = {
                    let mut vec = vec![0u8; 1 + 4];
                    algo.serialize_fixed(&mut vec[..1]).unwrap();
                    key_type.serialize_fixed(&mut vec[1..]).unwrap();
                    vec.append(&mut public_key.as_slice().serialize().unwrap());
                    vec.append(&mut msg.as_slice().serialize().unwrap());
                    vec
                };
                let p0 = ParamTmpRef::new_input(&vec);

                let p1 = ParamTmpRef::new_output(&mut out[..]);

                let mut op = Operation::new(p0, p1, ParamNone, ParamNone);

                invoke_command(CommandId::SignMessage.into(), &mut op)
                    .map_err(|e| e.to_string())?;

                Ok(RequestResponse::SignMessage { signature: out })
            }
            RequestMethod::HasKeys { pairs } => {
                //prepent lenght of entire set
                // each pair has key type (fixed size) and public key, which might have varying lenght
                let p0 = pairs
                    .into_iter()
                    .map(crate::utils::convert_haskeys_to_optee)
                    .collect::<Vec<_>>()
                    .serialize()
                    .unwrap();
                let p0 = ParamTmpRef::new_input(&p0);

                //prepare output for a simple boolean
                // 0 = false
                // 1 = true

                let mut out = [0];
                let p1 = ParamTmpRef::new_output(&mut out[..]);

                let mut op = Operation::new(p0, p1, ParamNone, ParamNone);

                invoke_command(CommandId::HasKeys.into(), &mut op).map_err(|e| e.to_string())?;

                Ok(RequestResponse::HasKeys { all: out[0] == 1 })
            }
            RequestMethod::VrfSign {
                key_type,
                public_key,
                transcript_data,
            } => {
                let p0 = {
                    let mut v = vec![0; 4 + 32];
                    key_type.serialize_fixed(&mut v[..4]).unwrap();
                    public_key.0.serialize_fixed(&mut v[4..]).unwrap();

                    //serialize transcript data
                    v.append(&mut transcript_data.serialize().unwrap());
                };
                let p0 = ParamTmpRef::new_input(&p0);

                let mut out = [0; 64];
                let p1 = ParamTmpRef::new_output(&mut out[..]);

                let mut op = invoke_command(CommandId::VrfSign.into(), &mut op)
                    .map_err(|e| e.to_string())?;

                Ok(RequestResponse::VrfSign { signature: todo!() })
            }
        }
    }
}
