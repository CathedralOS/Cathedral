// Resolve a program's declared interfaces against its owner's offered services.
// This describes connections, never grants authority or authenticates a provider.
pub const INTERACTION: u64 = 1;
pub const STORAGE: u64 = 2;
pub const INPUT_PRODUCER: u64 = 4;
pub const CAPTURE: u64 = 8;
/// Existing ordinary launch offer. New authority is never added implicitly.
pub const ALL: u64 = INTERACTION | STORAGE;
pub const SUPPORTED: u64 = ALL | INPUT_PRODUCER | CAPTURE;
pub const MAX_IMPORTS: usize = 4;
pub const RECORD_BYTES: usize = 16 + MAX_IMPORTS * 32;
pub const PROFILE_MAGIC: u64 = u64::from_le_bytes(*b"CTHIMP05");
pub const PROFILE_BYTES: usize = RECORD_BYTES + 24;
pub const FUNDED_MAGIC: u64 = u64::from_le_bytes(*b"CTHIMP03");
pub const BROKER_MAGIC: u64 = u64::from_le_bytes(*b"CTHIMP02");
pub const MAGIC: u64 = u64::from_le_bytes(*b"CTHIMP01");
/// Platform launch refusals; these are not new kernel errors.
pub const MISSING: u64 = 0x494d0001;
pub const INCOMPATIBLE: u64 = 0x494d0002;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Import {
    pub contract: u64,
    pub minimum: u64,
    pub maximum: u64,
    pub required: bool,
}
impl Import {
    pub const fn v1(contract: u64, required: bool) -> Self {
        Self {
            contract,
            minimum: 1,
            maximum: 1,
            required,
        }
    }
}
pub const REQUIRED: [Import; 2] = [Import::v1(INTERACTION, true), Import::v1(STORAGE, true)];
pub const OPTIONAL_STORAGE: [Import; 2] =
    [Import::v1(INTERACTION, true), Import::v1(STORAGE, false)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Malformed,
    Missing(u64),
    Version(u64),
}

/// Known providers currently implement version 1. Unknown optional imports
/// remain absent; unknown required imports refuse admission. Order has no meaning.
pub fn bind(imports: &[Import], offered: u64) -> Result<u64, Refusal> {
    if imports.len() > MAX_IMPORTS || offered & !SUPPORTED != 0 {
        return Err(Refusal::Malformed);
    }
    let mut bound = 0;
    for (index, import) in imports.iter().enumerate() {
        if import.contract == 0
            || import.minimum == 0
            || import.minimum > import.maximum
            || imports[..index]
                .iter()
                .any(|prior| prior.contract == import.contract)
        {
            return Err(Refusal::Malformed);
        }
        let known = matches!(
            import.contract,
            INTERACTION | STORAGE | INPUT_PRODUCER | CAPTURE
        );
        if !known || offered & import.contract == 0 {
            if import.required {
                return Err(Refusal::Missing(import.contract));
            }
        } else if !(import.minimum..=import.maximum).contains(&1) {
            if import.required {
                return Err(Refusal::Version(import.contract));
            }
        } else {
            bound |= import.contract;
        }
    }
    Ok(bound)
}

/// Fixed-size little-endian record at the beginning of the read-only ELF load
/// segment. Included in the sealed program bytes, independent of section headers.
pub const fn encode(imports: &[Import]) -> [u8; RECORD_BYTES] {
    assert!(imports.len() <= MAX_IMPORTS);
    let mut words = [0u64; RECORD_BYTES / 8];
    words[0] = MAGIC;
    words[1] = imports.len() as u64;
    let mut i = 0;
    while i < imports.len() {
        let entry = imports[i];
        words[2 + i * 4] = entry.contract;
        words[3 + i * 4] = entry.minimum;
        words[4 + i * 4] = entry.maximum;
        words[5 + i * 4] = entry.required as u64;
        i += 1;
    }
    let mut bytes = [0; RECORD_BYTES];
    i = 0;
    while i < RECORD_BYTES {
        bytes[i] = words[i / 8].to_le_bytes()[i % 8];
        i += 1;
    }
    bytes
}

