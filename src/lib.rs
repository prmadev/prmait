pub mod files;
pub mod git;
pub mod input;
pub mod journal;
pub mod tasks;
pub mod time;

pub fn fold_or_err<T, E>(mut accu: Vec<T>, item: Result<T, E>) -> Result<Vec<T>, E> {
    accu.push(item?);
    Ok(accu)
}
