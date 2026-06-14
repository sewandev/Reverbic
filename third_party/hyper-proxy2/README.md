# hyper-proxy2 (vendored fork)

> **Reverbic note:** This is a vendored copy of `hyper-proxy2 0.1.0`, applied via
> `[patch.crates-io]` in the root `Cargo.toml`. The Rust source is unchanged; only
> `Cargo.toml` bumps the Rustls TLS stack (hyper-rustls 0.26 -> 0.27, tokio-rustls
> 0.25 -> 0.26, rustls-native-certs 0.7 -> 0.8) to drop the vulnerable
> `rustls-webpki 0.102.8` instance that this crate otherwise pulls through the
> librespot/Spotify dependency chain. The `ring` crypto provider is kept to avoid
> requiring a C toolchain. Remove this fork once upstream `hyper-proxy2` publishes a
> release on the patched line. See issue #169.

[![Travis Build Status](https://travis-ci.org/siketyan/hyper-proxy2.svg?branch=master)](https://travis-ci.org/siketyan/hyper-proxy2)
[![MIT licensed](https://img.shields.io/github/license/siketyan/hyper-proxy2)](./LICENSE-MIT.md)
[![crates.io](https://img.shields.io/crates/v/hyper-proxy2)](https://crates.io/crates/hyper-proxy2)

A proxy connector for [hyper][1] based applications.

[Documentation][3]

## Example

```rust
use std::error::Error;

use bytes::Bytes;
use headers::Authorization;
use http_body_util::{BodyExt, Empty};
use hyper::{Request, Uri};
use hyper_proxy2::{Proxy, ProxyConnector, Intercept};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   let proxy = {
      let proxy_uri = "http://my-proxy:8080".parse().unwrap();
      let mut proxy = Proxy::new(Intercept::All, proxy_uri);
      proxy.set_authorization(Authorization::basic("John Doe", "Agent1234"));
      let connector = HttpConnector::new();
      let proxy_connector = ProxyConnector::from_proxy(connector, proxy).unwrap();
      proxy_connector
   };

   // Connecting to http will trigger regular GETs and POSTs.
   // We need to manually append the relevant headers to the request
   let uri: Uri = "http://my-remote-website.com".parse().unwrap();
   let mut req = Request::get(uri.clone()).body(Empty::<Bytes>::new()).unwrap();

   if let Some(headers) = proxy.http_headers(&uri) {
      req.headers_mut().extend(headers.clone().into_iter());
   }

   let client = Client::builder(TokioExecutor::new()).build(proxy);
   let fut_http = async {
      let res = client.request(req).await?;
      let body = res.into_body().collect().await?.to_bytes();

      Ok::<_, Box<dyn Error>>(String::from_utf8(body.to_vec()).unwrap())
   };

   // Connecting to an https uri is straightforward (uses 'CONNECT' method underneath)
   let uri = "https://my-remote-websitei-secured.com".parse().unwrap();
   let fut_https = async {
      let res = client.get(uri).await?;
      let body = res.into_body().collect().await?.to_bytes();

      Ok::<_, Box<dyn Error>>(String::from_utf8(body.to_vec()).unwrap())
   };

   let (http_res, https_res) = futures::future::join(fut_http, fut_https).await;
   let (_, _) = (http_res?, https_res?);

   Ok(())
}
```

## Features

`hyper-proxy2` exposes three main Cargo features, to configure which TLS implementation it uses to
connect to a proxy. It can also be configured without TLS support, by compiling without default
features entirely. The supported list of configurations is:

1. No TLS support (`default-features = false`)
2. TLS support via `native-tls` to link against the operating system's native TLS implementation
   (default)
3. TLS support via `rustls` (`default-features = false, features = ["rustls"]`)
4. TLS support via `rustls`, using a statically-compiled set of CA certificates to bypass the
   operating system's default store (`default-features = false, features = ["rustls-webpki"]`)

## Credits

Large part of the code comes from [reqwest][2].
The core part as just been extracted and slightly enhanced.

 Main changes are:
- support for authentication
- add non secured tunneling
- add the possibility to add additional headers when connecting to the proxy

[1]: https://crates.io/crates/hyper
[2]: https://github.com/seanmonstar/reqwest
[3]: https://docs.rs/hyper-proxy2
