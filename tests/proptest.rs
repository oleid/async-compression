mod utils;

macro_rules! tests {
    ($($name:ident),*) => {
        $(
            mod $name {
                mod stream {
                    use crate::utils;
                    use proptest::{prelude::any, proptest};
                    use std::iter::FromIterator;
                    proptest! {
                        #[test]
                        fn compress(ref input in any::<utils::InputStream>()) {
                            let compressed = utils::$name::stream::compress(input.stream());
                            let output = utils::$name::sync::decompress(&compressed);
                            assert_eq!(output, input.bytes());
                        }

                        #[test]
                        fn decompress(
                            ref input in any::<Vec<u8>>(),
                            chunk_size in 1..20usize,
                        ) {
                            let compressed = utils::$name::sync::compress(input);
                            let stream = utils::InputStream::from(Vec::from_iter(compressed.chunks(chunk_size).map(Vec::from)));
                            let output = utils::$name::stream::decompress(stream.stream());
                            assert_eq!(&output, input);
                        }
                    }
                }

                mod bufread {
                    use crate::utils;
                    use proptest::{prelude::any, proptest};
                    use std::iter::FromIterator;
                    proptest! {
                        #[test]
                        fn compress(ref input in any::<utils::InputStream>()) {
                            let compressed = utils::$name::bufread::compress(input.reader());
                            let output = utils::$name::sync::decompress(&compressed);
                            assert_eq!(output, input.bytes());
                        }

                        #[test]
                        fn decompress(
                            ref input in any::<Vec<u8>>(),
                            chunk_size in 1..20usize,
                        ) {
                            let compressed = utils::$name::sync::compress(input);
                            let stream = utils::InputStream::from(Vec::from_iter(compressed.chunks(chunk_size).map(Vec::from)));
                            let output = utils::$name::bufread::decompress(stream.reader());
                            assert_eq!(&output, input);
                        }
                    }
                }
            }
        )*
    }
}

tests!(brotli, deflate, gzip, zlib, zstd);
