//! Platforms without a backend yet (Windows DXGI backend is planned). Reports nothing.

use crate::OsReadings;

pub struct Reader;

impl Reader {
    pub fn new() -> Self {
        Self
    }

    pub fn read(&mut self) -> OsReadings {
        OsReadings::default()
    }
}
