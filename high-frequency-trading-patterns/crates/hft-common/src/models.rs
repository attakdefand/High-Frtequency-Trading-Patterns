use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side { Buy, Sell }

#[derive(Clone, Debug)]
pub struct Quote {
    pub bid: f64,
    pub ask: f64,
    pub ts:  Instant,
}

#[derive(Clone, Debug)]
pub struct Order {
    pub side: Side,
    pub qty:  f64,
    pub px:   f64,
}

#[derive(Clone, Debug)]
pub struct Fill {
    pub side: Side,
    pub qty:  f64,
    pub px:   f64,
    pub ts:   Instant,
}