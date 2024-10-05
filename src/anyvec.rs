//! A bridge trait to unify required vector operations over multiple implementations

/// A bridge trait to unify required vector operations over multiple implementations
pub trait AnyVec<T>
where
    Self: Default + AsRef<[T]> + AsMut<[T]> + IntoIterator<Item = T>,
{
    /// Creates a new vector by copying the given elements
    fn new(elements: &[T]) -> Result<Self, &'static str>
    where
        T: Clone,
    {
        // Init self and copy elements
        let mut this = Self::default();
        this.extend(elements)?;
        Ok(this)
    }
    /// Extends the vector from the given elements
    fn extend(&mut self, elements: &[T]) -> Result<(), &'static str>
    where
        T: Clone;

    /// Inserts the given element at the given index
    fn insert(&mut self, index: usize, element: T) -> Result<(), &'static str>;
    /// Pushes an element to the end of the vector
    fn push(&mut self, element: T) -> Result<(), &'static str> {
        self.insert(self.as_ref().len(), element)
    }
}
// Implement `AnyVec` for `Vec<u8>` if `std` is enabled
#[cfg(feature = "std")]
impl<T> AnyVec<T> for std::vec::Vec<T> {
    fn insert(&mut self, index: usize, element: T) -> Result<(), &'static str> {
        // Validate index
        let true = index <= self.len() else {
            return Err("Index is invalid");
        };

        // Allocate capacity and insert element
        self.try_reserve(1).map_err(|_| "Failed to alocate memory")?;
        self.insert(index, element);
        Ok(())
    }

    fn extend(&mut self, elements: &[T]) -> Result<(), &'static str>
    where
        T: Clone,
    {
        // Allocate capacity and extend vector
        self.try_reserve(elements.len()).map_err(|_| "Failed to alocate memory")?;
        self.extend_from_slice(elements);
        Ok(())
    }
}
// Implement `AnyVec` for `Vec<u8>` if `std` is enabled
#[cfg(feature = "arrayvec")]
impl<T, const CAP: usize> AnyVec<T> for arrayvec::ArrayVec<T, CAP> {
    fn insert(&mut self, index: usize, element: T) -> Result<(), &'static str> {
        // Insert element
        self.try_insert(index, element).map_err(|_| "Index is invalid")
    }

    fn extend(&mut self, elements: &[T]) -> Result<(), &'static str>
    where
        T: Clone,
    {
        // Extend vector
        for element in elements.iter().cloned() {
            // Push each element
            self.try_push(element).map_err(|_| "Not enough memory")?;
        }
        Ok(())
    }
}
