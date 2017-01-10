extern crate gl;

/// evaluate the expression, then check for GL error.
#[macro_export]
macro_rules! glcheck {
    ($e: expr) => (
        {
            $e;
            assert_eq!(unsafe {gl::GetError()}, 0);
        }
    )
}