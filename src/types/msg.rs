use std::os::raw::c_char;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

// ============ CONSTANTS ============

// Action codes
pub const ACTION_ADD: c_char = 65;      // 'A'
pub const ACTION_CANCEL: c_char = 67;   // 'C'
pub const ACTION_MODIFY: c_char = 77;   // 'M'
pub const ACTION_CLEAR: c_char = 82;    // 'R'
pub const ACTION_TRADE: c_char = 84;    // 'T'
pub const ACTION_FILL: c_char = 70;     // 'F'
pub const ACTION_NONE: c_char = 78;     // 'N'

// Side codes
pub const SIDE_ASK: c_char = 65;        // 'A'
pub const SIDE_BID: c_char = 66;        // 'B'
pub const SIDE_NONE: c_char = 78;       // 'N'

// Flag bits
pub const FLAG_LAST: u8 = 128;
pub const FLAG_TOB: u8 = 64;

// Price
pub const FIXED_PRICE_SCALE: f64 = 1_000_000_000.0;
pub const UNDEF_PRICE: i64 = i64::MAX;

// ============ MBO MESSAGE ============

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordHeader {
    pub rtype: u8,
    pub publisher_id: u16,
    pub instrument_id: u32,
    pub ts_event: u64,
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MboMsg {
    pub hd: RecordHeader,
    pub order_id: u64,
    pub price: i64,
    pub size: u32,
    pub flags: u8,
    pub channel_id: u8,
    pub action: c_char,
    pub side: c_char,
    pub ts_recv: u64,
    pub ts_in_delta: i32,
    pub sequence: u32,
}

impl MboMsg {
    pub fn action_char(&self) -> char { char::from(self.action as u8) }
    pub fn side_char(&self) -> char { char::from(self.side as u8) }
    pub fn is_add(&self) -> bool { self.action == ACTION_ADD }
    pub fn is_cancel(&self) -> bool { self.action == ACTION_CANCEL }
    pub fn is_modify(&self) -> bool { self.action == ACTION_MODIFY }
    pub fn is_clear(&self) -> bool { self.action == ACTION_CLEAR }
    pub fn is_trade(&self) -> bool { self.action == ACTION_TRADE }
    pub fn is_fill(&self) -> bool { self.action == ACTION_FILL }
    pub fn is_bid(&self) -> bool { self.side == SIDE_BID }
    pub fn is_ask(&self) -> bool { self.side == SIDE_ASK }
    pub fn is_last(&self) -> bool { self.flags & FLAG_LAST != 0 }
    pub fn is_tob(&self) -> bool { self.flags & FLAG_TOB != 0 }
    pub fn price_f64(&self) -> f64 { self.price as f64 / FIXED_PRICE_SCALE }
    pub fn is_undef_price(&self) -> bool { self.price == UNDEF_PRICE }
    pub fn instrument_id(&self) -> u32 { self.hd.instrument_id }
    pub fn publisher_id(&self) -> u16 { self.hd.publisher_id }
    pub fn ts_event(&self) -> u64 { self.hd.ts_event }
}

// ============ PRICE LEVEL ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: i64,
    pub size: u32,
    pub count: u32,
}

impl PriceLevel {
    pub fn new(price: i64) -> Self {
        Self { price, size: 0, count: 0 }
    }
    pub fn price_f64(&self) -> f64 {
        self.price as f64 / FIXED_PRICE_SCALE
    }
}

impl std::fmt::Display for PriceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:4} @ {:6.2} | {} order(s)", self.size, self.price_f64(), self.count)
    }
}

// ============ BID ASK PAIR ============

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BidAskPair {
    pub bid_px: i64,
    pub bid_sz: u32,
    pub bid_ct: u32,
    pub ask_px: i64,
    pub ask_sz: u32,
    pub ask_ct: u32,
}

impl BidAskPair {
    pub fn new() -> Self { Self::default() }
    pub fn bid_price_f64(&self) -> f64 { self.bid_px as f64 / FIXED_PRICE_SCALE }
    pub fn ask_price_f64(&self) -> f64 { self.ask_px as f64 / FIXED_PRICE_SCALE }
}

