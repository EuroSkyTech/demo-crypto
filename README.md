# MPDS Crypto Demo

This demo demonstrates a very simple to understand (hopefully), fully rust-based Webauthn client / server demo that derives P256 keys (with the public key encoded as `base58btc` multibase) from the Webauthn [PRF](https://github.com/w3c/webauthn/wiki/Explainer:-PRF-extension) extension.

Key derivation works like this:

- Generate a PRF with empty evals
- Generate a HKDF salt with a domain seperation value
- Extract a PRK from the PRF
- Iterate for a small number of rounds until we find a ECDSA keypair (or give up with an error):
  - Generate OKM using the `round` number, user `did` and `key-type` ("`signing`") info
  - Create a PKCS8 secret key in DER notion and generate an P256 keypair in software (using ring)

In the demo, the resulting public key is encoded to the appropriate multibase and shown in the HTML. In a real-world application the private key would require appropriate safekeeping (e.g. [importing it](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/importKey) on the web platform as non-extractable).

Testing it locally: run "just dev", go the https://localhost:9999 and accept the self-signed certificate

General notes:

- It only works for clients who have the ability and willingness to rely solely on Webauthn Passkeys for authentication (this requires [iOS 18+](https://developer.apple.com/documentation/safari-release-notes/safari-18-release-notes) and is not yet widely supported on Android)
- When adding a web client to post / sign as well, this currently requires setting up related origins (via .well-known extensions) and the presence of the web client on registration; it also requires the client to be trustworthy
- Mac specific: the demo has some user presence verification issues in clamshell mode

Out of scope:

- Actually useful ATProto operations such as signing content or PLC management of the resulting keys
- "Rotation" key derivation for PLC operations (although this is trivial)
- On-device private key backup
