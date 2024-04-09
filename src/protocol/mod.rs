use enumset::EnumSetType;

#[derive(Debug, EnumSetType)]
pub enum ChannelFlags {
    Stream,
    LZ4,
}
