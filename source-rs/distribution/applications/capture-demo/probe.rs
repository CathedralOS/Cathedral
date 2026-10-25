//! Disposable recipient observer: inspect every delivered mask/padding byte.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{capture as wire, matrix, services};
use cathedral_user_runtime::{
    Error,
    capture::{Capture, Image},
};
pub fn refusals(capture: &mut Capture) -> Result<(), Error> {
    for words in [
        [wire::BEGIN, 99, 0, 0, 0, 0, 0, 0],
        services::request(cathedral_platform_contracts::imports::STORAGE, 1, 1, 0),
    ] {
        assert_eq!(capture.request(words)?, matrix::error(abi::DENIED));
    }
    Ok(())
}
pub fn image(image: &mut Image, capture: &mut Capture, ticket: u64) -> Result<(), Error> {
    assert_eq!((image.width, image.height, image.stride), (200, 104, 200));
    let bytes = image.object.map()?;
    assert_eq!(&bytes[..4], &0x9b7028u32.to_le_bytes());
    for y in 20..84 {
        for x in 10..190 {
            let at = (y * 200 + x) * 4;
            assert_eq!(&bytes[at..at + 4], &[0; 4]);
        }
    }
    assert!(bytes[200 * 104 * 4..].iter().all(|b| *b == 0));
    assert!(bytes[..200 * 104 * 4].chunks_exact(4).all(|p| p[3] == 0));
    image.object.unmap()?;
    assert!(matches!(capture.take(ticket),Err(Error(c)) if c==abi::DENIED as i64));
    cathedral_user_runtime::write(
        b"Cathedral capture: recipient verified redaction, padding and single delivery\n",
    )?;
    Ok(())
}
