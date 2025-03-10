use bytes::Bytes;
use flate2::Compression;
use futures_core::stream::Stream;
use std::io::Result;

decoder! {
    /// A gzip decoder, or decompressor.
    #[cfg_attr(docsrs, doc(cfg(feature = "gzip")))]
    GzipDecoder
}

encoder! {
    /// A gzip encoder, or compressor.
    #[cfg_attr(docsrs, doc(cfg(feature = "gzip")))]
    GzipEncoder
}

impl<S: Stream<Item = Result<Bytes>>> GzipEncoder<S> {
    /// Creates a new encoder which will read uncompressed data from the given stream and emit a
    /// compressed stream.
    pub fn new(stream: S, level: Compression) -> Self {
        Self {
            inner: crate::stream::generic::Encoder::new(
                stream,
                crate::codec::GzipEncoder::new(level),
            ),
        }
    }
}
