# Self-signed store certificates (Test strategy #6's transport-assurance pair)

The two `.der` pairs here are self-signed leaf certificates used by
`tests/steam.rs` to stand up loopback TLS listeners for the transport-assurance
legs: `partner.steam-api.com.crt.der` carries the name the production client
verifies, and `some-other-name.example.crt.der` carries another. The client in
those legs runs the production trust store (webpki roots only), so neither
certificate may ever chain — the fixtures exist precisely so the failure is
*chain-building*, not "the peer speaks no TLS". They are test fixtures; they
certify nothing and must never be used anywhere else.

Regenerate (ECDSA P-256, 20 years — `notAfter` is November 2046, which is a
scheduled test-fixture refresh, not an incident):

```sh
NAME=partner.steam-api.com   # or some-other-name.example
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -noenc \
  -days 7300 -keyout "$NAME.key.pem" -out "$NAME.crt.pem" \
  -subj "/CN=$NAME" -addext "subjectAltName=DNS:$NAME" \
  -addext "basicConstraints=critical,CA:FALSE"
openssl pkey -in "$NAME.key.pem" -outform DER -out "$NAME.key.pkcs8.der"
openssl x509 -in "$NAME.crt.pem" -outform DER -out "$NAME.crt.der"
rm "$NAME.key.pem" "$NAME.crt.pem"
```
