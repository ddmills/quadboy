// Profiling utilities for Tracy integration

/// Macro for creating Tracy spans with conditional compilation
#[macro_export]
macro_rules! tracy_span {
    ($name:expr) => {
        #[cfg(feature = "tracy")]
        let _span = tracy_client::span!($name);
    };
}

/// Macro for marking Tracy frame boundaries
#[macro_export]
macro_rules! tracy_frame_mark {
    () => {
        #[cfg(feature = "tracy")]
        tracy_client::frame_mark();
    };
}

/// Macro for Tracy plot values
#[macro_export]
macro_rules! tracy_plot {
    ($name:expr, $value:expr) => {
        #[cfg(feature = "tracy")]
        tracy_client::plot!($name, $value);
    };
}
