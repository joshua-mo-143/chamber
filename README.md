# boulder: A self-hostable credentials manager web service.

Do you have NIH syndrome? Me too, which is why I made this web service so I can stop paying £2.99 a month for 1Password.

## Deployment
Currently boulder only supports deployment through Shuttle, but Dockerfile deployments should be getting added shortly.

## Features
- Seal/unseal and timed unseal feature
	- The web service initially starts out as locked and cannot be accessed unless you unseal it by using the master key given to you when you first create the web service.
- Basic IAM system
	- Create users and assign them roles. Lock credential usability either by role or by username.
	- Users can only grab credentials by either using the API with the custom header or the CLI, and must log in through the CLI first to be able to get a key.
- Entirely self sufficient - no database required!
	- An in-memory database is currently being used but file storage is on the roadmap, because you shouldn't hate yourself.
	- However, Postgres is aimed to be supported because everyone and their mom can get one these days and they are pretty cheap. Plus, persistent external storage is pretty good.

## Roadmap for v0.1.0
- [x] CLI
	- [x] Config
	- [x] Unseal
	- [x] Sign in
	- [x] Create/delete users 
	- [x] Set/get/remove secrets
- [x] Web service
	- [x] JWT auth
	- [x] Unsealing route
	- [x] Set/get/remove secrets
	- [x] Create/delete users
- [x] KV in-memory database
	- [x] AES-256-GCM en/decryption
	- [x] Seal/unseal using API key
	- [x] Basic IAM system
	- [ ] Secrets grouping
- [ ] Postgres database
	- [ ] AES-256-GCM en/decryption
	- [ ] Seal/unseal using API key
	- [ ] Basic IAM system
	- [ ] Secrets grouping

## Project Layout
- boulder-cli: The CLI.
- boulder-server: The web server.
- boulder-db: The in-memory database implementation.
