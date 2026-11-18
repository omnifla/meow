use meow::reactive::{take_dirty, Signal};

#[test]
fn set_and_get_round_trip() {
    let signal = Signal::new(1);
    signal.set(5);
    assert_eq!(signal.get(), 5);
}

#[test]
fn clones_share_state() {
    let a = Signal::new(0);
    let b = a.clone();
    b.update(|value| *value += 2);
    assert_eq!(a.get(), 2);
}

#[test]
fn writes_mark_dirty_until_taken() {
    let signal = Signal::new(0);
    assert!(!take_dirty());
    signal.set(1);
    assert!(take_dirty());
    assert!(!take_dirty());
}
