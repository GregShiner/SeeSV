pub(crate) mod tests {
    use crate::external::*;

    // Helper to create test data that lives long enough
    pub(crate) struct TestData {
        pub(crate) int_data: Vec<i32>,
        pub(crate) float_data: Vec<f32>,
        pub(crate) string_data: Vec<u8>,
        pub(crate) offsets: Vec<usize>,
        pub(crate) lengths: Vec<usize>,
    }

    impl TestData {
        pub(crate) fn new_int(values: Vec<i32>) -> Self {
            TestData {
                int_data: values,
                float_data: Vec::new(),
                string_data: Vec::new(),
                offsets: Vec::new(),
                lengths: Vec::new(),
            }
        }

        pub(crate) fn new_float(values: Vec<f32>) -> Self {
            TestData {
                int_data: Vec::new(),
                float_data: values,
                string_data: Vec::new(),
                offsets: Vec::new(),
                lengths: Vec::new(),
            }
        }

        pub(crate) fn new_string(strings: Vec<&str>) -> Self {
            let mut data = Vec::new();
            let mut offsets = Vec::new();
            let mut lengths = Vec::new();

            for s in strings {
                offsets.push(data.len());
                lengths.push(s.len());
                data.extend_from_slice(s.as_bytes());
            }

            TestData {
                int_data: Vec::new(),
                float_data: Vec::new(),
                string_data: data,
                offsets,
                lengths,
            }
        }
    }

    #[test]
    fn test_int_subchunk_conversion() {
        let test_data = TestData::new_int(vec![1, 2, 3, 4, 5]);

        let external_subchunk = ExternalSubChunkView {
            data: test_data.int_data.as_ptr() as *const u8,
            offsets: std::ptr::null(),
            lengths: std::ptr::null(),
            num_of_items: test_data.int_data.len(),
        };

        unsafe {
            let subchunk: SubChunk<'_, IntValues> = SubChunk::from_external(&external_subchunk);
            assert_eq!(subchunk.values, &[1, 2, 3, 4, 5]);
        }
    }

    #[test]
    fn test_float_subchunk_conversion() {
        let test_data = TestData::new_float(vec![1.0, 2.5, 3.7]);

        let external_subchunk = ExternalSubChunkView {
            data: test_data.float_data.as_ptr() as *const u8,
            offsets: std::ptr::null(),
            lengths: std::ptr::null(),
            num_of_items: test_data.float_data.len(),
        };

        unsafe {
            let subchunk: SubChunk<'_, FloatValues> = SubChunk::from_external(&external_subchunk);
            assert_eq!(subchunk.values, &[1.0, 2.5, 3.7]);
        }
    }

    #[test]
    fn test_string_subchunk_conversion() {
        let test_data = TestData::new_string(vec!["hello", "world", "test"]);

        let external_subchunk = ExternalSubChunkView {
            data: test_data.string_data.as_ptr(),
            offsets: test_data.offsets.as_ptr(),
            lengths: test_data.lengths.as_ptr(),
            num_of_items: test_data.offsets.len(),
        };

        unsafe {
            let subchunk: SubChunk<'_, StringValues> = SubChunk::from_external(&external_subchunk);
            assert_eq!(subchunk.values, vec!["hello", "world", "test"]);
        }
    }

    #[test]
    fn test_int_chunk_conversion() {
        let test_data1 = TestData::new_int(vec![1, 2, 3]);
        let test_data2 = TestData::new_int(vec![4, 5, 6]);

        let external_subchunks = [
            ExternalSubChunkView {
                data: test_data1.int_data.as_ptr() as *const u8,
                offsets: std::ptr::null(),
                lengths: std::ptr::null(),
                num_of_items: test_data1.int_data.len(),
            },
            ExternalSubChunkView {
                data: test_data2.int_data.as_ptr() as *const u8,
                offsets: std::ptr::null(),
                lengths: std::ptr::null(),
                num_of_items: test_data2.int_data.len(),
            },
        ];

        let external_chunk = ExternalChunkView {
            sub_chunks: external_subchunks.as_ptr(),
            referenced_chunks: std::ptr::null(),
            num_of_sub_chunks: external_subchunks.len(),
            data_type: ExternalDataType::Int,
        };

        unsafe {
            let chunk: Chunk<'_, IntValues> = Chunk::from_external(&external_chunk);
            assert_eq!(chunk.sub_chunks.len(), 2);
            assert_eq!(chunk.sub_chunks[0].values, &[1, 2, 3]);
            assert_eq!(chunk.sub_chunks[1].values, &[4, 5, 6]);
        }
    }

    #[test]
    fn test_column_view_conversion() {
        let test_data = TestData::new_int(vec![10, 20, 30]);
        let column_name = "test_column";

        let external_subchunk = ExternalSubChunkView {
            data: test_data.int_data.as_ptr() as *const u8,
            offsets: std::ptr::null(),
            lengths: std::ptr::null(),
            num_of_items: test_data.int_data.len(),
        };

        let external_chunk = ExternalChunkView {
            sub_chunks: &external_subchunk as *const _,
            referenced_chunks: std::ptr::null(),
            num_of_sub_chunks: 1,
            data_type: ExternalDataType::Int,
        };

        let external_column = ExternalColumnView {
            name: column_name.as_ptr(),
            name_len: column_name.len(),
            chunks: &external_chunk as *const _,
            num_of_chunks: 1,
            data_type: ExternalDataType::Int,
        };

        unsafe {
            let column = ColumnView::from_external(&external_column);
            assert_eq!(column.name, "test_column");

            match &column.chunks {
                ChunkViews::Int(chunks) => {
                    assert_eq!(chunks.len(), 1);
                    assert_eq!(chunks[0].sub_chunks[0].values, &[10, 20, 30]);
                }
                _ => panic!("Expected Int chunks"),
            }
        }
    }

    #[test]
    fn test_table_view_conversion() {
        // Setup test data
        let int_data = TestData::new_int(vec![1, 2, 3]);
        let float_data = TestData::new_float(vec![1.5, 2.5]);
        let int_col_name = "int_col";
        let float_col_name = "float_col";

        // Create subchunks
        let int_subchunk = ExternalSubChunkView {
            data: int_data.int_data.as_ptr() as *const u8,
            offsets: std::ptr::null(),
            lengths: std::ptr::null(),
            num_of_items: int_data.int_data.len(),
        };

        let float_subchunk = ExternalSubChunkView {
            data: float_data.float_data.as_ptr() as *const u8,
            offsets: std::ptr::null(),
            lengths: std::ptr::null(),
            num_of_items: float_data.float_data.len(),
        };

        // Create chunks
        let int_chunk = ExternalChunkView {
            sub_chunks: &int_subchunk as *const _,
            referenced_chunks: std::ptr::null(),
            num_of_sub_chunks: 1,
            data_type: ExternalDataType::Int,
        };

        let float_chunk = ExternalChunkView {
            sub_chunks: &float_subchunk as *const _,
            referenced_chunks: std::ptr::null(),
            num_of_sub_chunks: 1,
            data_type: ExternalDataType::Float,
        };

        // Create columns
        let columns = [
            ExternalColumnView {
                name: int_col_name.as_ptr(),
                name_len: int_col_name.len(),
                chunks: &int_chunk as *const _,
                num_of_chunks: 1,
                data_type: ExternalDataType::Int,
            },
            ExternalColumnView {
                name: float_col_name.as_ptr(),
                name_len: float_col_name.len(),
                chunks: &float_chunk as *const _,
                num_of_chunks: 1,
                data_type: ExternalDataType::Float,
            },
        ];

        // Create table
        let external_table = ExternalTableView {
            columns: columns.as_ptr(),
            num_of_columns: columns.len(),
        };

        unsafe {
            let table = TableView::from_external(&external_table);
            assert_eq!(table.columns.len(), 2);
            assert_eq!(table.columns[0].name, "int_col");
            assert_eq!(table.columns[1].name, "float_col");
        }
    }

    // #[test]
    // fn test_empty_structures() {
    //     // Test empty subchunk
    //     let external_subchunk = ExternalSubChunkView {
    //         data: std::ptr::null(),
    //         offsets: std::ptr::null(),
    //         lengths: std::ptr::null(),
    //         num_of_items: 0,
    //     };
    //
    //     unsafe {
    //         let subchunk: SubChunk<'_, IntValues> = SubChunk::from_external(&external_subchunk);
    //         assert_eq!(subchunk.values.len(), 0);
    //     }
    // }

    #[test]
    fn test_string_with_empty_strings() {
        let test_data = TestData::new_string(vec!["hello", "", "world"]);

        let external_subchunk = ExternalSubChunkView {
            data: test_data.string_data.as_ptr(),
            offsets: test_data.offsets.as_ptr(),
            lengths: test_data.lengths.as_ptr(),
            num_of_items: test_data.offsets.len(),
        };

        unsafe {
            let subchunk: SubChunk<'_, StringValues> = SubChunk::from_external(&external_subchunk);
            assert_eq!(subchunk.values, vec!["hello", "", "world"]);
        }
    }

    #[test]
    fn test_string_with_unicode() {
        let test_data = TestData::new_string(vec!["Hello", "世界", "🦀"]);

        let external_subchunk = ExternalSubChunkView {
            data: test_data.string_data.as_ptr(),
            offsets: test_data.offsets.as_ptr(),
            lengths: test_data.lengths.as_ptr(),
            num_of_items: test_data.offsets.len(),
        };

        unsafe {
            let subchunk: SubChunk<'_, StringValues> = SubChunk::from_external(&external_subchunk);
            assert_eq!(subchunk.values, vec!["Hello", "世界", "🦀"]);
        }
    }
}
