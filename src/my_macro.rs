#[macro_export]
macro_rules! if_err_continue {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(_e) => continue,
        }
    };
}

#[macro_export]
macro_rules! if_err_return {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(_e) => return,
        }
    };
}

#[macro_export]
macro_rules! if_none_continue {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            _ => continue,
        }
    };
}

#[macro_export]
macro_rules! if_none_return {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            _ => return,
        }
    };
}
