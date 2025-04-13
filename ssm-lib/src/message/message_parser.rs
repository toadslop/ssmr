use super::ClientMessage;

impl ClientMessage {
    /// TODO: details about the serialization logic
    ///
    /// ## Errors
    ///
    /// TODO: document possible serialization errors
    pub fn serialize(&self) -> Result<Vec<u8>, crate::Error> {
        // TODO: Implement serialization logic
        Ok(vec![])
    }
}

/// putString puts a string value to a byte array starting from the specified offset.  (comment from original)
// TODO: what is the purpose of offset end? why not just go to the end of the input?
#[allow(dead_code)]
fn put_string(
    byte_array: &mut [u8],
    offset_start: usize,
    offset_end: usize,
    input_string: &str,
) -> Result<(), Error> {
    let byte_array_length = byte_array.len();

    // TODO: consider making these debug asserts
    if byte_array_length == 0
        || offset_start > byte_array_length - 1
        || offset_end > byte_array_length - 1
        || offset_start > offset_end
    {
        log::error!("put_string failed: Offset is invalid.");
        Err(Error::OffsetOutOfBounds)?;
    }

    if offset_end - offset_start + 1 < input_string.len() {
        log::error!("put_string failed: Not enough space to save the string");
        Err(Error::BufferTooSmall)?;
    }

    // wipe out the array location first and then insert the new value. (comment from original)
    // TODO: why is this necessary? Can't we just overwrite without clearing first?
    byte_array
        .iter_mut()
        .skip(offset_start)
        .take(offset_end)
        .for_each(|byte| *byte = b' ');

    byte_array[offset_start..(offset_start + input_string.len())]
        .copy_from_slice(input_string.as_bytes());

    Ok(())
}

#[allow(dead_code)]
fn put_bytes(
    byte_array: &mut [u8],
    offset_start: usize,
    offset_end: usize,
    input_bytes: &[u8],
) -> Result<(), Error> {
    let byte_array_length = byte_array.len();

    // TODO: consider making these debug asserts
    // TODO: fix code duplication of offset validation
    if byte_array_length == 0
        || offset_start > byte_array_length - 1
        || offset_end > byte_array_length - 1
        || offset_start > offset_end
    {
        log::error!("put_bytes failed: Offset is invalid.");
        Err(Error::OffsetOutOfBounds)?;
    }

    if offset_end - offset_start + 1 < input_bytes.len() {
        log::error!("put_bytes failed: Not enough space to save the string");
        Err(Error::BufferTooSmall)?;
    }

    byte_array[offset_start..(offset_start + input_bytes.len())].copy_from_slice(input_bytes);

    Ok(())
}

/// TODO: document
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// TODO: document
    #[error("Offset is outside the byte array.")]
    OffsetOutOfBounds,

    /// TODO: document
    #[error("Not enough space to save the string.")]
    BufferTooSmall,
}

#[cfg(test)]
mod test {
    use std::fmt::Debug;

    #[derive(Debug)]
    struct TestParams<I> {
        name: &'static str,
        byte_array: Vec<u8>,
        offset_start: usize,
        offset_end: usize,
        input: I,
        expected: Result<(), super::Error>,
    }

    fn get_n_byte_buffer(size: usize) -> Vec<u8> {
        vec![0; size]
    }

    fn get_8_byte_buffer() -> Vec<u8> {
        get_n_byte_buffer(8)
    }

    // fn get_16_byte_buffer() -> Vec<u8> {
    //     get_n_byte_buffer(16)
    // }

    fn default_byte_buffer_generator() -> Vec<u8> {
        get_8_byte_buffer()
    }

    trait HasLen {
        fn len(&self) -> usize;
    }

    impl HasLen for &str {
        fn len(&self) -> usize {
            str::len(self)
        }
    }

    impl<T> HasLen for &[T] {
        fn len(&self) -> usize {
            <[T]>::len(self)
        }
    }

    impl<T, const N: usize> HasLen for [T; N] {
        fn len(&self) -> usize {
            N
        }
    }

    trait AsBytes {
        fn as_bytes(&self) -> &[u8];
    }

    impl AsBytes for &str {
        fn as_bytes(&self) -> &[u8] {
            str::as_bytes(self)
        }
    }

    impl AsBytes for &[u8] {
        fn as_bytes(&self) -> &[u8] {
            self
        }
    }

    #[derive(Debug, Default)]
    struct TestSuite<Input>
    where
        Input: Default,
    {
        test_cases: Vec<TestParams<Input>>,
    }

