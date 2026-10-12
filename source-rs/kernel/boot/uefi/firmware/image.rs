//! Read the already relocated PE32+ image. UEFI remains the loader; we only
//! recover page permissions before retiring firmware execution privileges.
//! Format: https://learn.microsoft.com/en-us/windows/win32/debug/pe-format
use cathedral_arch::{
    ImageRange,
    memory::{ImageSection, KernelImage, Permissions},
};
const EMPTY: ImageSection = ImageSection {
    range: ImageRange { base: 0, bytes: 0 },
    permissions: Permissions {
        writable: false,
        executable: false,
    },
};
pub struct LoadedSections {
    range: ImageRange,
    sections: [ImageSection; 16],
    count: usize,
}
impl LoadedSections {
    pub fn protection(&self) -> KernelImage<'_> {
        KernelImage::new(self.range, &self.sections[..self.count]).expect("validated image")
    }
}
pub fn parse(bytes: &[u8], base: u64) -> Result<LoadedSections, &'static str> {
    let word = |at: usize| -> Result<u16, &'static str> {
        Ok(u16::from_le_bytes(
            bytes
                .get(at..at + 2)
                .ok_or("short PE header")?
                .try_into()
                .unwrap(),
        ))
    };
    let dword = |at: usize| -> Result<u32, &'static str> {
        Ok(u32::from_le_bytes(
            bytes
                .get(at..at + 4)
                .ok_or("short PE header")?
                .try_into()
                .unwrap(),
        ))
    };
    if word(0)? != 0x5a4d {
        return Err("missing MZ");
    }
    let pe = dword(0x3c)? as usize;
    if dword(pe)? != 0x4550 {
        return Err("missing PE");
    }
    let count = word(pe + 6)? as usize;
    let optional_bytes = word(pe + 20)? as usize;
    let optional = pe + 24;
    if optional_bytes < 112 || word(optional)? != 0x20b || count == 0 || count > 16 {
        return Err("unsupported PE32+ section table");
    }
    let alignment = dword(optional + 32)?;
    let image_bytes = dword(optional + 56)? as u64;
    let headers = dword(optional + 60)? as usize;
    let section_table = optional + optional_bytes;
    if alignment < 4096
        || !alignment.is_power_of_two()
        || image_bytes != bytes.len() as u64
        || headers > bytes.len()
        || section_table + count * 40 > headers
    {
        return Err("invalid loaded PE geometry");
    }
    let mut result = LoadedSections {
        range: ImageRange {
            base,
            bytes: image_bytes,
        },
        sections: [EMPTY; 16],
        count,
    };
    for index in 0..count {
        let section = section_table + index * 40;
        let size = dword(section + 8)? as u64;
        let rva = dword(section + 12)? as u64;
        let flags = dword(section + 36)?;
        if size == 0 || rva < headers as u64 || !rva.is_multiple_of(alignment as u64) {
            return Err("invalid loaded PE section");
        }
        result.sections[index] = ImageSection {
            range: ImageRange {
                base: base.checked_add(rva).ok_or("PE address overflow")?,
                bytes: (size + 4095) & !4095,
            },
            permissions: Permissions {
                writable: flags & 0x8000_0000 != 0,
                executable: flags & 0x2000_0000 != 0,
            },
        };
    }
    KernelImage::new(result.range, &result.sections[..count])
        .map_err(|_| "invalid image permissions")?;
    if !result.sections[..count]
        .iter()
        .any(|s| s.permissions.executable)
    {
        return Err("no executable section");
    }
    Ok(result)
}
