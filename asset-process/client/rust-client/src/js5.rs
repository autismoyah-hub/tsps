//! Revision 239 cache wire protocol. A response is split into 512-byte blocks;
//! continuation blocks start with 0xff and contain up to 511 payload bytes.
use crate::{Error, Result};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    pub archive: u8,
    pub group: u16,
    /// Complete JS5 container, without the disk-only version suffix.
    pub container: Vec<u8>,
}

struct Pending {
    response: Response,
    total: usize,
    block: usize,
}

pub struct Decoder {
    header: [u8; 8],
    header_len: usize,
    pending: Option<Pending>,
    xor: u8,
    max_size: usize,
    failed: bool,
}

impl Decoder {
    pub fn new(max_size: usize, xor: u8) -> Self {
        Self {
            header: [0; 8],
            header_len: 0,
            pending: None,
            xor,
            max_size,
            failed: false,
        }
    }

    /// Accept arbitrary TCP fragments. Discard the decoder after an error.
    pub fn feed(&mut self, input: &[u8]) -> Result<Vec<Response>> {
        if self.failed {
            return Err(Error::Invalid("failed JS5 decoder"));
        }
        let result = self.feed_inner(input);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn feed_inner(&mut self, input: &[u8]) -> Result<Vec<Response>> {
        let mut ready = Vec::new();
        for &encrypted in input {
            let byte = encrypted ^ self.xor;
            if let Some(p) = &mut self.pending {
                if p.block == 512 {
                    if byte != 255 {
                        return Err(Error::Invalid("JS5 continuation marker"));
                    }
                    p.block = 1;
                    continue;
                }
                p.response.container.push(byte);
                p.block += 1;
                if p.response.container.len() == p.total {
                    ready.push(self.pending.take().unwrap().response);
                }
            } else {
                self.header[self.header_len] = byte;
                self.header_len += 1;
                if self.header_len != 8 {
                    continue;
                }
                self.header_len = 0;
                let h = self.header;
                if h[3] > 2 {
                    return Err(Error::Unsupported("JS5 compression"));
                }
                let length = u32::from_be_bytes(h[4..8].try_into().unwrap()) as usize;
                if length > self.max_size {
                    return Err(Error::Invalid("JS5 group size limit"));
                }
                let total = length
                    .checked_add(if h[3] == 0 { 5 } else { 9 })
                    .ok_or(Error::Invalid("JS5 group size overflow"))?;
                let response = Response {
                    archive: h[0],
                    group: u16::from_be_bytes([h[1], h[2]]),
                    container: h[3..8].to_vec(),
                };
                if total == 5 {
                    ready.push(response);
                } else {
                    self.pending = Some(Pending {
                        response,
                        total,
                        block: 8,
                    });
                }
            }
        }
        Ok(ready)
    }

    pub fn finish(&self) -> Result<()> {
        if self.failed {
            Err(Error::Invalid("failed JS5 decoder"))
        } else if self.header_len != 0 || self.pending.is_some() {
            Err(Error::Truncated)
        } else {
            Ok(())
        }
    }
}

pub fn request(archive: u8, group: u16, urgent: bool) -> [u8; 4] {
    let id = group.to_be_bytes();
    [u8::from(urgent), archive, id[0], id[1]]
}

/// Maximum containers retained by one transport batch.
pub const MAX_BATCH: usize = 4;

/// Read-only cache downloader. It never sends game-login packets.
pub struct Client {
    stream: TcpStream,
    decoder: Decoder,
    usable: bool,
}

impl Client {
    pub fn connect(
        address: SocketAddr,
        revision: u32,
        key: [u32; 4],
        timeout: Duration,
        max_size: usize,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect_timeout(&address, timeout)?;
        // Original up socket constructor enables TCP_NODELAY before JS5 handshake.
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        stream.write_all(&crate::protocol::js5_handshake(revision, key))?;
        let mut reply = [0];
        stream.read_exact(&mut reply)?;
        if reply[0] != 0 {
            return Err(format!("JS5 handshake rejected: {}", reply[0]).into());
        }
        Ok(Self {
            stream,
            decoder: Decoder::new(max_size, 0),
            usable: true,
        })
    }

    pub fn fetch(
        &mut self,
        archive: u8,
        group: u16,
    ) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(self.fetch_batch(&[(archive, group)])?.pop().unwrap())
    }

