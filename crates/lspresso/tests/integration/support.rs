use std::{sync::mpsc, time::Duration};

use lspresso::{
    Client,
    gen_lsp_types::{ClientCapabilities, Uri},
};

pub fn test_uri() -> Uri {
    lspresso::path_to_uri(&std::path::PathBuf::from(if cfg!(windows) {
        r"C:\t.json"
    } else {
        "/t.json"
    }))
}

pub fn spawn_and_init(server: std::process::Command) -> Client {
    let mut client = Client::spawn(server).expect("spawn");
    client
        .initialize(ClientCapabilities::default())
        .expect("initialize");
    client
}

pub fn assert_completes_within<F, T>(wall: Duration, f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel::<std::thread::Result<T>>();
    std::thread::spawn(move || {
        drop(tx.send(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))));
    });
    match rx.recv_timeout(wall) {
        Ok(Ok(val)) => val,
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(mpsc::RecvTimeoutError::Timeout) => panic!("operation hung for >{wall:?}"),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            unreachable!("catch_unwind prevents disconnect")
        }
    }
}

#[test]
#[should_panic(expected = "injected panic")]
fn assert_completes_within_propagates_panics() {
    let _: () = assert_completes_within(Duration::from_secs(1), || panic!("injected panic"));
}
