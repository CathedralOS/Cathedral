//! Child process gates distinguish staged, durable and lost-acknowledgement states.
use super::{Result, adapters::Engine, trace::Batch};
use std::{
    io::{self, Write},
    path::Path,
};

fn records(value: u8) -> Batch {
    [b"head".as_slice(), b"event/last", b"object/value"]
        .into_iter()
        .map(|key| (key.to_vec(), vec![value; 8192]))
        .collect()
}

fn gate() {
    println!("READY");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    panic!("parent must terminate this process at the gate");
}

pub fn run(mode: &str, kind: &str, path: &Path, phase: &str) -> Result<()> {
    let mut engine = Engine::open(kind, path)?;
    match mode {
        "init" => engine.apply(&records(0), || {})?,
        "writer" => {
            assert!(phase == "staged" || phase == "committed");
            engine.apply(&records(1), || {
                if phase == "staged" {
                    gate();
                }
            })?;
            gate(); // Commit returned, but no application acknowledgement was sent.
        }
        "full" => {
            engine.apply(&records(0), || {})?;
            engine.checkpoint()?;
            engine.deny_growth(path)?;
            let mut too_large = records(1);
            too_large.push((b"oversized".to_vec(), vec![3; 16 * 1024 * 1024]));
            assert!(
                engine.apply(&too_large, || {}).is_err(),
                "capacity denial did not fail"
            );
            drop(engine);
            let mut reopened = Engine::open(kind, path)?;
            assert_eq!(
                reopened.scan(b"", &[255])?,
                records(0).into_iter().collect()
            );
            reopened.apply(&records(2), || {})?;
            assert_eq!(
                reopened.scan(b"", &[255])?,
                records(2).into_iter().collect()
            );
            println!("VERIFIED capacity denial and recovery");
        }
        "verify" => {
            let expected = if phase == "staged" { 0 } else { 1 };
            assert_eq!(
                engine.scan(b"", &[255])?,
                records(expected).into_iter().collect()
            );
            println!("VERIFIED");
        }
        _ => return Err("unknown recovery mode".into()),
    }
    Ok(())
}
