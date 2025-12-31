#[cfg(test)]
mod proptests {
    use crate::external::*;
    use crate::tests::test_external::tests::TestData;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_int_roundtrip(values in prop::collection::vec(any::<i32>(), 0..100)) {
            let test_data = TestData::new_int(values.clone());

            let external_subchunk = ExternalSubChunkView {
                data: test_data.int_data.as_ptr() as *const u8,
                offsets: std::ptr::null(),
                lengths: std::ptr::null(),
                num_of_items: test_data.int_data.len(),
            };

            unsafe {
                let subchunk: SubChunk<'_, IntValues> = SubChunk::from_external(&external_subchunk);
                prop_assert_eq!(subchunk.values, values.as_slice());
            }
        }
    }

    proptest! {
        #[test]
        fn test_string_conversion(
            strings in prop::collection::vec(any::<String>(), 0..50)
        ) {
            let str_refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
            let test_data = TestData::new_string(str_refs.clone());

            let external_subchunk = ExternalSubChunkView {
                data: test_data.string_data.as_ptr(),
                offsets: test_data.offsets.as_ptr(),
                lengths: test_data.lengths.as_ptr(),
                num_of_items: test_data.offsets.len(),
            };

            unsafe {
                let subchunk = SubChunk::from_external(&external_subchunk);
                let result: Vec<&str> = subchunk.values;
                prop_assert_eq!(result, str_refs);
            }
        }

        #[test]
        fn test_float_special_values(
            values in prop::collection::vec(
                prop::num::f32::ANY,
                0..100
            )
        ) {
            let test_data = TestData::new_float(values.clone());

            let external_subchunk = ExternalSubChunkView {
                data: test_data.float_data.as_ptr() as *const u8,
                offsets: std::ptr::null(),
                lengths: std::ptr::null(),
                num_of_items: test_data.float_data.len(),
            };

            unsafe {
                let subchunk: SubChunk<'_, FloatValues> = SubChunk::from_external(&external_subchunk);
                prop_assert_eq!(subchunk.values.len(), values.len());

                // Check NaN values separately (NaN != NaN)
                for (a, b) in subchunk.values.iter().zip(values.iter()) {
                    if a.is_nan() {
                        prop_assert!(b.is_nan());
                    } else {
                        prop_assert_eq!(a, b);
                    }
                }
            }
        }

        #[test]
        fn test_multiple_subchunks(
            subchunk_sizes in prop::collection::vec(0usize..20, 1..10)
        ) {
            let mut all_test_data = Vec::new();
            let mut external_subchunks = Vec::new();

            for &size in &subchunk_sizes {
                let values: Vec<i32> = (0..size as i32).collect();
                let test_data = TestData::new_int(values);

                external_subchunks.push(ExternalSubChunkView {
                    data: test_data.int_data.as_ptr() as *const u8,
                    offsets: std::ptr::null(),
                    lengths: std::ptr::null(),
                    num_of_items: test_data.int_data.len(),
                });

                all_test_data.push(test_data);
            }

            let external_chunk = ExternalChunkView {
                sub_chunks: external_subchunks.as_ptr(),
                referenced_chunks: std::ptr::null(),
                num_of_sub_chunks: external_subchunks.len(),
                data_type: ExternalDataType::Int,
            };

            unsafe {
                let chunk: Chunk<'_, IntValues> = Chunk::from_external(&external_chunk);
                prop_assert_eq!(chunk.sub_chunks.len(), subchunk_sizes.len());

                for (i, &expected_size) in subchunk_sizes.iter().enumerate() {
                    prop_assert_eq!(chunk.sub_chunks[i].values.len(), expected_size);
                }
            }
        }
    }
}
