//! Locate imports through ELF load headers; the kernel separately validates code.
use super::*;
pub fn from_elf(bytes: &[u8]) -> Result<Option<Declaration>, Refusal> {
    if bytes.get(..6) != Some(b"\x7fELF\x02\x01") || bytes.len() < 64 {
        return Err(Refusal::Malformed);
    }
    let start = usize::try_from(word(bytes, 32)?).map_err(|_| Refusal::Malformed)?;
    let size = u16::from_le_bytes(bytes[54..56].try_into().unwrap()) as usize;
    let count = u16::from_le_bytes(bytes[56..58].try_into().unwrap()) as usize;
    if size != 56 || count > 16 {
        return Err(Refusal::Malformed);
    }
    let end = start.checked_add(count * size).ok_or(Refusal::Malformed)?;
    let headers = bytes.get(start..end).ok_or(Refusal::Malformed)?;
    let mut found = None;
    for header in headers.chunks_exact(size) {
        // PT_LOAD, read-only. The runtime linker puts the declaration first.
        if header[..8] != [1, 0, 0, 0, 4, 0, 0, 0] {
            continue;
        }
        let offset = usize::try_from(word(header, 8)?).map_err(|_| Refusal::Malformed)?;
        let length = usize::try_from(word(header, 32)?).map_err(|_| Refusal::Malformed)?;
        let end = offset.checked_add(length).ok_or(Refusal::Malformed)?;
        let data = bytes.get(offset..end).ok_or(Refusal::Malformed)?;
        if data.get(..6) != Some(b"CTHIMP") {
            continue;
        }
        // A future/invalid record version is not an undeclared legacy image.
        if data.get(..8) != Some(&MAGIC.to_le_bytes())
            && data.get(..8) != Some(&BROKER_MAGIC.to_le_bytes())
        {
            return Err(Refusal::Malformed);
        }
        if found.is_some() {
            return Err(Refusal::Malformed);
        }
        found = Some(Declaration::decode(
            data.get(..RECORD_BYTES).ok_or(Refusal::Malformed)?,
        )?);
    }
    Ok(found)
}
