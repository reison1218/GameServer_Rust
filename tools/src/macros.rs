pub trait GetMutRef {
    fn get_mut_ref(&self) -> &mut Self {
        let ptr = self as *const Self;
        let ptr = ptr as *mut Self;
        unsafe { ptr.as_mut().unwrap() }
    }
}
#[macro_export(local_inner_macros)]
macro_rules! get_mut_ref {
    ($e:ty) => {
        impl GetMutRef for $e {}
    };
}
