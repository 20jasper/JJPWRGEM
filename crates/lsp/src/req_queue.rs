// vendored from lsp-server v0.10.0
use std::collections::HashMap;

pub(crate) use gen_lsp_types::json_rpc::Id;

pub struct Outgoing<O> {
    next_id: i64,
    pending: HashMap<Id, O>,
}

impl<O> Outgoing<O> {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            pending: HashMap::new(),
        }
    }

    #[expect(dead_code, reason = "first caller is the schema fetch")]
    pub fn register(&mut self, data: O) -> Id {
        let id = Id::Number(self.next_id);
        self.pending.insert(id.clone(), data);
        self.next_id += 1;
        id
    }

    pub fn complete(&mut self, id: &Id) -> Option<O> {
        self.pending.remove(id)
    }
}
