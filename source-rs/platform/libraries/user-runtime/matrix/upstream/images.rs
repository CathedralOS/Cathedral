//! Retain sealed readers for replay; ordinary forwarding does not map pixels.
//! A capture/raster owner can explicitly map them for one bounded operation.
use super::*;
pub struct Images([Option<(u64, Object)>; wire::MAX_SURFACES]);
impl Default for Images {
    fn default() -> Self {
        Self::empty()
    }
}
impl Images {
    pub const fn empty() -> Self {
        Self([const { None }; wire::MAX_SURFACES])
    }
    pub fn get(&self, key: u64) -> Result<&Object, Error> {
        self.0
            .iter()
            .flatten()
            .find(|(k, _)| *k == key)
            .map(|(_, image)| image)
            .ok_or(Error(abi::BAD_HANDLE as i64))
    }
    /// A compositor/capture provider may map retained sealed readers while
    /// rasterizing. Ordinary recursive forwarding continues to use get unmapped.
    pub fn map(&mut self, key: u64) -> Result<(), Error> {
        self.0
            .iter_mut()
            .flatten()
            .find(|(k, _)| *k == key)
            .ok_or(Error(abi::BAD_HANDLE as i64))?
            .1
            .map()?;
        Ok(())
    }
    pub fn unmap(&mut self, key: u64) -> Result<(), Error> {
        let object = &mut self
            .0
            .iter_mut()
            .flatten()
            .find(|(k, _)| *k == key)
            .ok_or(Error(abi::BAD_HANDLE as i64))?
            .1;
        if object.bytes().is_some() {
            object.unmap()?;
        }
        Ok(())
    }
    pub fn room(&self, key: u64) -> Result<(), Error> {
        if self
            .0
            .iter()
            .any(|v| v.is_none() || v.as_ref().is_some_and(|(k, _)| *k == key))
        {
            Ok(())
        } else {
            Err(Error(abi::NO_MEMORY as i64))
        }
    }
    /// Call only after the upstream accepted a previously validated update.
    pub fn commit(&mut self, key: u64, object: Option<Object>) {
        let previous = self
            .0
            .iter()
            .position(|v| v.as_ref().is_some_and(|(k, _)| *k == key));
        if let Some(object) = object {
            let index = previous
                .or_else(|| self.0.iter().position(Option::is_none))
                .expect("one reader per live surface");
            self.0[index] = Some((key, object));
        } else if let Some(index) = previous {
            self.0[index] = None;
        }
    }
}
