use crate::{
    codec::{
        gzip::header::{self, Header},
        Decode,
    },
    util::PartialBuffer,
};
use std::io::{Error, ErrorKind, Result};

use flate2::Crc;

#[derive(Debug)]
enum State {
    Header(header::Parser),
    Decoding,
    Footer(PartialBuffer<Vec<u8>>),
    Done,
    Invalid,
}

#[derive(Debug)]
pub struct GzipDecoder {
    inner: crate::codec::FlateDecoder,
    crc: Crc,
    state: State,
    header: Header,
}

impl GzipDecoder {
    pub(crate) fn new() -> Self {
        Self {
            inner: crate::codec::FlateDecoder::new(false),
            crc: Crc::new(),
            state: State::Header(header::Parser::default()),
            header: Header::default(),
        }
    }

    fn check_footer(&mut self, input: &[u8]) -> Result<()> {
        if input.len() < 8 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid gzip footer length",
            ));
        }

        let crc = self.crc.sum().to_le_bytes();
        let bytes_read = self.crc.amount().to_le_bytes();

        if crc != input[0..4] {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "CRC computed does not match",
            ));
        }

        if bytes_read != input[4..8] {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "amount of bytes read does not match",
            ));
        }

        Ok(())
    }

    fn process(
        &mut self,
        input: &mut PartialBuffer<&[u8]>,
        output: &mut PartialBuffer<&mut [u8]>,
        inner: impl Fn(
            &mut Self,
            &mut PartialBuffer<&[u8]>,
            &mut PartialBuffer<&mut [u8]>,
        ) -> Result<bool>,
    ) -> Result<bool> {
        loop {
            self.state = match std::mem::replace(&mut self.state, State::Invalid) {
                State::Header(mut parser) => {
                    if let Some(header) = parser.input(input)? {
                        self.header = header;
                        State::Decoding
                    } else {
                        State::Header(parser)
                    }
                }

                State::Decoding => {
                    if inner(self, input, output)? {
                        State::Footer(vec![0; 8].into())
                    } else {
                        State::Decoding
                    }
                }

                State::Footer(mut footer) => {
                    footer.copy_unwritten_from(input);

                    if footer.unwritten().is_empty() {
                        self.check_footer(footer.written())?;
                        State::Done
                    } else {
                        State::Footer(footer.take())
                    }
                }

                State::Done => State::Done,
                State::Invalid => panic!("Reached invalid state"),
            };

            if let State::Footer(_) | State::Done = self.state {
                return Ok(true);
            }

            if input.unwritten().is_empty() || output.unwritten().is_empty() {
                return Ok(false);
            }
        }
    }
}

impl Decode for GzipDecoder {
    fn decode(
        &mut self,
        input: &mut PartialBuffer<&[u8]>,
        output: &mut PartialBuffer<&mut [u8]>,
    ) -> Result<bool> {
        self.process(input, output, |this, input, output| {
            let prior_written = output.written().len();
            let done = this.inner.decode(input, output)?;
            this.crc.update(&output.written()[prior_written..]);
            Ok(done)
        })
    }

    fn finish(&mut self, output: &mut PartialBuffer<&mut [u8]>) -> Result<bool> {
        self.process(
            &mut PartialBuffer::new(&[][..]),
            output,
            |this, _, output| {
                let prior_written = output.written().len();
                let done = this.inner.finish(output)?;
                this.crc.update(&output.written()[prior_written..]);
                Ok(done)
            },
        )
    }
}
