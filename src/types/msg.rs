use std::os::raw::c_char;
use serde::{Serialize, Deserialize};

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone )]
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
