use std::{
    boxed::Box,
    ops::{Deref, DerefMut, Drop, FnMut},
};

/// The DropGuard will remain to `Send` and `Sync` from `T`.
///
/// # Examples
///
/// The `LinkedList<T>` is `Send`.
/// So the `DropGuard` will be too, but it will not be `Sync`:
///
/// ```
/// use drop_guard::DropGuard;
/// use std::collections::LinkedList;
/// use std::thread;
///
/// let list: LinkedList<u32> = LinkedList::new();
///
/// let a_list = DropGuard::new(list, |_| {});
///
/// // Send the guarded list to another thread
/// thread::spawn(move || {
///     assert_eq!(0, a_list.len());
/// }).join();
/// ```
pub struct DropGuard<T, F: FnMut(T)> {
    data: Option<T>,
    func: Box<F>,
}

impl<T: Sized, F: FnMut(T)> DropGuard<T, F> {
    /// Creates a new guard taking in your data and a function.
    ///
    /// ```
    /// use drop_guard::DropGuard;
    ///
    /// let s = String::from("a commonString");
    /// let mut s = DropGuard::new(s, |final_string| println!("s became {} at last", final_string));
    ///
    /// // much code and time passes by ...
    /// *s = "a rainbow".to_string();
    ///
    /// // by the end of this function the String will have become a rainbow
    /// ```
    pub fn new(data: T, func: F) -> DropGuard<T, F> {
        DropGuard { data: Some(data), func: Box::new(func) }
    }
}


/// Use the captured value.
///
/// ```
/// use drop_guard::DropGuard;
///
/// let val = DropGuard::new(42usize, |_| {});
/// assert_eq!(42, *val);
/// ```
impl<T, F: FnMut(T)> Deref for DropGuard<T, F> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data.as_ref().expect("the data is here until the drop")
    }
}

/// Modify the captured value.
///
/// ```
/// use drop_guard::DropGuard;
///
/// let mut val = DropGuard::new(vec![2, 3, 4], |_| {});
/// assert_eq!(3, val.len());
///
/// val.push(5);
/// assert_eq!(4, val.len());
/// ```
impl<T, F: FnMut(T)> DerefMut for DropGuard<T, F> {
    fn deref_mut(&mut self) -> &mut T {
        self.data.as_mut().expect("the data is here until the drop")
    }
}

/// React to dropping the value.
/// In this example we measure the time the value is alive.
///
/// ```
/// use drop_guard::DropGuard;
/// use std::time::Instant;
///
/// let start_time = Instant::now();
/// let val = DropGuard::new(42usize, |_| {
///     let time_alive = start_time.elapsed();
///     println!("value lived for {}ns", time_alive.subsec_nanos())
/// });
/// assert_eq!(42, *val);
/// ```
impl<T, F: FnMut(T)> Drop for DropGuard<T, F> {
    fn drop(&mut self) {
        let mut data: Option<T> = None;
        std::mem::swap(&mut data, &mut self.data);
        let ref mut f = self.func;
        f(data.expect("the data is here until the drop"));
    }
}