    /// Send a bounded wave before reading responses, as Java's JS5 queues do.
    /// Results follow request order even when the server replies out of order.
    /// Invalid local arguments do not consume or poison the connection.
    pub fn fetch_batch(
        &mut self,
        keys: &[(u8, u16)],
    ) -> std::result::Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        if !self.usable {
            return Err("JS5 connection must be reopened after an error".into());
        }
        if keys.is_empty() || keys.len() > MAX_BATCH {
            return Err("JS5 batch size limit".into());
        }
        for (index, key) in keys.iter().enumerate() {
            if keys[..index].contains(key) {
                return Err("duplicate JS5 batch request".into());
            }
        }
        self.usable = false;
        let mut requests = Vec::with_capacity(keys.len() * 4);
        for &(archive, group) in keys {
            requests.extend_from_slice(&request(archive, group, true));
        }
        self.stream.write_all(&requests)?;
        let mut results: Vec<Option<Vec<u8>>> = (0..keys.len()).map(|_| None).collect();
        let mut remaining = keys.len();
        let mut buffer = [0; 8192];
        loop {
            let count = self.stream.read(&mut buffer)?;
            if count == 0 {
                return Err("JS5 connection closed before response".into());
            }
            for response in self.decoder.feed(&buffer[..count])? {
                let Some(index) = keys
                    .iter()
                    .position(|key| *key == (response.archive, response.group))
                else {
                    return Err("JS5 response identifier mismatch".into());
                };
                if results[index].is_some() {
                    return Err("duplicate JS5 response".into());
                }
                results[index] = Some(response.container);
                remaining -= 1;
            }
            if remaining == 0 {
                // A partial unsolicited response in this read is also invalid.
                self.decoder.finish()?;
                self.usable = true;
                return Ok(results.into_iter().map(Option::unwrap).collect());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Server-format fixture generator: first send archive/group, then split the
    // container on absolute 512-byte wire boundaries, inserting 0xff separators.
    fn wire(container: &[u8], xor: u8) -> Vec<u8> {
        let mut result = vec![7, 1, 44];
        for &b in container {
            if result.len() % 512 == 0 {
                result.push(255);
            }
            result.push(b);
        }
        result.iter_mut().for_each(|b| *b ^= xor);
        result
    }
    #[test]
    fn all_boundaries_and_xor() {
        for length in [0usize, 1, 503, 504, 505, 1015, 1016, 2048] {
            for compression in 0..=2 {
                let mut container = vec![compression];
                container.extend_from_slice(&(length as u32).to_be_bytes());
                if compression != 0 {
                    container.extend_from_slice(&999u32.to_be_bytes());
                }
                container.extend((0..length).map(|i| i as u8));
                for xor in [0, 173] {
                    let mut bytes = wire(&container, xor);
                    bytes.extend(wire(&[0, 0, 0, 0, 0], xor));
                    for split in 0..=bytes.len() {
                        let mut decoder = Decoder::new(4096, xor);
                        let mut actual = decoder.feed(&bytes[..split]).unwrap();
                        actual.extend(decoder.feed(&bytes[split..]).unwrap());
                        decoder.finish().unwrap();
                        assert_eq!(actual.len(), 2);
                        assert_eq!(
                            actual[0],
                            Response {
                                archive: 7,
                                group: 300,
                                container: container.clone()
                            }
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn truncation_limits_and_marker() {
        let mut c = vec![0, 0, 0, 2, 0];
        c.extend([42; 512]);
        let bytes = wire(&c, 0);
        for end in 1..bytes.len() {
            let mut d = Decoder::new(1024, 0);
            d.feed(&bytes[..end]).unwrap();
            assert_eq!(d.finish(), Err(Error::Truncated));
        }
        assert!(Decoder::new(511, 0).feed(&bytes).is_err());
        let mut invalid = bytes;
        invalid[512] = 0;
        let mut d = Decoder::new(1024, 0);
        assert!(d.feed(&invalid).is_err());
        assert!(d.feed(&[]).is_err());
    }
}

#[cfg(test)]
mod transport_tests {
    use super::*;
    #[test]
    fn original_nodelay_is_set_on_each_js5_connection() {
        let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address=listener.local_addr().unwrap();
        let worker=std::thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream,_)=listener.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                stream.set_write_timeout(Some(Duration::from_secs(3))).unwrap();
                let mut handshake=[0;21];stream.read_exact(&mut handshake).unwrap();
                assert_eq!(handshake,crate::protocol::js5_handshake(239,[1,2,3,4]));
                stream.write_all(&[0]).unwrap();
            }
        });
        for _ in 0..2 {
            let client=Client::connect(address,239,[1,2,3,4],Duration::from_secs(3),4096).unwrap();
            assert!(client.stream.nodelay().unwrap());
            assert!(client.usable);
        }
        worker.join().unwrap();
    }
}
