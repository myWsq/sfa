pub mod harness;
pub mod report;
pub mod runner;

#[cfg(test)]
mod tests {
    use crate::harness::{Codec, default_cases, default_matrix};

    #[test]
    fn matrix_contains_tar_and_sfa_for_each_dataset() {
        let cases = default_cases();
        let matrix = default_matrix();
        assert_eq!(matrix.len(), cases.len() * 2 * 2);
        assert!(matrix.iter().any(|job| job.codec == Codec::Lz4));
        assert!(matrix.iter().any(|job| job.codec == Codec::Zstd));
    }
}
