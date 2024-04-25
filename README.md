# chamber: A self-hostable SecretOps service.

Do you have NIH syndrome? Me too, which is why I made this web service so I can avoid the complexity of having to use Hashicorp Vault.

## Usage
The easiest way to start using Chamber is via the CLI. You will need to install it using the following:
```bash
cargo install chamber-cli
```
You'll want to then set the URL of your Chamber instance using `chamber website set [VALUE]`.

Initially when you load up the web service, a root key will be auto generated for you or you can genrate a keyfile using `chamber keygen` that will then get put into your web service and persisted. You will need to use this key to unseal the web service using `chamber unseal [VALUE]`. 

Once this is done, you can then generate a `chamber.bin` file using `chamber keygen` and use `chamber upload` to upload the new keyfile to the web service to reset your seal key (and cryptographic key)!

### Deployment to Shuttle 
To deploy this as a Shuttle service, run the following:
```bash
cargo shuttle init --from joshua-mo-143/chamber --subfolder chamber-server
```

Assuming you have the CLI, you will want to use `chamber keygen` to generate a key file so you can include it in your deployments. Your keyfile will be saved using `shuttle-persist`.

### Non Shuttle Deployment
This is being worked on!

## Features
- Store your secrets in a self-hostable web server
- Lock and unlock your instance using root key
- Encrypt your secrets using AES-256-GCM 
- Signed using ED25519
- IAM system that allows you to lock secrets by role whitelist and power level
- Categorise your secrets easily using tags
- Postgres backend (multiple backends to be supported in future)
- Written in Rust 

## In Progress
- Tracing

## Long(er) Term Roadmap
- Using stored SSH keys as an additional security measure for CLI access
- Expanding SDK
- Supporting non-Shuttle deployment

## How Chamber works
There are several moving parts to Chamber:
- A web server that can be unlocked and locked as required
- A command-line interface that serves as the current primary way to interact with a Chamber server
- The core (which holds methods for storing data, encryption and decryption, and other misc things)
- An SDK for interacting with a Chamber instance on a Rust application

## How secure is Chamber?
Please refer to the [SECURITY.md](./SECURITY.md) file for a full explanation.

The TL;DR:
- Encrypted via AES-256-GCM, signed with Ed25519
- Currently uses a naive nonce sequence implementation
- Users can only retrieve secrets that they have the correct tags and numeric access level for
- You can seal your instance when it's not required to keep it open
