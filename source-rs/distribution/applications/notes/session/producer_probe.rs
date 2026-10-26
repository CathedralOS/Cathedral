//! Disposable desktop-test build only. Exercise the real broker without
//! changing the installed Notes manifest or granting ordinary apps production.
use super::*;
#[cfg(not(feature = "input-producer-partial"))]
use cathedral_platform_contracts::services as broker;
pub fn run(services: &mut Services) {
    let view = call(services, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(view[0], 0);
    #[cfg(feature = "input-producer-partial")]
    {
        match services.acquire_endpoint_part(
            cathedral_platform_contracts::imports::INPUT_PRODUCER,
            1,
            1,
            0,
        ) {
            Ok(endpoint) => {
                // The reply receiver has not transferred. Early traffic must not
                // produce input or fill that retained receiver's queue.
                endpoint
                    .send(&wire::encode([wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0]))
                    .unwrap();
                // Keep this right until ordinary app death. Its reply partner is
                // still retained by the offer, exercising interrupted acquisition.
                cathedral_user_runtime::write(
                    b"Cathedral producer: partial acquisition retained\n",
                )
                .unwrap();
            }
            Err(Error(code)) if code == abi::NO_MEMORY as i64 || code == abi::BUSY as i64 => {
                // A complete refusal is not an ambiguous reply. The runtime must
                // leave the shared interaction/broker connection usable.
                assert!(matches!(
                    services.input_producer(1, 1),
                    Err(Error(code)) if code == abi::NO_MEMORY as i64 || code == abi::BUSY as i64
                ));
                cathedral_user_runtime::write(
                    b"Cathedral producer: capacity refused; broker retained\n",
                )
                .unwrap();
            }
            Err(error) => panic!("partial producer acquisition: {error:?}"),
        }
    }
    #[cfg(not(feature = "input-producer-partial"))]
    match services.input_producer(1, 1) {
        Err(Error(code)) if code == broker::UNAVAILABLE as i64 => {
            cathedral_user_runtime::write(b"Cathedral producer: undelegated acquisition refused\n")
                .unwrap();
        }
        Ok(pair) => {
            let again = services.input_producer(1, 1).unwrap();
            assert_eq!((again.0.raw(), again.1.raw()), (pair.0.raw(), pair.1.raw()));
            let mut client = Client::connect(pair, time::after(300).unwrap()).unwrap();
            let mut producer = |words| {
                let (bytes, n) = client.call_retained(&wire::encode(words)).unwrap();
                wire::decode(&bytes[..n]).unwrap()
            };
            let info = producer([wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0]);
            assert_eq!(info[0], 0);
            assert_eq!(&info[5..7], &[2, wire::PRODUCER_FOCUS]);
            for op in [wire::VIEW, wire::STORAGE_INFO, wire::INPUT_GROUP_CREATE] {
                assert_eq!(producer([op, 0, 0, 0, 0, 0, 0, 0])[0], abi::DENIED);
            }
            assert_eq!(
                call(services, [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0])[0],
                abi::DENIED
            );
            assert_eq!(
                producer([wire::PRODUCER_POINTER, info[1], 3, 1, 1, 0, 0, 0])[0],
                abi::DENIED
            );
            assert_eq!(
                producer([wire::PRODUCER_POINTER, info[1] + 1, 1, 1, 1, 0, 0, 0])[0],
                abi::DENIED
            );
            assert_eq!(
                producer([
                    wire::PRODUCER_POINTER,
                    info[1],
                    1,
                    info[2] / 2,
                    info[3] / 2,
                    0,
                    0,
                    0
                ]),
                [0; 8]
            );
            client.retire();
            cathedral_user_runtime::write(
                b"Cathedral producer: acquired isolated connection and closed\n",
            )
            .unwrap();
        }
        Err(error) => panic!("producer acquisition: {error:?}"),
    }
    assert_eq!(call(services, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]), view);
}
