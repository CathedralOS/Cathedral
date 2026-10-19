use super::*;
#[test]
fn rejected_stage_cannot_publish_a_partial_batch() {
    let mut batch = Batch::default();
    batch.bind(7, 3);
    batch.begin(1).unwrap();
    let words = [wire::PAINT, 3, 9, 0, 0, 20, 20, 123];
    batch.stage(words, ()).unwrap();
    assert_eq!(batch.stage(words, ()), Err(abi::INVALID_ARGUMENT));
    assert_eq!(batch.prepare(1), Err(abi::INVALID_ARGUMENT));
    batch.abort(1).unwrap();
    assert!(batch.entries.iter().all(Option::is_none));
    batch.begin(2).unwrap();
    batch.stage(words, ()).unwrap();
    batch.committed(2);
    assert_eq!(batch.prepare(2), Ok(false));
    assert_eq!(batch.begin(2), Err(abi::DENIED));
    batch.bind(8, 3);
    assert_eq!(batch.prepare(2), Err(abi::DENIED));
    batch.begin(1).unwrap();
}
#[test]
fn capacity_and_epoch_changes_release_every_payload() {
    use std::{cell::Cell, rc::Rc};
    struct Reader(Rc<Cell<usize>>);
    impl Drop for Reader {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let released = Rc::new(Cell::new(0));
    let mut batch = Batch::default();
    batch.bind(7, 3);
    batch.begin(1).unwrap();
    for handle in 0..wire::PRESENT_LIMIT {
        batch
            .stage(
                [wire::PAINT, 3, handle as u64, 0, 0, 1, 1, 0],
                Reader(released.clone()),
            )
            .unwrap();
    }
    assert_eq!(
        batch.stage(
            [wire::PAINT, 3, wire::PRESENT_LIMIT as u64, 0, 0, 1, 1, 0],
            Reader(released.clone())
        ),
        Err(abi::NO_MEMORY)
    );
    assert_eq!(released.get(), 1);
    batch.bind(7, 4);
    assert_eq!(released.get(), wire::PRESENT_LIMIT + 1);
    assert_eq!(batch.prepare(1), Err(abi::DENIED));
}
