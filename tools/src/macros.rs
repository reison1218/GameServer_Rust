///获得可变指针trait
pub trait GetMutRef {
    fn get_mut_ref(&self) -> &mut Self {
        let ptr = self as *const Self;
        let ptr = ptr as *mut Self;
        unsafe { ptr.as_mut().unwrap() }
    }
}
///将&self指针转换成&mut self指针
#[macro_export(local_inner_macros)]
macro_rules! get_mut_ref {
    ($e:ty) => {
        impl tools::macros::GetMutRef for $e {}
    };
}
