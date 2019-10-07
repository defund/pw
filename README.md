### Usage
* `pw` list all entries
* `pw add` add new entry
* `pw del` delete entry
* `pw <name>` generate password for entry; `name` can refer to either an entry's name or shortened name

> ##### Can I still create entries with names `add` and `del`?
> You can, but you won't be able to reference it to generate a password. Suck it up :p

All data is stored in the user's home directory under the filename `.pw.json` by default. You can overwrite the path with the `PW_PATH` environment variable.

> ##### How do I stop people from messing with my data?
> Each entry is tamper-resistant (see crypto), but that won't stop somebody from deleting entries, duplicating entries, or malforming the data in general. I'd suggest setting read-only permissions for your user, since list and generate are read-only operations. When you want to add or delete entries, just run as superuser.

### Crypto
Each entry is individually encrypted and authenticated with ChaCha20/Poly1305. The key is derived with Argon2id from the master key and a randomly generated 16-byte salt. The plaintext is the UTF-8 encoding of the password. The additionally associated data consists of the entry name, shortened name, and extra data. The exact construction is `SHA256(name) || SHA256(short) || SHA256(extra)` to avoid collisions.

### Building
```
cargo +nightly build --release
```
The binary is located at `target/release/pw`.
