# chamber: A self-hostable SecretOps service.

Do you have NIH syndrome? Me too, which is why I made this web service so I can avoid the complexity of having to use Hashicorp Vault.

## Usage
The easiest way to start using Chamber is via the CLI. Currently there is no package published on crates.io, so you will need to install it using the following:
```bash
cargo install chamber
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
A dockerfile has been added for your convenience.

The dockerfile takes the `DATABASE_URL` and `PORT` environment variables.

## Features
- Store your secrets in a self-hostable web server
- Lock and unlock using 
- Encrypt your secrets using AES-256-GCM 
- IAM system that allows you to lock secrets by role whitelist and power level
- Categorise your secrets easily using tags
- Postgres backend (multiple backends to be supported in future)
- Written in Rust 

## Roadmap for v0.1.0
- [x] CLI
	- [x] Config
	- [x] Unseal
	- [x] Sign in
	- [x] Create/delete users 
	- [x] Set/get secrets
	- [x] Remove secrets
	- [x] Table output
	- [x] Generate a pre-made cryptokey file to use in web service
	- [x] Basic key rotation (to be improved)
- [x] Web service
	- [x] JWT auth
	- [x] Unsealing route
	- [x] Set/get/remove secrets
	- [x] Secrets access-level requirement and role whitelist
	- [x] Create/delete users
	- [x] User roles/access level
	- [x] Persisting cryptokey through file
- [x] Postgres database
	- [x] AES-256-GCM en/decryption
	- [x] Seal/unseal using API key
	- [x] Basic IAM system
	- [x] Secrets grouping

## Long(er) Term Roadmap
- Logging/tracing
- SDK

## How Chamber works
There are several moving parts to Chamber:
- A web server that has can be unlocked and locked as required
- A command-line interface that serves as the current primary way to interact with a Chamber server
- The core (which holds methods for storing data, encryption and decryption, and other misc things)
