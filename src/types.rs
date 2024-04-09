#[repr(u8)]
#[derive(Debug)]
pub enum KeyMap {
    SpiRst = 27,
    SpiDc = 25,
    Backlight = 24,
    KeyUp = 6,
    KeyDown = 19,
    KeyLeft = 5,
    KeyRight = 26,
    KeyPress = 13,
    KeyOk = 21,
    KeyMain = 20,
    KeyCancel = 16,
}
