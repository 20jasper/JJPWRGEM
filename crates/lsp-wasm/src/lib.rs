use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Server(lsp::Server);

#[wasm_bindgen]
impl Server {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(lsp::Server::new())
    }

    pub fn handle(&mut self, message: &str) -> Vec<String> {
        self.0.handle(message)
    }
}