// ============ LEVEL ORDERS ============

#[derive(Debug, Clone)]
pub struct LevelOrders {
    pub price: i64,
    pub orders: Vec<MboMsg>,
}

impl LevelOrders {
    pub fn new(price: i64) -> Self {
        Self { price, orders: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    pub fn level(&self) -> PriceLevel {
        PriceLevel {
            price: self.price,
            count: self.orders.iter().filter(|o| !o.is_tob()).count() as u32,
            size: self.orders.iter().map(|o| o.size).sum(),
        }
    }
}

// ============ BOOK ============

pub struct Book {
    pub orders_by_id: BTreeMap<u64, MboMsg>,
    pub offers: BTreeMap<i64, LevelOrders>,
    pub bids: BTreeMap<i64, LevelOrders>,
}

impl Book {
    pub fn new() -> Self {
        Self {
            orders_by_id: BTreeMap::new(),
            offers: BTreeMap::new(),
            bids: BTreeMap::new(),
        }
    }

    pub fn bbo(&self) -> (Option<PriceLevel>, Option<PriceLevel>) {
        (self.get_bid_level(0), self.get_ask_level(0))
    }

    pub fn get_bid_level(&self, idx: usize) -> Option<PriceLevel> {
        self.bids.values().rev().nth(idx).map(|l| l.level())
    }

    pub fn get_ask_level(&self, idx: usize) -> Option<PriceLevel> {
        self.offers.values().nth(idx).map(|l| l.level())
    }

    pub fn get_order(&self, id: u64) -> Option<&MboMsg> {
        self.orders_by_id.get(&id)
    }

    pub fn get_snapshot(&self, level_count: usize) -> Vec<BidAskPair> {
        (0..level_count).map(|i| {
            let mut pair = BidAskPair::new();
            if let Some(bid) = self.get_bid_level(i) {
                pair.bid_px = bid.price;
                pair.bid_sz = bid.size;
                pair.bid_ct = bid.count;
            }
            if let Some(ask) = self.get_ask_level(i) {
                pair.ask_px = ask.price;
                pair.ask_sz = ask.size;
                pair.ask_ct = ask.count;
            }
            pair
        }).collect()
    }

    pub fn get_depth(&self, levels: usize) -> (Vec<PriceLevel>, Vec<PriceLevel>) {
        let bids: Vec<PriceLevel> = self.bids.values().rev().take(levels).map(|l| l.level()).collect();
        let asks: Vec<PriceLevel> = self.offers.values().take(levels).map(|l| l.level()).collect();
        (bids, asks)
    }

    pub fn apply(&mut self, mbo: &MboMsg) {
        // Trade, Fill, None: no change
        if mbo.is_trade() || mbo.is_fill() || mbo.action == ACTION_NONE {
            return;
        }
        // Clear book
        if mbo.is_clear() {
            self.clear();
            return;
        }
        // Side must be A or B
        if !mbo.is_ask() && !mbo.is_bid() {
            return;
        }
        // UNDEF_PRICE with TOB: clear side
        if mbo.is_undef_price() && mbo.is_tob() {
            self.side_levels_mut(mbo.side).clear();
            return;
        }

        if mbo.is_add() {
            self.add(mbo);
        } else if mbo.is_cancel() {
            self.cancel(mbo);
        } else if mbo.is_modify() {
            self.modify(mbo);
        }
    }

    fn clear(&mut self) {
        self.orders_by_id.clear();
        self.offers.clear();
        self.bids.clear();
    }

    fn add(&mut self, mbo: &MboMsg) {
        if mbo.is_tob() {
            let levels = self.side_levels_mut(mbo.side);
            levels.clear();
            levels.insert(mbo.price, LevelOrders { price: mbo.price, orders: vec![mbo.clone()] });
        } else {
            self.orders_by_id.insert(mbo.order_id, mbo.clone());
            let level = self.get_or_insert_level(mbo.price, mbo.side);
            level.orders.push(mbo.clone());
        }
    }

    fn cancel(&mut self, mbo: &MboMsg) {
        if let Some(order) = self.orders_by_id.get_mut(&mbo.order_id) {
            let price = order.price;
            let side = order.side;

            if order.size >= mbo.size {
                order.size -= mbo.size;
            }

            if order.size == 0 {
                self.orders_by_id.remove(&mbo.order_id);
                if let Some(level) = self.side_levels_mut(side).get_mut(&price) {
                    level.orders.retain(|o| o.order_id != mbo.order_id);
                    if level.is_empty() {
                        self.side_levels_mut(side).remove(&price);
                    }
                }
            }
        }
    }

    fn modify(&mut self, mbo: &MboMsg) {
        if let Some(order) = self.orders_by_id.get(&mbo.order_id).cloned() {
            let old_price = order.price;
            let side = order.side;

            if let Some(level) = self.side_levels_mut(side).get_mut(&old_price) {
                level.orders.retain(|o| o.order_id != mbo.order_id);
                if level.is_empty() {
                    self.side_levels_mut(side).remove(&old_price);
                }
            }

            let new_level = self.get_or_insert_level(mbo.price, mbo.side);
            new_level.orders.push(mbo.clone());
            self.orders_by_id.insert(mbo.order_id, mbo.clone());
        } else {
            self.add(mbo);
        }
    }

    fn side_levels_mut(&mut self, side: c_char) -> &mut BTreeMap<i64, LevelOrders> {
        if side == SIDE_ASK { &mut self.offers } else { &mut self.bids }
    }

    fn get_or_insert_level(&mut self, price: i64, side: c_char) -> &mut LevelOrders {
        self.side_levels_mut(side).entry(price).or_insert_with(|| LevelOrders::new(price))
    }
}

// ============ MARKET ============

pub struct Market {
    pub books: BTreeMap<u32, BTreeMap<u16, Book>>,
}

impl Market {
    pub fn new() -> Self {
        Self { books: BTreeMap::new() }
    }

    pub fn get_book(&mut self, instrument_id: u32, publisher_id: u16) -> &mut Book {
        self.books
            .entry(instrument_id)
            .or_insert_with(BTreeMap::new)
            .entry(publisher_id)
            .or_insert_with(Book::new)
    }

    pub fn bbo(&mut self, instrument_id: u32, publisher_id: u16) -> (Option<PriceLevel>, Option<PriceLevel>) {
        self.get_book(instrument_id, publisher_id).bbo()
    }

    pub fn apply(&mut self, mbo: &MboMsg) {
        self.get_book(mbo.instrument_id(), mbo.publisher_id()).apply(mbo);
    }

    pub fn aggregated_bbo(&self, instrument_id: u32) -> (Option<PriceLevel>, Option<PriceLevel>) {
        let Some(books) = self.books.get(&instrument_id) else {
            return (None, None);
        };

        // Best bid = max price
        let best_bid = books.values()
            .filter_map(|b| b.get_bid_level(0))
            .max_by_key(|l| l.price)
            .map(|best| {
                let all_at_price: Vec<_> = books.values()
                    .filter_map(|b| b.get_bid_level(0))
                    .filter(|l| l.price == best.price)
                    .collect();
                PriceLevel {
                    price: best.price,
                    size: all_at_price.iter().map(|l| l.size).sum(),
                    count: all_at_price.iter().map(|l| l.count).sum(),
                }
            });

        // Best ask = min price
        let best_ask = books.values()
            .filter_map(|b| b.get_ask_level(0))
            .min_by_key(|l| l.price)
            .map(|best| {
                let all_at_price: Vec<_> = books.values()
                    .filter_map(|b| b.get_ask_level(0))
                    .filter(|l| l.price == best.price)
                    .collect();
                PriceLevel {
                    price: best.price,
                    size: all_at_price.iter().map(|l| l.size).sum(),
                    count: all_at_price.iter().map(|l| l.count).sum(),
                }
            });

        (best_bid, best_ask)
    }
}
