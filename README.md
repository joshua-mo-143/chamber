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
Please refer to the [SECURITY.md](./SECURITY.md) file for a full explanation.

For the TL;DR: This service is reasonably secure if you are using it for small scale projects. Work is being done to enhance the security as outlined in the aforementioned file. Expect the API to be unstable.

