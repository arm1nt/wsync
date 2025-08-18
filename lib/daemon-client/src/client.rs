use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Deserializer;
use crate::errors::ClientError;

pub struct Client {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>
}

impl Client {
    pub fn new(stream: UnixStream) -> Result<Self, ClientError> {
        let r = stream.try_clone()?;
        let w = stream;
        Ok( Self { reader: BufReader::new(r), writer: BufWriter::new(w)} )
    }

    pub fn read_line(&mut self) -> Result<String, ClientError> {
        let mut buf = String::new();
        let bytes_read = self.reader.read_line(&mut buf)?;

        if bytes_read == 0 {
            return Err(ClientError::Protocol("Connection closed before reading request data"));
        }

        Ok(buf)
    }

    pub fn read_json<T: DeserializeOwned>(&mut self) -> Result<T, ClientError> {
        let mut stream = Deserializer::from_reader(&mut self.reader).into_iter::<T>();
        let data = stream.next();

        if data.is_none() {
            return Err(ClientError::Protocol("Missing command data"));
        }

        Ok(data.unwrap()?)
    }

    pub fn write_json<T: Serialize>(&mut self, data: &T) -> Result<(), ClientError> {
        serde_json::to_writer_pretty(&mut self.writer, data)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn shutdown(&mut self) {
        let _ = self.writer.get_ref().shutdown(Shutdown::Both);
    }
}
