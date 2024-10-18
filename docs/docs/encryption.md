# Encryption

Rwf uses [AES-128](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard) for encrypting user [sessions](../controllers/sessions) and private [cookies](../controllers/cookies). The same functionality is available through the [`rwf::crypto`](https://docs.rs/rwf/latest/rwf/crypto/index.html) module to encrypt and decrypt arbitrary data.

## Encrypt data

To encrypt data using AES-128 and the application secret key, you can use the [`encrypt`](https://docs.rs/rwf/latest/rwf/crypto/fn.encrypt.html) function, for example:

```rust
use rwf::crypto::encrypt;

let data = serde_json::json!({
    "user": "test",
    "password": "hunter2"
});

// JSON is converted into a byte array.
let data = serde_json::to_vec(&data).unwrap();

// Data is encrypted with AES.
let encrypted = encrypt(&data).unwrap();
```

Any kind of data can be encrypted, as long as it's serializable to an array of bytes. Serialization can typically be achieved by using [`serde`](https://docs.rs/serde/latest/serde/).

Encryption produces a base64-encoded UTF-8 string. You can save this string in the database or send it via an insecure medium like email.

## Decrypt data

To decrypt the data, you can call the [`decrypt`](https://docs.rs/rwf/latest/rwf/crypto/fn.decrypt.html) function on the string produced by the `encrypt` function. The decryption algorithm will automatically convert the base64-encoded string to bytes and decrypt those bytes using the secret key, for example:

```rust
use rwf::crypto::decrypt;

let decrypted = decrypt(&encrypted).unwrap();
let json = serde_json::from_slice(&decrypted).unwrap();

assert_eq!(json["user"], "test");
```
