//! Keep: checked time units.
//! Replace: the application's input format.
use skein_lib::Wall;
/// Checked conversion of an entered epoch-seconds value.
#[must_use]
pub fn wall(seconds: u64) -> Option<Wall> {
    Some(Wall::from_nanos(seconds.checked_mul(1_000_000_000)?))
}
/// UTC time of day, for the drawing's numeric fallback.
#[must_use]
pub fn clock(wall: Wall) -> [u8; 5] {
    let seconds = wall.as_secs().rem_euclid(86_400);
    let hour = seconds.div_euclid(3600);
    let minute = seconds.rem_euclid(3600).div_euclid(60);
    [
        digit(hour.div_euclid(10)),
        digit(hour.rem_euclid(10)),
        b':',
        digit(minute.div_euclid(10)),
        digit(minute.rem_euclid(10)),
    ]
}
fn digit(n: u64) -> u8 {
    b'0'.checked_add(u8::try_from(n).expect("clock digit fits")).expect("clock digit fits ASCII")
}
