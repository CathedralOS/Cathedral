// Explicit lifecycle: validate assigned space, initialize an engine while marked
// formatting, durably close it, then publish ready. Opening never formats.
use super::Result;
use backend::Backend;
use header::{Header, geometry};
use image::Image;
use redb::{Database, ReadableDatabase, StorageBackend, TableDefinition};
use region::Region;
use std::{path::Path, sync::Arc};

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("volume-lab");
const ID_KEY: &[u8] = b"volume-id";

pub fn run(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("verify-catalog") | Some("verify-catalog-recovery") if args.len() == 4 => catalog::verify(Path::new(&args[1]), args[2].parse()?, args[3].parse()?, args[0] == "verify-catalog-recovery"),
        Some("verify-objects") | Some("verify-retention") if args.len() == 5 => objects::verify(Path::new(&args[1]), args[2].parse()?, args[3].parse()?, args[4].parse()?, args[0] == "verify-retention"),
        Some("verify-guest") if args.len() == 5 => guest::verify(Path::new(&args[1]), args[2].parse()?, args[3].parse()?, args[4].parse()?),
        Some("prove") if args.len() == 2 => proof::run(Path::new(&args[1])),
        Some("format") if args.len() == 4 => {
            let bytes = args[2].parse()?;
            geometry(bytes)?;
            let id = u128::from_str_radix(&args[3], 16)?.to_be_bytes();
            if id == [0; 16] { return Err("volume ID must be nonzero".into()); }
            let image = Arc::new(Image::create(Path::new(&args[1]), bytes)?);
            format(Region::new(image, 0, bytes)?, id)?;
            println!("FORMATTED experimental volume; bytes={bytes} id={}", args[3]);
            Ok(())
        }
        Some("open") | Some("inspect") if args.len() == 4 => {
            let image = Arc::new(Image::open(Path::new(&args[1]))?);
            let region = Region::new(image, args[2].parse()?, args[3].parse()?)?;
            let header = region.headers()?;
            if args[0] == "open" { let _db = open(region)?; }
            println!("VOLUME ready={} generation={} capacity={} logical_length={} id={:032x}", header.ready, header.generation, header.capacity(), header.length, u128::from_be_bytes(header.id));
            Ok(())
        }
        _ => Err("usage: engines volume prove NEW_DIRECTORY | format NEW_IMAGE BYTES ID_HEX | open IMAGE OFFSET BYTES | inspect IMAGE OFFSET BYTES | verify-guest/verify-objects/verify-retention IMAGE OFFSET BYTES REVISION | verify-catalog/verify-catalog-recovery IMAGE OFFSET BYTES".into()),
    }
}

fn format(region: Region, id: [u8; 16]) -> Result<()> {
    if id == [0; 16] {
        return Err("volume ID must be nonzero".into());
    }
    region.blank()?; // Inspect the whole range before the first mutation.
    let header = Header {
        id,
        generation: 0,
        region_bytes: region.bytes,
        length: 0,
        ready: false,
    };
    region.publish_header(&header)?;
    let backend = Backend::new(region, header);
    let db = Database::builder()
        .set_cache_size(1024 * 1024)
        .create_with_backend(backend.clone())?;
    let mut tx = db.begin_write()?;
    tx.set_durability(redb::Durability::Immediate)?;
    tx.open_table(TABLE)?.insert(ID_KEY, id.as_slice())?;
    tx.commit()?;
    drop(db); // Includes any engine close-time housekeeping while still formatting.
    backend.sync_data()?;
    backend.mark_ready()?;
    Ok(())
}

fn open(region: Region) -> Result<Database> {
    let header = region.headers()?;
    if !header.ready || header.length == 0 {
        return Err("volume formatting is incomplete".into());
    }
    let id = header.id;
    // redb's custom backend entry also permits creation, but it creates only at
    // length zero. The ready envelope above forbids that state; nonempty invalid
    // engine headers are rejected by redb, never treated as a new database.
    let db = Database::builder()
        .set_cache_size(1024 * 1024)
        .create_with_backend(Backend::new(region, header))?;
    {
        let read = db.begin_read()?;
        let table = read.open_table(TABLE)?;
        let found = table.get(ID_KEY)?.ok_or("missing volume identity record")?;
        if found.value() != id {
            return Err("envelope/engine identity mismatch".into());
        }
    }
    Ok(db)
}
