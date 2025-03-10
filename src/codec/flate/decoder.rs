use crate::{codec::Decode, util::PartialBuffer};
use std::io::{Error, ErrorKind, Result};

use flate2::{Decompress, FlushDecompress, Status};

#[derive(Debug)]
pub struct FlateDecoder {
    decompress: Decompress,
}

impl FlateDecoder {
    pub(crate) fn new(zlib_header: bool) -> Self {
        Self {
            decompress: Decompress::new(zlib_header),
        }
    }

    fn decode(
        &mut self,
        input: &mut PartialBuffer<&[u8]>,
        output: &mut PartialBuffer<&mut [u8]>,
        flush: FlushDecompress,
    ) -> Result<Status> {
        let prior_in = self.decompress.total_in();
        let prior_out = self.decompress.total_out();

        let status =
            self.decompress
                .decompress(input.unwritten(), output.unwritten_mut(), flush)?;

        input.advance((self.decompress.total_in() - prior_in) as usize);
        output.advance((self.decompress.total_out() - prior_out) as usize);

        Ok(status)
    }
}

impl Decode for FlateDecoder {
    fn decode(
        &mut self,
        input: &mut PartialBuffer<&[u8]>,
        output: &mut PartialBuffer<&mut [u8]>,
    ) -> Result<bool> {
        match self.decode(input, output, FlushDecompress::None)? {
            Status::Ok => Ok(false),
            Status::StreamEnd => Ok(true),
            Status::BufError => Err(Error::new(ErrorKind::Other, "unexpected BufError")),
        }
    }

    fn finish(&mut self, output: &mut PartialBuffer<&mut [u8]>) -> Result<bool> {
        match self.decode(
            &mut PartialBuffer::new(&[][..]),
            output,
            FlushDecompress::Finish,
        )? {
            Status::Ok => Ok(false),
            Status::StreamEnd => Ok(true),
            Status::BufError => Err(Error::new(ErrorKind::Other, "unexpected BufError")),
        }
    }
}