#[derive(Debug)]
pub struct Declaration {
    pub brokered: bool,
    /// Explicit private pages, reader pages and surface demand. Owner offers must cover it.
    pub profile: Option<[u64; 3]>,
    /// Requested channel/descendant ceilings. The owner may grant less.
    pub resources: Option<(u64, u64)>,
    entries: [Import; MAX_IMPORTS],
    count: usize,
}
impl Declaration {
    pub fn entries(&self) -> &[Import] {
        &self.entries[..self.count]
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, Refusal> {
        if bytes.len()
            != if word(bytes, 0)? == PROFILE_MAGIC {
                PROFILE_BYTES
            } else {
                RECORD_BYTES
            }
            || !matches!(
                word(bytes, 0)?,
                MAGIC | BROKER_MAGIC | FUNDED_MAGIC | PROFILE_MAGIC
            )
        {
            return Err(Refusal::Malformed);
        }
        let header = word(bytes, 8)?;
        let resources = if matches!(word(bytes, 0)?, FUNDED_MAGIC | PROFILE_MAGIC) {
            if header >> 48 != 0 || header & 0xff00 != 0 {
                return Err(Refusal::Malformed);
            }
            Some(((header >> 16) & 0xffff, (header >> 32) & 0xffff))
        } else {
            None
        };
        let count = usize::try_from(if resources.is_some() {
            header & 0xff
        } else {
            header
        })
        .map_err(|_| Refusal::Malformed)?;
        if count > MAX_IMPORTS {
            return Err(Refusal::Malformed);
        }
        let mut declaration = Self {
            brokered: matches!(word(bytes, 0)?, BROKER_MAGIC | FUNDED_MAGIC | PROFILE_MAGIC),
            profile: if word(bytes, 0)? == PROFILE_MAGIC {
                Some([
                    word(bytes, RECORD_BYTES)?,
                    word(bytes, RECORD_BYTES + 8)?,
                    word(bytes, RECORD_BYTES + 16)?,
                ])
            } else {
                None
            },
            resources,
            entries: [Import::v1(0, false); MAX_IMPORTS],
            count,
        };
        for (i, entry) in declaration.entries[..count].iter_mut().enumerate() {
            let at = 16 + i * 32;
            let required = word(bytes, at + 24)?;
            if required > 1 {
                return Err(Refusal::Malformed);
            }
            *entry = Import {
                contract: word(bytes, at)?,
                minimum: word(bytes, at + 8)?,
                maximum: word(bytes, at + 16)?,
                required: required != 0,
            };
        }
        if bytes[16 + count * 32..RECORD_BYTES]
            .iter()
            .any(|&byte| byte != 0)
        {
            return Err(Refusal::Malformed);
        }
        // Validate syntax even when a required interface is unavailable.
        for (i, entry) in declaration.entries().iter().enumerate() {
            if entry.contract == 0
                || entry.minimum == 0
                || entry.minimum > entry.maximum
                || declaration.entries()[..i]
                    .iter()
                    .any(|prior| prior.contract == entry.contract)
            {
                return Err(Refusal::Malformed);
            }
        }
        Ok(declaration)
    }
}
fn word(bytes: &[u8], at: usize) -> Result<u64, Refusal> {
    let end = at.checked_add(8).ok_or(Refusal::Malformed)?;
    Ok(u64::from_le_bytes(
        bytes
            .get(at..end)
            .ok_or(Refusal::Malformed)?
            .try_into()
            .unwrap(),
    ))
}

/// Same import semantics, opting into the two-endpoint scoped bootstrap.
pub const fn encode_brokered(imports: &[Import]) -> [u8; RECORD_BYTES] {
    let mut bytes = encode(imports);
    let magic = BROKER_MAGIC.to_le_bytes();
    let mut i = 0;
    while i < 8 {
        bytes[i] = magic[i];
        i += 1;
    }
    bytes
}

/// Requested funding is metadata, never authority. Owners cap it to their offer;
/// ordinary leaves request no channel-creation or descendant capacity.
pub const fn encode_funded(
    imports: &[Import],
    channels: u16,
    descendants: u16,
) -> [u8; RECORD_BYTES] {
    let mut bytes = encode_brokered(imports);
    let magic = FUNDED_MAGIC.to_le_bytes();
    let header = (imports.len() as u64 | ((channels as u64) << 16) | ((descendants as u64) << 32))
        .to_le_bytes();
    let mut i = 0;
    while i < 8 {
        bytes[i] = magic[i];
        bytes[i + 8] = header[i];
        i += 1;
    }
    bytes
}

/// Explicit resource requirements with scoped broker startup.
/// The host refuses insufficient offers before spawning; this record grants nothing.
pub const fn encode_profile(
    imports: &[Import],
    channels: u16,
    descendants: u16,
    profile: [u64; 3],
) -> [u8; PROFILE_BYTES] {
    let base = encode_funded(imports, channels, descendants);
    let mut bytes = [0; PROFILE_BYTES];
    let mut i = 0;
    while i < RECORD_BYTES {
        bytes[i] = base[i];
        i += 1;
    }
    i = 0;
    while i < 8 {
        bytes[i] = PROFILE_MAGIC.to_le_bytes()[i];
        i += 1;
    }
    i = 0;
    while i < 24 {
        bytes[RECORD_BYTES + i] = profile[i / 8].to_le_bytes()[i % 8];
        i += 1;
    }
    bytes
}
