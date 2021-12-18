# Remote Signer with OPTEE

This project's purpose is to allow safe cryptographic key storage and use for on-cloud blockchain nodes.

It uses the [OPTEE Framework](https://github.com/Zondax/tee-base).


## How to use

After having [built qemu with optee support](https://github.com/sccommunity/rust-optee-trustzone-sdk/wiki/Getting-started-with-OPTEE-for-QEMU-ARMv8), a couple of environment variables are still needed,
alternatively, if you have the dependencies already and you set `QEMU_V8` and `OPTEE`, you can `make qemu` and it will build it for you.

Depending on what kind of ARM you have setup QEMU for, either set `QEMU_V8` or `QEMU_V7` environment variables.
Then proceed to `make deps` to install the required rust tooling.

Afterwards, set `OPTEE` to the path of your QEMU installation.
Lastly, set `SHARED_FOLDER` to the folder that you want to mount in QEMU to share files between your system and the VM.

To run, simple `make run` (or `run-debug` for added debug arguments)
