//! A bridge trait to unify required vector operations over multiple implementations

use crate::err;
use crate::error::{Memory, MemoryError};
use core::cmp;

/// A bridge trait to unify required vector operations over multiple implementations
pub trait AnyVec<T>
where
    Self: Default + AsRef<[T]> + AsMut<[T]> + IntoIterator<Item = T>,
{
    /// Creates a new vector by copying the given elements
    fn new(elements: &[T]) -> Result<Self, MemoryError>
    where
        T: Clone,
    {
        // Init self and copy elements
        let mut this = Self::default();
        this.extend(elements)?;
        Ok(this)
    }
    /// Extends the vector from the given elements
    fn extend(&mut self, elements: &[T]) -> Result<(), MemoryError>
    where
        T: Clone;

    /// Inserts the given element at the given index
    fn insert(&mut self, index: usize, element: T) -> Result<(), MemoryError>;
    /// Pushes an element to the end of the vector
    fn push(&mut self, element: T) -> Result<(), MemoryError> {
        self.insert(self.as_ref().len(), element)
    }
}
// Implement `AnyVec` for `Vec<u8>` if `std` is enabled
#[cfg(feature = "std")]
impl<T> AnyVec<T> for std::vec::Vec<T> {
    fn insert(&mut self, index: usize, element: T) -> Result<(), MemoryError> {
        // Limit index and allocate slot
        let index = cmp::min(index, self.len());
        self.try_reserve(1).map_err(|_| err!(Memory, "failed to allocate memory"))?;

        // Insert element
        self.insert(index, element);
        Ok(())
    }

    fn extend(&mut self, elements: &[T]) -> Result<(), MemoryError>
    where
        T: Clone,
    {
        // Allocate capacity and extend vector
        self.try_reserve(elements.len()).map_err(|_| err!(Memory, "failed to allocate memory"))?;
        self.extend_from_slice(elements);
        Ok(())
    }
}
// Implement `AnyVec` for `arrayvec::ArrayVec<T, CAP>` if `arrayvec` is enabled
#[cfg(feature = "arrayvec")]
impl<T, const CAP: usize> AnyVec<T> for arrayvec::ArrayVec<T, CAP> {
    fn insert(&mut self, index: usize, element: T) -> Result<(), MemoryError> {
        // Limit index and insert element
        let index = cmp::min(index, self.len());
        self.try_insert(index, element).map_err(|_| err!(Memory, "not enough memory"))
    }

    fn extend(&mut self, elements: &[T]) -> Result<(), MemoryError>
    where
        T: Clone,
    {
        // Extend vector
        for element in elements.iter().cloned() {
            // Push each element
            self.try_push(element).map_err(|_| err!(Memory, "not enough memory"))?;
        }
        Ok(())
    }
}
// Implement `AnyVec` for `heapless::Vec<T, CAP>` if `heapless` is enabled
#[cfg(feature = "heapless")]
impl<T, const CAP: usize> AnyVec<T> for heapless::Vec<T, CAP> {
    fn insert(&mut self, index: usize, element: T) -> Result<(), MemoryError> {
        // Limit index and insert element
        let index = cmp::min(index, self.len());
        self.insert(index, element).map_err(|_| err!(Memory, "not enough memory"))
    }

    fn extend(&mut self, elements: &[T]) -> Result<(), MemoryError>
    where
        T: Clone,
    {
        // Extend vector
        for element in elements.iter().cloned() {
            // Push each element
            self.push(element).map_err(|_| err!(Memory, "not enough memory"))?;
        }
        Ok(())
    }
}
