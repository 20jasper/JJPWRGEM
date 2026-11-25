use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Server)]
#[derive(Debug, Default)]
pub struct WasmServer(crate::Server);

#[wasm_bindgen(js_class = Server)]
impl WasmServer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(crate::Server::new(env!("CARGO_PKG_VERSION")))
    }

    pub fn handle(&mut self, message: &str) -> Vec<String> {
        self.0.handle(message)
    }
}
