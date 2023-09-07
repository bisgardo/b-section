#[cfg(test)]
pub mod helpers {
    pub fn new_lookup<T: Clone>(arr: &[T]) -> impl Fn(i64) -> Result<T, String> + '_ {
        |idx| {
            if idx < 0 {
                return Err(format!("negative index {}", idx));
            }
            let idx = idx as usize;
            if idx >= arr.len() {
                return Err(format!("index {} out of bounds", idx));
            }
            Ok(arr[idx].clone())
        }
    }
}
