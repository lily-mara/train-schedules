#[macro_export]
macro_rules! classes {
    ($($tail:tt)*) => {
        {
            let mut s = String::new();

            _classes_internal!(s; $($tail)*);

            s
        }
    }
}

#[macro_export]
macro_rules! _classes_internal {
    () => {};

    ($_class_str:expr; $class_name:expr => $condition:expr) => {
        {
            if $condition {
                $_class_str.push(' ');
                $_class_str.push_str(&$class_name);
            }
        }
    };

    ($_class_str:expr; $class_name:expr) => {
        _classes_internal!($_class_str; $class_name => true);
    };

    ($_class_str:expr; $class_name:expr => $condition:expr, $($tail:tt)*) => {
        {
            _classes_internal!($_class_str; $class_name => $condition);
            _classes_internal!($_class_str; $($tail)*);
        }
    };

    ($_class_str:expr; $class_name:expr, $($tail:tt)*) => {
        _classes_internal!($_class_str; $class_name);
        _classes_internal!($_class_str; $($tail)*);
    };
}
