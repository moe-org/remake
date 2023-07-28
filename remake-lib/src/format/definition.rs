
/// The platforam code.
#[repr(u64)]
pub enum Platform {
    Window = 0,
    Unix = 1,
    Mac = 2,
    Freebsd = 3,
}

impl TryFrom<u64> for Platform {
    type Error = ();

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            x if x == Platform::Window as u64 => Ok(Platform::Window),
            x if x == Platform::Unix as u64 => Ok(Platform::Unix),
            x if x == Platform::Mac as u64 => Ok(Platform::Mac),
            _ => Err(()),
        }
    }
}
