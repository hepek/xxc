use std::io::Read;
// Issues the correct number of reads to read buf.len() bytes
// Gives up in case of EOF (Ok(0))
pub fn read_fill<R: Read>(reader: &mut R, buf: &mut[u8]) -> Result<usize, std::io::Error> {
    let mut bytes_read = 0;

    loop {
        match reader.read(&mut buf[bytes_read..])? {
            0 => return Ok(bytes_read),
            n => {
                bytes_read += n;
                if bytes_read == buf.len() {
                    return Ok(bytes_read);
                }
            },
        }
    }
}
