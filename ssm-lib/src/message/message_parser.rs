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
        log::error!("putString failed: Offset is invalid.");
        Err(Error::OffsetOutOfBounds)?;
    }

    if offset_end - offset_start + 1 < input_string.len() {
        log::error!("putString failed: Not enough space to save the string");
        Err(Error::BufferTooSmall)?;
    }

    // wipe out the array location first and then insert the new value. (comment from original)
    // TODO: why is this necessary? Can't we just overwrite without clearing first?
    byte_array
        .iter_mut()
        .take(offset_end)
        .for_each(|byte| *byte = b' ');

    byte_array[..input_string.len()].copy_from_slice(input_string.as_bytes());

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
    struct TestParams<I> {
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

    #[test]
    fn put_string() {
        let test_cases = [
            TestParams {
                byte_array: default_byte_buffer_generator(),
                offset_start: 0,
                offset_end: 7,
                input: "hello",
                expected: Ok(()),
            },
            TestParams {
                byte_array: default_byte_buffer_generator(),
                offset_start: 1,
                offset_end: 7,
                input: "hello",
                expected: Ok(()),
            },
            // This test case existed in the original implementation, but is not included here because
            // we made the case impossible by using an unsigned integer
            // TestParams {
            //     name: "Bad offset",
            //     expectation: Expectation::Failure,
            //     byte_array: default_byte_buffer_generator(),
            //     offset_start: -1,
            //     offset_end: 7,
            //     input: "hello",
            //     expected: "Offset is outside",
            // },
            TestParams {
                byte_array: default_byte_buffer_generator(),
                offset_start: 0,
                offset_end: 7,
                input: "longinputstring",
                expected: Err(super::Error::BufferTooSmall),
            },
            // Cannot put anything in a 0-length buffer
            TestParams {
                byte_array: get_n_byte_buffer(0),
                offset_start: 0,
                offset_end: 7,
                input: "longinputstring",
                expected: Err(super::Error::OffsetOutOfBounds),
            },
        ];

        for case in test_cases {
            let TestParams::<&str> {
                mut byte_array,
                offset_start,
                offset_end,
                input,
                expected,
            } = case;

            let result = super::put_string(&mut byte_array, offset_start, offset_end, input);

            assert_eq!(result, expected);

            if result.is_ok() {
                let output = String::from_utf8(byte_array)
                    .expect("Should be able to convert input to utf8 string");

                assert!(output.contains(input));
            }
        }
    }
}
