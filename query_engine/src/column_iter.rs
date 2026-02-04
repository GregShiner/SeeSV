use std::iter::FlatMap;

use crate::external::{Chunk, IntValues, SubChunk};

enum ThunkedChunk<T: Sized, I: IntoIterator<Item = T>> {
    Unevaluated(I),
    Evaluated(Vec<T>),
}

impl<T: Sized, I: std::iter::IntoIterator<Item = T>> ThunkedChunk<T, I> {
    fn read(&mut self) -> &[T] {
        // If self hasnt been evaluated yet
        if let ThunkedChunk::Unevaluated(iterator) = self {
            // Evaluate the iterator and collect the result into a vector
            let items: Vec<T> = iterator.into_iter.collect(); // TODO: realloc go brrr
            *self = ThunkedChunk::Evaluated(items);
        }
        // Else, self has already been evaluated

        match self {
            ThunkedChunk::Evaluated(items) => items.as_slice(),
            _ => unreachable!(), // SAFETY: If self was Unevaluated, it will be set to Evaluated
        }
    }
}

impl<'a> IntoIterator for Chunk<'a, IntValues<'a>> {
    type Item = i32;

    type IntoIter = FlatMap;

    fn into_iter(
        self,
    ) -> FlatMap<
        std::slice::Iter<'a, SubChunk<'a, &'a [i32]>>,
        &'a [i32],
        impl FnMut(&crate::external::SubChunk<'a, &'a [i32]>) -> &'a [i32],
    > {
        self.sub_chunks.iter().flat_map(|s| s.values)
    }
}
/* impl<'a> Iterator for Chunk<'a, IntValues<'a>> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        /*
         *
         * */
        // if
        // if let Some(sub_chunk) = self.sub_chunks.get(self.current_sub_chunk) {
        //     if let Some(value) = sub_chunk.values.get(self.current_index) {
        //         self.current_index += 1;
        //         return Some(value);
        //     } else {
        //         self.current_sub_chunk += 1;
        //         self.current_index = 0;
        //
        //     }
        // } else {
        //     self.current_sub_chunk += 1;
        //     self.current_index = 0;
        // }
    }
} */
