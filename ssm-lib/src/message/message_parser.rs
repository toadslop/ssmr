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
fn put_string(byte_array: &mut [u8], span: Span, input_string: &str) -> Result<(), Error> {
    let Span(offset_start, offset_end) = span;
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
fn put_bytes(byte_array: &mut [u8], span: Span, input_bytes: &[u8]) -> Result<(), Error> {
    let Span(offset_start, offset_end) = span;
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

/// The original implementation had this function so we provide it here too for consistency's sake.
/// It helps because the implementation only uses big-endian, so we can specify that here.
#[allow(dead_code)]
fn long_to_bytes(input: i64) -> [u8; 8] {
    input.to_be_bytes()
}

fn put_long(byte_array: &mut [u8], span: Span, value: i64) -> Result<(), Error> {
    todo!()
}

#[derive(Debug, Clone, Copy)]
struct Span(usize, usize);

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self(start, end)
    }

    pub fn span_8(start: usize) -> Self {
        Self::new(start, start + 8)
    }
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
        span: super::Span,
        input: I,
        expected_buffer: &'static [u8],
        expected_output: Result<(), super::Error>,
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

    impl HasLen for i64 {
        fn len(&self) -> usize {
            8
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
        Input: Clone + Debug + Default + HasLen,
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
            mut test_func: impl FnMut(&mut [u8], super::Span, Input) -> Result<(), super::Error>,
            additional_assertions: impl Fn(&[u8], super::Span, Input, &str),
        ) {
            for case in self.test_cases {
                let TestParams::<Input> {
                    name,
                    mut byte_array,
                    span,
                    input,
                    expected_buffer,
                    expected_output,
                } = case;
                let super::Span(offset_start, offset_end) = span;
                let original_byte_array = byte_array.clone();
                let result = test_func(&mut byte_array, span, input.clone());

                assert_eq!(
                    result, expected_output,
                    "{name}: expected output does not match"
                );
                assert_eq!(
                    byte_array, expected_buffer,
                    "{name}: modified byte array is not as expected"
                );

                if result.is_ok() {
                    let byte_array_start = original_byte_array[0..offset_start].to_vec();
                    let byte_array_end = original_byte_array[(offset_end + 1)..].to_vec();

                    assert_eq!(
                        &byte_array[offset_start..(offset_start + input.len())],
                        &expected_buffer[offset_start..(offset_start + input.len())],
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

                    additional_assertions(&byte_array, span, input, name);
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
                span: super::Span(0, 7),
                input: "hello",
                expected_buffer: b"hello  \0",
                expected_output: Ok(()),
            })
            .add_test_case(TestParams {
                name: "basic offset",
                byte_array: default_byte_buffer_generator(),
                span: super::Span(1, 7),
                input: "hello",
                expected_buffer: b"\0hello  ",
                expected_output: Ok(()),
            })
            .add_test_case(TestParams {
                name: "Data too long for buffer",
                byte_array: default_byte_buffer_generator(),
                span: super::Span(0, 7),
                input: "longinputstring",
                expected_buffer: &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                expected_output: Err(super::Error::BufferTooSmall),
            })
            .add_test_case(TestParams {
                name: "Size 0 buffer",
                byte_array: get_n_byte_buffer(0),
                span: super::Span(0, 7),
                input: "longinputstring",
                expected_buffer: &[],
                expected_output: Err(super::Error::OffsetOutOfBounds),
            })
            .execute(super::put_string, |byte_array, span, input, name| {
                let super::Span(offset_start, offset_end) = span;
                let trailing = byte_array[(offset_start + input.len())
                    ..(offset_start + input.len() + offset_end - input.len())]
                    .to_vec();

                let trailing_should_be = " ".repeat(offset_end - input.len());

                assert_eq!(
                    trailing,
                    trailing_should_be.as_bytes(),
                    "{name}: any leftofter offset should be \\s character"
                );
            });
    }

    #[test]
    fn put_bytes() {
        TestSuite::new()
            .add_test_case(TestParams {
                name: "basic",
                byte_array: default_byte_buffer_generator(),
                span: super::Span::new(0, 3),
                input: [0x22, 0x55, 0xff, 0x22].as_ref(),
                expected_buffer: &[0x22, 0x55, 0xff, 0x22, 0x00, 0x00, 0x00, 0x00],
                expected_output: Ok(()),
            })
            .add_test_case(TestParams {
                name: "basic offset",
                byte_array: default_byte_buffer_generator(),
                span: super::Span::new(1, 4),
                input: [0x22, 0x55, 0xff, 0x22].as_ref(),
                expected_buffer: &[0x00, 0x22, 0x55, 0xff, 0x22, 0x00, 0x00, 0x00],
                expected_output: Ok(()),
            })
            .add_test_case(TestParams {
                name: "Data too long for buffer",
                byte_array: default_byte_buffer_generator(),
                span: super::Span::new(0, 2),
                input: [0x22, 0x55, 0x00, 0x22].as_ref(),
                expected_buffer: &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                expected_output: Err(super::Error::BufferTooSmall),
            })
            .add_test_case(TestParams {
                name: "Zero size buffer",
                byte_array: get_n_byte_buffer(0),
                span: super::Span::new(0, 7),
                input: [0x22, 0x55, 0x00, 0x22].as_ref(),
                expected_buffer: &[],
                expected_output: Err(super::Error::OffsetOutOfBounds),
            })
            .execute(super::put_bytes, |_, _, _, _| {});
    }

    #[test]
    /// Although this test trivially tests the behavior of a standard libary function,
    /// which may make the test seem pointless, it serves mostly to help confirm that
    /// the library converts to bytes using big-endian.
    fn long_to_bytes() {
        let actual = super::long_to_bytes(5_747_283);
        let expected = [0x00, 0x00, 0x00, 0x00, 0x00, 0x57, 0xb2, 0x53];

        assert_eq!(actual, expected);
    }

    // #[test]
    // fn test_put_long() {
    //     TestSuite::new()
    //         .add_test_case(TestParams {
    //             name: "Basic",
    //             byte_array: get_n_byte_buffer(9),
    //             span: super::Span(0, 0),
    //             input: 5_747_283,
    //             expected_buffer: &[],
    //             expected_output: Ok(()),
    //         })
    //         .execute(super::put_long, |_, _, _, _| {});
    // }
}
