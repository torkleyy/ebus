pub struct Telegram<'a> {
    /// QQ - source eBUS address
    pub src: u8,
    /// ZZ - destination eBUS address
    pub dest: u8,
    /// Service command, encoded LSB first
    pub service: u16,
    /// Up to 16 data bytes
    pub data: &'a [u8],
    /// Whether the data is expected to have an additional CRC prepended
    pub needs_data_crc: bool,
    /// Whether or not to wait for the recipient to reply
    pub expect_reply: bool,
}