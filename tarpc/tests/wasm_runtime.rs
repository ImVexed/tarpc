#![cfg(target_arch = "wasm32")]

use futures::StreamExt;
use tarpc::server::Channel as _;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_test::wasm_bindgen_test;

#[tarpc::service]
trait World {
    async fn hello(name: String) -> String;
}

#[derive(Clone)]
struct HelloServer;

impl World for HelloServer {
    async fn hello(self, _: tarpc::context::Context, name: String) -> String {
        format!("Hello, {name}!")
    }
}

#[wasm_bindgen_test(async)]
async fn wasm_server_client_roundtrip_over_in_memory_transport() {
    console_error_panic_hook::set_once();

    let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

    // Drive the server: execute request handlers and ensure responses are actually written.
    let server = tarpc::server::BaseChannel::with_defaults(server_transport);
    spawn_local(async move {
        server
            .execute(HelloServer.serve())
            .for_each_concurrent(None, |response| async move {
                spawn_local(async move {
                    let _ = response.await;
                });
            })
            .await;
    });

    // Drive the client dispatch loop.
    let tarpc::client::NewClient { client, dispatch } =
        WorldClient::new(tarpc::client::Config::default(), client_transport);
    spawn_local(async move {
        let _ = dispatch.await;
    });

    let resp = client
        .hello(tarpc::context::current(), "WASM".to_string())
        .await
        .expect("rpc should succeed");
    assert_eq!(resp, "Hello, WASM!");
}

