use hft_common::prelude::*;

pub struct Logic { cfg: Cfg, ctr: u64 }
impl Logic {
    pub fn new(cfg: Cfg) -> Self { Self { cfg, ctr: 0 } }

    pub fn on_quote(&mut self, q:&Quote) -> Option<Order> {
        self.ctr += 1;
        // placeholder: cross every 256th quote
        if self.ctr % 256 == 0 {
            Some(Order{ side: Side::Buy, qty: 100.0, px: q.ask })
        } else { None }
    }
    pub fn on_fill(&mut self, _f:&Fill) {}
}
