use crate::external::{Chunk, FloatValues, IntValues, StringValues};

enum ThunkedChunk<T: Sized, I: IntoIterator<Item = T>> {
    Unevaluated(I),
    Evaluated(Vec<T>),
}

impl<T: Sized, I: std::iter::IntoIterator<Item = T>> ThunkedChunk<T, I> {
    fn read(&mut self) -> &[T] {
        // If self hasnt been evaluated yet
        if let ThunkedChunk::Unevaluated(_) = self {
            // Take ownership of the iterator, replacing it with a temporary empty vec
            let iterator = std::mem::replace(self, ThunkedChunk::Evaluated(Vec::new()));

            // Now we can consume the iterator
            if let ThunkedChunk::Unevaluated(iter) = iterator {
                let items: Vec<T> = iter.into_iter().collect();
                *self = ThunkedChunk::Evaluated(items);
            }
        }
        // Else, self has already been evaluated

        match self {
            ThunkedChunk::Evaluated(items) => items.as_slice(),
            _ => unreachable!(), // SAFETY: If self was Unevaluated, it will be set to Evaluated
        }
    }
}

impl<'a> IntoIterator for Chunk<'a, IntValues<'a>> {
    type Item = &'a i32;

    type IntoIter = Box<dyn Iterator<Item = &'a i32> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.sub_chunks.into_iter().flat_map(|s| s.values.iter()))
    }
}

impl<'a> IntoIterator for Chunk<'a, FloatValues<'a>> {
    type Item = &'a f32;
    type IntoIter = Box<dyn Iterator<Item = &'a f32> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.sub_chunks.into_iter().flat_map(|s| s.values.iter()))
    }
}

impl<'a> IntoIterator for Chunk<'a, StringValues<'a>> {
    type Item = &'a str;
    type IntoIter = Box<dyn Iterator<Item = &'a str> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            self.sub_chunks
                .into_iter()
                .flat_map(|s| s.values.into_iter()),
        )
    }
}