    impl<Input> TestSuite<Input>
    where
        Input: Clone + Debug + Default + HasLen + AsBytes,
    {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add_test_case(mut self, test_case: TestParams<Input>) -> Self {
            self.test_cases.push(test_case);
            self
        }

        pub fn execute(
            self,
            mut test_func: impl FnMut(&mut [u8], usize, usize, Input) -> Result<(), super::Error>,
            additional_assertions: impl Fn(&[u8], usize, usize, Input, &str),
        ) {
            for case in self.test_cases {
                let TestParams::<Input> {
                    name,
                    mut byte_array,
                    offset_start,
                    offset_end,
                    input,
                    expected,
                } = case;
                let original_byte_array = byte_array.clone();
                let result = test_func(&mut byte_array, offset_start, offset_end, input.clone());

                assert_eq!(result, expected);

                if result.is_ok() {
                    let byte_array_start = original_byte_array[0..offset_start].to_vec();
                    let byte_array_end = original_byte_array[(offset_end + 1)..].to_vec();

                    assert_eq!(
                        &byte_array[offset_start..(offset_start + input.len())],
                        input.as_bytes(),
                        "{name}: bytes within range should be input value"
                    );

                    assert_eq!(
                        byte_array_start,
                        original_byte_array[0..offset_start],
                        "{name}: bytes before offset should not be changed"
                    );

                    assert_eq!(
                        byte_array_end,
                        original_byte_array[(offset_end + 1)..],
                        "{name}: bytes after offset should not be changed"
                    );

                    additional_assertions(&byte_array, offset_start, offset_end, input, name);
                }
            }
        }
    }

    #[test]
    fn put_string() {
        TestSuite::new()
            .add_test_case(TestParams {
                name: "basic",
                byte_array: default_byte_buffer_generator(),
                offset_start: 0,
                offset_end: 7,
                input: "hello",
                expected: Ok(()),
            })
            .add_test_case(TestParams {
                name: "basic offset",
                byte_array: default_byte_buffer_generator(),
                offset_start: 1,
                offset_end: 7,
                input: "hello",
                expected: Ok(()),
            })
            .add_test_case(TestParams {
                name: "Data too long for buffer",
                byte_array: default_byte_buffer_generator(),
                offset_start: 0,
                offset_end: 7,
                input: "longinputstring",
                expected: Err(super::Error::BufferTooSmall),
            })
            .add_test_case(TestParams {
                name: "Size 0 buffer",
                byte_array: get_n_byte_buffer(0),
                offset_start: 0,
                offset_end: 7,
                input: "longinputstring",
                expected: Err(super::Error::OffsetOutOfBounds),
            })
            .execute(
                super::put_string,
                |byte_array, offset_start, offset_end, input, name| {
                    let trailing = byte_array[(offset_start + input.len())
                        ..(offset_start + input.len() + offset_end - input.len())]
                        .to_vec();

                    let trailing_should_be = " ".repeat(offset_end - input.len());

                    assert_eq!(
                        trailing,
                        trailing_should_be.as_bytes(),
                        "{name}: any leftofter offset should be \\s character"
                    );
                },
            );
    }

    // TODO: refactor these testing logics to remove duplication.
    #[test]
    fn put_bytes() {
        TestSuite::new()
            .add_test_case(TestParams {
                name: "basic",
                byte_array: default_byte_buffer_generator(),
                offset_start: 0,
                offset_end: 3,
                input: [0x22, 0x55, 0xff, 0x22].as_ref(),
                expected: Ok(()),
            })
            .add_test_case(TestParams {
                name: "basic offset",
                byte_array: default_byte_buffer_generator(),
                offset_start: 1,
                offset_end: 4,
                input: [0x22, 0x55, 0xff, 0x22].as_ref(),
                expected: Ok(()),
            })
            .add_test_case(TestParams {
                name: "Data too long for buffer",
                byte_array: default_byte_buffer_generator(),
                offset_start: 0,
                offset_end: 2,
                input: [0x22, 0x55, 0x00, 0x22].as_ref(),
                expected: Err(super::Error::BufferTooSmall),
            })
            .add_test_case(TestParams {
                name: "Zero size buffer",
                byte_array: get_n_byte_buffer(0),
                offset_start: 0,
                offset_end: 7,
                input: [0x22, 0x55, 0x00, 0x22].as_ref(),
                expected: Err(super::Error::OffsetOutOfBounds),
            })
            .execute(super::put_bytes, |_, _, _, _, _| {});
    }
}
