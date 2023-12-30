# chamber: A self-hostable SecretOps service.

Do you have NIH syndrome? Me too, which is why I made this web service so I can avoid the complexity of having to use Hashicorp Vault.

## Usage
The easiest way to start using Chamber is via the CLI. You will need to install it using the following:
```bash
cargo install chamber-cli
```
You'll want to then set the URL of your Chamber instance using `chamber website set [VALUE]`.

Initially when you load up the web service, a root key will be auto generated for you that you can find in the logs. You will need to use this key to unseal the web service using `chamber unseal [VALUE]`. 

Once this is done, you can then generate a `chamber.bin` file using `chamber keygen` and use `chamber upload` to upload the new keyfile to the web service to reset your seal key (and cryptographic key)!

### Deployment to Shuttle 
To deploy this as a Shuttle service, run the following:
```bash
cargo shuttle init --from joshua-mo-143/chamber --subfolder chamber-server
```

You will probably want to use `chamber keygen` to generate a new keyfile and include it in your project root. This will allow you to keep your keyfile persistent across deployments. `shuttle-persist` support is planned to make it easier to persist your keyfiles.

### Dockerfile Deployment 
[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/template/VS9MIg?referralCode=zFpLxH)
A dockerfile has been added for your convenience.

The dockerfile takes the `DATABASE_URL` and `PORT` environment variables.

## Features
- Store your secrets in a self-hostable web server
- Lock and unlock your instance using root key 
- Encrypt your secrets using AES-256-GCM 
- IAM system that allows you to lock secrets by role whitelist and power level
- Categorise your secrets easily using tags
- Postgres backend (multiple backends to be supported in future)
- Written in Rust 

## Future short-term features
- Secrets will be encrypted and signed

## Long(er) Term Roadmap
- Logging/tracing
- SDK

## How Chamber works
There are several moving parts to Chamber:
- A web server that has can be unlocked and locked as required
- A command-line interface that serves as the current primary way to interact with a Chamber server
- The core (which holds methods for storing data, encryption and decryption, and other misc things)

## How secure is Chamber?
Chamber currently uses the `ring` crate, which is under the Rustls stack. A security audit was done in 2020, which you can find more about [here](https://github.com/rustls/rustls/blob/main/audit/TLS-01-report.pdf). Only four minor findings were found and were either security recommendations or noteworthy (but **not exploitable!**) issues.

Secrets are currently encrypted through AES-256-GCM. This provides *reasonably* good security where personal projects are concerned or where Chamber may be used for small scale production projects. We also utilise a nonce sequence based on an incrementing u64 starting from the number 1, which should be reasonably good enough for most cases. However, no form of signing is currently implemented. This is due to be resolved before v0.3.0 release.

Efforts have been made to ensure that the nonce wrapper and all related structs do not implement Clone to make sure that data is not duplicated unnecessarily. `zeroize` will also be used to ensure that the freed memory clears itself.
