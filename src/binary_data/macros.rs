#[macro_export]
macro_rules! next_value {
    ( str; $buf:expr ) => {{
        let mut vec = [0u8; 2];

        let size = match $buf.read(&mut vec) {
            Ok(2) => u16::from_le_bytes(vec) as usize,
            Err(_) | Ok(_) => return None,
        };

        let mut data = vec![0; size];

        let data = match $buf.read(&mut data) {
            Ok(0) | Err(_) => return None,
            Ok(_) => data,
        };

        data
    }};
    ( u8; $buf:expr ) => {{
        let mut data: [u8; 1] = [0];

        let data = match $buf.read(&mut data) {
            Ok(0) | Err(_) => return None,
            Ok(_) => data[0],
        };

        data
    }};
}
