use enumset::EnumSetType;

#[derive(Debug, EnumSetType)]
pub enum ChannelFlags {
    /// The channel is a stream of large data f.e. assets.
    ///  After the flags byte a length delimited packet follows to give infomation about the stream itself, after which the actual stream starts.
    Stream,
    /// The following data is compressed with the LZ4 algorithm.
    LZ4,
}
