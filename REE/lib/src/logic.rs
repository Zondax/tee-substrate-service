use zondee_teec::wrapper::{Operation, ParamNone, ParamTmpRef};

fn send(cmd_id: u32, msg: &[u8], out_len: usize) -> Result<Vec<u8>, u32> {
    let mut out = vec![0u8; out_len];

    let p0 = ParamTmpRef::new_input(&msg);
    let p1 = ParamTmpRef::new_output(&mut out[..]);

    let mut op = Operation::new(p0, p1, ParamNone, ParamNone);

    super::invoke_command(cmd_id, &mut op)?;

    Ok(out)
}

pub fn echo(msg: &[u8]) -> Result<bool, u32> {
    let out = send(0, msg, msg.len())?;

    Ok(out == msg)
}
