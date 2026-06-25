pub struct ByteUtils;

impl ByteUtils {
    pub fn bytes_to_mb(bytes: usize) -> f64 {
        let mb = bytes as f64 / 1024.0 / 1024.0;
        (mb * 100.0).round() / 100.0
    }
}

#[cfg(test)]
mod test {

    use super::ByteUtils;

    #[test]
    fn test_1() {
        let bytes = 5_250_000;
        println!("{}", ByteUtils::bytes_to_mb(bytes));
    }
}
