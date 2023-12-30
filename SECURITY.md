## How secure is Chamber?
Hopefully this document will give you some insight into how secure Chamber is and what measures have been taken to ensure sufficient safety.

### General
Chamber currently uses the `ring` crate, which is under the Rustls stack. A security audit was done in 2020, which you can find more about [here](https://github.com/rustls/rustls/blob/main/audit/TLS-01-report.pdf). Only four minor findings were found and were either security recommendations or noteworthy (but **not exploitable!**) issues.

Secrets are currently encrypted through AES-256-GCM. This provides *reasonably* good security where personal projects are concerned or where Chamber may be used for small scale production projects. We also utilise a nonce sequence based on an incrementing u64 counter. This should be reasonably good enough for most cases - however, it's acknowledged that this is not ideal.

Efforts have been made to ensure that all structs related to secrets encryption or decryption do not implement Clone to make sure that data is not duplicated unnecessarily. `zeroize` will also be used to ensure that the freed memory clears itself. Some further measures may need to be taken to ensure total safety. See page 6 of [this security report](https://cure53.de/pentest-report_rust-libs_2022.pdf) which outlines how and why zeroize may potentially not be fully safe. 

It should be noted that secrets don't get signed at the moment and are only encrypted. This is a short-term issue and will be fixed in the near future. Being able to use security-based software with peace of mind should not be compromised. 

Passwords are hashed using the `argon2` crate with the default Argon2id settings (19MB memory cost, 2 iterations and 1 degree of paralellism). This is in line with [the OWASP Cheat Sheet for password storage,](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) and as such is relatively sufficient. It should be noted that the default setup is one of their recommended configurations for the `Argon2id` algorithm (the default variant when you use `Argon2::default()`). 

### Key Rotation
Root keys, despite being quite long as a default, are currently stored in plaintext. This is slated to be changed so that it will be necessary to use a PGP key or a similar algorithm public key to unseal the Chamber instance. However, some investigative work is required to come to a conclusion on what should be used.

Should the Chamber instance be compromised, users who hold the root key are able to re-encrypt all given keys within a Chamber instance by re-uploading a `chamber.bin` file (requires the instance to be unsealed). It is recommended that you do this every 3 months or sooner. This reduces the chance that your cryptographic key will get stolen.

Additionally, you are required to log in as a user to be able to access any of the secrets. It is highly recommended to use the initial root user login to create secrets with the required role permissions and access level numbers, then delete the root user role. This will prevent users from attempting to log in as the default root user. Evidently this won't stop bad actors who have a root key from abusing the instance, but it will stop hijacked users from accessing secrets that would normally require a higher access level or role that they don't currently possess. 

Some work is planned on making this more modular so that users are not forced to reset their root key and cryptographic key at the same time.
