//! Inspect the real host-formatted redb envelope and opaque engine bytes, then
//! exercise unaligned IO in a separate scratch region. No guest database runtime.
use cathedral_ata_pio::Ata;
use cathedral_storage_volume::{
    BlockRegion,
    header::{BLOCK, DATA_START},
    inspect,
};

const SIZE: u64 = 4 * 1024 * 1024;
const SCRATCH: u64 = BLOCK + SIZE + BLOCK;

pub fn run() -> u64 {
    let mut disk = Ata::open().unwrap();
    let mut control = [0; 264];
    BlockRegion::new(&mut disk, SCRATCH, BLOCK)
        .unwrap()
        .read(0, &mut control)
        .unwrap();
    let phase = u64::from_le_bytes(control[256..264].try_into().unwrap());
    assert!(phase <= 2, "unexpected fixture phase");
    // Scope releases the exclusive borrow before opening the scratch region.
    {
        let mut volume = BlockRegion::new(&mut disk, BLOCK, SIZE).unwrap();
        let decoded = inspect(&mut volume);
        if phase == 2 {
            assert!(decoded.is_err(), "corrupt envelope accepted");
            return 2;
        }
        let header = decoded.unwrap();
        assert!(header.ready && header.length >= 8192);
        assert_eq!(u128::from_be_bytes(header.id), 1);
        assert_eq!(header.region_bytes, SIZE);
        opaque_io(&mut volume, &control[..256], phase);
    }
    scratch(&mut disk, phase);
    phase
}

// Separate the IO buffer's lifetime from envelope decoding on the bounded user stack.
fn opaque_io(volume: &mut BlockRegion<'_, Ata>, expected: &[u8], phase: u64) {
    let mut block = [0; BLOCK as usize];
    volume.read(DATA_START, &mut block[..256]).unwrap();
    assert_eq!(&block[..256], expected);
    assert!(volume.write(SIZE - 1, &[0, 0]).is_err());
    assert!(volume.read(u64::MAX, &mut [0]).is_err());
    // Rewrite exactly one unchanged envelope copy through actual sector IO.
    // The host later compares the complete volume and reopens redb itself.
    if phase == 0 {
        volume.read(0, &mut block).unwrap();
        volume.write(0, &block).unwrap();
        volume.flush().unwrap();
    }
}

fn scratch(disk: &mut Ata, phase: u64) {
    let mut region = BlockRegion::new(disk, SCRATCH, BLOCK).unwrap();
    let mut bytes = [0; BLOCK as usize];
    region.read(0, &mut bytes).unwrap();
    if phase == 0 {
        assert!(bytes[264..].iter().all(|b| *b == 0xc3));
        region.write(509, &[0x37; 1031]).unwrap();
        region.write(BLOCK - 1, &[0x58]).unwrap();
        region.write(256, &1u64.to_le_bytes()).unwrap();
        region.flush().unwrap(); // Success is reported only after FLUSH CACHE.
    } else {
        assert!(bytes[264..509].iter().all(|b| *b == 0xc3));
        assert!(bytes[509..1540].iter().all(|b| *b == 0x37));
        assert!(bytes[1540..4095].iter().all(|b| *b == 0xc3));
        assert_eq!(bytes[4095], 0x58);
    }
}
