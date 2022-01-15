pub fn state_changed<T>(state: &mut T, new_state: T) -> bool
where
    T: PartialEq,
{
    if *state != new_state {
        *state = new_state;
        return true;
    }

    false
}

pub fn host() -> &'static str {
    unsafe { &crate::HOST }
}
